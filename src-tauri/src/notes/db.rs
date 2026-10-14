// SQLite 数据层：rusqlite + WAL + 事务 version + 版本化迁移。
use std::path::Path;
use std::sync::Mutex;

use rusqlite::{params, types::Value, Connection, OptionalExtension, Row};

use super::models::{Cursor, MetaIndex, Page, PageList, PageMeta, Stats, Tag, UpdateResult, View};
use super::summary::{default_project_dir, escape_like, make_summary, nanoid, now_ms};

pub struct Database {
    conn: Mutex<Connection>,
}

const PAGE_LIMIT: i64 = 51;

// 过滤条件唯一来源：list / list_outlines / stats 三处共用，保证计数与列表永远一致。
const NOTE_ACTIVE: &str = "deleted_at IS NULL AND kind = 'note'";
const NOTE_TRASH: &str = "deleted_at IS NOT NULL AND kind = 'note'";
const OUTLINE_ACTIVE: &str = "deleted_at IS NULL AND kind = 'outline'";
const OUTLINE_TRASH: &str = "deleted_at IS NOT NULL AND kind = 'outline'";

fn parse_strings(s: &str) -> Vec<String> {
    serde_json::from_str(s).unwrap_or_default()
}

/// 新建便签的默认项目：本 Note 项目的上一层文件夹（编译期路径，如 D:\MyProject）。
fn default_projects_json() -> String {
    match default_project_dir() {
        Some(d) => serde_json::to_string(&vec![d]).unwrap_or_else(|_| "[]".to_string()),
        None => "[]".to_string(),
    }
}

/// 从某 JSON 数组列聚合去重后的取值（用于标签/项目下拉候选）。
fn distinct_json_values(conn: &Connection, col: &str) -> rusqlite::Result<Vec<String>> {
    let sql = format!(
        "SELECT DISTINCT json_each.value FROM pages, json_each(pages.{col})
         WHERE json_each.value <> '' ORDER BY json_each.value"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

fn row_to_meta(row: &Row) -> rusqlite::Result<PageMeta> {
    let tags: String = row.get(7)?;
    let projects: String = row.get(8)?;
    let from_view: Option<String> = row.get(9)?;
    Ok(PageMeta {
        id: row.get(0)?,
        summary: row.get(1)?,
        created_at: row.get(2)?,
        deleted_at: row.get(3)?,
        version: row.get(4)?,
        title: row.get(5)?,
        kind: row.get(6)?,
        tags: parse_strings(&tags),
        projects: parse_strings(&projects),
        from_view,
    })
}

fn row_to_page(row: &Row) -> rusqlite::Result<Page> {
    let tags: String = row.get(9)?;
    let projects: String = row.get(10)?;
    let from_view: Option<String> = row.get(11)?;
    let nodes_json: Option<String> = row.get(12)?;
    Ok(Page {
        id: row.get(0)?,
        content: row.get(1)?,
        summary: row.get(2)?,
        created_at: row.get(3)?,
        updated_at: row.get(4)?,
        deleted_at: row.get(5)?,
        version: row.get(6)?,
        title: row.get(7)?,
        kind: row.get(8)?,
        tags: parse_strings(&tags),
        projects: parse_strings(&projects),
        from_view,
        nodes: nodes_json.and_then(|s| serde_json::from_str(&s).ok()),
    })
}

impl Database {
    /// 打开文件数据库并完成建表迁移。
    pub fn open(path: &Path) -> rusqlite::Result<Self> {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let conn = Connection::open(path)?;
        conn.busy_timeout(std::time::Duration::from_secs(5))?;
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA foreign_keys = ON;",
        )?;
        let db = Database {
            conn: Mutex::new(conn),
        };
        db.migrate()?;
        Ok(db)
    }

    /// 内存数据库（测试用）。
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn open_in_memory() -> rusqlite::Result<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Database {
            conn: Mutex::new(conn),
        };
        db.migrate()?;
        Ok(db)
    }

    fn migrate(&self) -> rusqlite::Result<()> {
        let mut conn = self.conn.lock().unwrap();

        // v1 基础结构（老库与全新库统一起点，IF NOT EXISTS 幂等）。
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                version integer primary key,
                applied_at integer not null
             );
             CREATE TABLE IF NOT EXISTS pages (
                id          text primary key,
                content     text not null default '',
                summary     text not null default '',
                created_at  integer not null,
                updated_at  integer not null,
                deleted_at  integer,
                version     integer not null default 1
             );
             CREATE INDEX IF NOT EXISTS idx_active ON pages(created_at desc) WHERE deleted_at IS NULL;
             CREATE INDEX IF NOT EXISTS idx_trash ON pages(deleted_at desc) WHERE deleted_at IS NOT NULL;",
        )?;

        let current: i64 = conn.query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
            [],
            |r| r.get(0),
        )?;

        // v2：便签元数据（标题 / 类型 / 标签 / 项目 / 视图来源 / 结构树）。
        if current < 2 {
            let tx = conn.transaction()?;
            tx.execute_batch(
                "ALTER TABLE pages ADD COLUMN title      text not null default '';
                 ALTER TABLE pages ADD COLUMN kind       text not null default 'note';
                 ALTER TABLE pages ADD COLUMN tags       text not null default '[]';
                 ALTER TABLE pages ADD COLUMN projects   text not null default '[]';
                 ALTER TABLE pages ADD COLUMN from_view  text;
                 ALTER TABLE pages ADD COLUMN nodes_json text;",
            )?;
            tx.execute(
                "INSERT INTO schema_migrations (version, applied_at) VALUES (2, ?1)",
                params![now_ms()],
            )?;
            tx.commit()?;
        }

        // v3：标签系统（ID + 状态 + 说明 + 多对多关联）。
        if current < 3 {
            let tx = conn.transaction()?;
            tx.execute_batch(
                "CREATE TABLE tags (
                    id          text primary key,
                    name        text not null,
                    description text not null default '',
                    extinguished integer not null default 0,
                    deleted_at  integer
                 );
                 CREATE TABLE card_tags (
                    card_id text not null,
                    tag_id  text not null,
                    PRIMARY KEY (card_id, tag_id)
                 );
                 CREATE INDEX idx_card_tags_tag ON card_tags(tag_id);",
            )?;

            // 旧 pages.tags 存的是名字数组，迁移成 tags（ID）+ card_tags，并把 pages.tags 改写为 ID 数组。
            let mut name_to_id: std::collections::HashMap<String, String> =
                std::collections::HashMap::new();
            let mut pages: Vec<(String, Vec<String>)> = Vec::new();
            {
                let mut stmt = tx.prepare("SELECT id, tags FROM pages")?;
                let rows = stmt.query_map([], |r| {
                    Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
                })?;
                for r in rows {
                    let (pid, tags_json) = r?;
                    let names: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
                    let ids: Vec<String> = names
                        .into_iter()
                        .filter(|n| !n.is_empty())
                        .map(|name| {
                            name_to_id
                                .entry(name.clone())
                                .or_insert_with(|| nanoid(21))
                                .clone()
                        })
                        .collect();
                    pages.push((pid, ids));
                }
            }
            for (name, id) in &name_to_id {
                tx.execute(
                    "INSERT INTO tags (id, name) VALUES (?1, ?2)",
                    params![id, name],
                )?;
            }
            for (pid, ids) in &pages {
                let ids_json = serde_json::to_string(ids).unwrap_or_else(|_| "[]".to_string());
                tx.execute("UPDATE pages SET tags = ?2 WHERE id = ?1", params![pid, ids_json])?;
                for tid in ids {
                    tx.execute(
                        "INSERT OR IGNORE INTO card_tags (card_id, tag_id) VALUES (?1, ?2)",
                        params![pid, tid],
                    )?;
                }
            }

            tx.execute(
                "INSERT INTO schema_migrations (version, applied_at) VALUES (3, ?1)",
                params![now_ms()],
            )?;
            tx.commit()?;
        }

        // v4：活标签重名的 DB 级唯一约束。先清理历史重名与孤儿关联，再建部分唯一索引，
        // 避免「已有重名活标签的库」在建索引时崩溃、且下次启动反复失败。
        if current < 4 {
            let tx = conn.transaction()?;

            // 1) 清理孤儿 card_tags（指向已彻底删除卡片的关联，autocommit 时代可能残留）。
            tx.execute(
                "DELETE FROM card_tags WHERE card_id NOT IN (SELECT id FROM pages)",
                [],
            )?;

            // 2) 处理历史重名活标签：每组保留 rowid 最小的一个，其余自动改名 name (2)/(3)…
            let mut dup_groups: Vec<(String, Vec<String>)> = Vec::new();
            {
                let mut stmt = tx.prepare(
                    "SELECT name FROM tags WHERE extinguished = 0 GROUP BY name HAVING COUNT(*) > 1",
                )?;
                let names: Vec<String> = stmt
                    .query_map([], |r| r.get::<_, String>(0))?
                    .collect::<Result<_, _>>()?;
                drop(stmt);
                let mut stmt2 = tx.prepare(
                    "SELECT id FROM tags WHERE name = ?1 AND extinguished = 0 ORDER BY rowid",
                )?;
                for name in names {
                    let ids: Vec<String> = stmt2
                        .query_map(params![name], |r| r.get::<_, String>(0))?
                        .collect::<Result<_, _>>()?;
                    if ids.len() > 1 {
                        dup_groups.push((name, ids));
                    }
                }
            }
            for (name, ids) in &dup_groups {
                let mut k = 2;
                for id in ids.iter().skip(1) {
                    let mut new_name = format!("{name} ({k})");
                    loop {
                        let exists: i64 = tx.query_row(
                            "SELECT COUNT(*) FROM tags WHERE name = ?1 AND extinguished = 0",
                            params![new_name],
                            |r| r.get(0),
                        )?;
                        if exists == 0 {
                            break;
                        }
                        k += 1;
                        new_name = format!("{name} ({k})");
                    }
                    tx.execute("UPDATE tags SET name = ?2 WHERE id = ?1", params![id, new_name])?;
                    k += 1;
                }
            }

            // 3) 建部分唯一索引（只约束活标签）。
            tx.execute_batch(
                "CREATE UNIQUE INDEX IF NOT EXISTS idx_tags_active_name
                 ON tags(name) WHERE extinguished = 0;",
            )?;

            tx.execute(
                "INSERT INTO schema_migrations (version, applied_at) VALUES (4, ?1)",
                params![now_ms()],
            )?;
            tx.commit()?;
        }

        // v5：导入账本 —— 记「哪些上游文本已经拉过」。
        //
        // 刻意【不依附卡片】：卡片被彻底删除（purge）后这一行仍然在，同一段文本因此
        // 不会第二次进来。也正因为如此，这里不建指向 pages 的外键、purge 也不级联删它。
        if current < 5 {
            let tx = conn.transaction()?;
            tx.execute_batch(
                "CREATE TABLE IF NOT EXISTS imported_texts (
                    text_id     TEXT PRIMARY KEY,
                    page_id     TEXT,
                    imported_at INTEGER NOT NULL
                 );",
            )?;
            tx.execute(
                "INSERT INTO schema_migrations (version, applied_at) VALUES (5, ?1)",
                params![now_ms()],
            )?;
            tx.commit()?;
        }
        Ok(())
    }

    // ---------- 查询 ----------

    fn get_locked(conn: &Connection, id: &str) -> rusqlite::Result<Option<Page>> {
        conn.query_row(
            "SELECT id, content, summary, created_at, updated_at, deleted_at, version,
                    title, kind, tags, projects, from_view, nodes_json
             FROM pages WHERE id = ?1",
            params![id],
            row_to_page,
        )
        .optional()
    }

    pub fn get(&self, id: &str) -> rusqlite::Result<Option<Page>> {
        let conn = self.conn.lock().unwrap();
        Self::get_locked(&conn, id)
    }

    /// 只读对外接口用：全量变更清单，**只有 id / updated_at / deleted_at 三列**。
    ///
    /// 含已删除条目（`deleted_at` 非空），外部据此识别删除。
    /// 刻意不取正文、标题、标签 —— 这是给轮询用的轻量接口，全量下发也不大。
    pub fn changes_all(&self) -> rusqlite::Result<Vec<(String, i64, Option<i64>)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT id, updated_at, deleted_at FROM pages ORDER BY updated_at DESC, id DESC")?;
        let rows = stmt.query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, Option<i64>>(2)?,
            ))
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    /// 分页列表。`query` 为原始搜索串（内部做 LIKE 转义），空串匹配全部。
    /// 便签页只返回 kind='note'；回收站返回全部（含视图）。
    /// `tags` 为 AND（全部命中），`projects` 为 OR（任一命中）。
    pub fn list(
        &self,
        view: View,
        cursor: Option<&Cursor>,
        query: &str,
        tags: &[String],
        projects: &[String],
    ) -> rusqlite::Result<PageList> {
        let conn = self.conn.lock().unwrap();
        let query = escape_like(query);

        let mut where_parts: Vec<String> = Vec::new();
        let mut params: Vec<Value> = Vec::new();

        match view {
            View::Active => where_parts.push(NOTE_ACTIVE.to_string()),
            View::Trash => where_parts.push(NOTE_TRASH.to_string()),
        }

        if !query.is_empty() {
            where_parts.push(
                "(content LIKE '%' || ? || '%' ESCAPE '\\' OR title LIKE '%' || ? || '%' ESCAPE '\\')"
                    .to_string(),
            );
            params.push(Value::Text(query.clone()));
            params.push(Value::Text(query));
        }

        for t in tags {
            where_parts.push(
                "EXISTS (SELECT 1 FROM json_each(pages.tags) WHERE json_each.value = ?)".to_string(),
            );
            params.push(Value::Text(t.clone()));
        }

        if !projects.is_empty() {
            let ors: Vec<String> = projects
                .iter()
                .map(|_| {
                    "EXISTS (SELECT 1 FROM json_each(pages.projects) WHERE json_each.value = ?)"
                        .to_string()
                })
                .collect();
            where_parts.push(format!("({})", ors.join(" OR ")));
            for p in projects {
                params.push(Value::Text(p.clone()));
            }
        }

        if let Some(c) = cursor {
            match view {
                View::Active => {
                    where_parts.push("(created_at < ? OR (created_at = ? AND id < ?))".to_string());
                    params.push(Value::Integer(c.ts));
                    params.push(Value::Integer(c.ts));
                    params.push(Value::Text(c.id.clone()));
                }
                View::Trash => {
                    where_parts.push("(deleted_at < ? OR (deleted_at = ? AND id < ?))".to_string());
                    params.push(Value::Integer(c.ts));
                    params.push(Value::Integer(c.ts));
                    params.push(Value::Text(c.id.clone()));
                }
            }
        }

        let order = match view {
            View::Active => "created_at DESC, id DESC",
            View::Trash => "deleted_at DESC, id DESC",
        };
        let sql = format!(
            "SELECT id, summary, created_at, deleted_at, version, title, kind, tags, projects, from_view FROM pages WHERE {} ORDER BY {} LIMIT ?",
            where_parts.join(" AND "),
            order
        );
        params.push(Value::Integer(PAGE_LIMIT));

        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(rusqlite::params_from_iter(params.iter()), row_to_meta)?;
        let mut items: Vec<PageMeta> = Vec::new();
        for r in rows {
            items.push(r?);
        }

        let has_more = items.len() as i64 == PAGE_LIMIT;
        let next_cursor = if has_more {
            items.last().map(|m| Cursor {
                ts: match view {
                    View::Active => m.created_at,
                    View::Trash => m.deleted_at.unwrap_or(0),
                },
                id: m.id.clone(),
            })
        } else {
            None
        };

        Ok(PageList {
            items,
            has_more,
            next_cursor,
        })
    }

    // ---------- 写操作 ----------

    pub fn create(&self, kind: &str) -> rusqlite::Result<Page> {
        let conn = self.conn.lock().unwrap();
        let id = nanoid(21);
        let now = now_ms();
        let kind = if kind.is_empty() { "note" } else { kind };
        let projects_json = default_projects_json();
        let nodes_json: Option<String> = if kind == "outline" { Some("[]".to_string()) } else { None };
        conn.execute(
            "INSERT INTO pages (id, content, summary, created_at, updated_at, deleted_at, version,
                                title, kind, tags, projects, nodes_json)
             VALUES (?1, '', '', ?2, ?2, NULL, 1, '', ?3, '[]', ?4, ?5)",
            params![id, now, kind, projects_json, nodes_json],
        )?;
        Ok(Page {
            id,
            content: String::new(),
            summary: String::new(),
            created_at: now,
            updated_at: now,
            deleted_at: None,
            version: 1,
            title: String::new(),
            kind: kind.to_string(),
            tags: Vec::new(),
            projects: parse_strings(&projects_json),
            from_view: None,
            nodes: nodes_json.as_deref().and_then(|s| serde_json::from_str(s).ok()),
        })
    }

    /// 更新内容与摘要，version 在同一事务内递增。未提供的元数据（title/tags/projects）保留原值。
    /// 返回 None 表示页不存在。
    pub fn update(
        &self,
        id: &str,
        content: &str,
        title: Option<&str>,
        tags: Option<&[String]>,
        projects: Option<&[String]>,
        from_view: Option<&str>,
    ) -> rusqlite::Result<Option<UpdateResult>> {
        let mut conn = self.conn.lock().unwrap();
        let summary = make_summary(content);
        let now = now_ms();
        let tx = conn.transaction()?;

        let cur: Option<(String, String, String, Option<String>)> = tx
            .query_row(
                "SELECT title, tags, projects, from_view FROM pages WHERE id = ?1",
                params![id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .optional()?;
        if cur.is_none() {
            tx.commit()?;
            return Ok(None);
        }
        let (cur_title, cur_tags, cur_projects, cur_from_view) = cur.unwrap();
        let new_title = title.map(|s| s.to_string()).unwrap_or(cur_title);
        let new_tags = match tags {
            Some(t) => serde_json::to_string(t).unwrap_or_else(|_| "[]".to_string()),
            None => cur_tags,
        };
        let new_projects = match projects {
            Some(p) => serde_json::to_string(p).unwrap_or_else(|_| "[]".to_string()),
            None => cur_projects,
        };
        let new_from_view = match from_view {
            Some("") => None,
            Some(s) => Some(s.to_string()),
            None => cur_from_view,
        };

        tx.execute(
            "UPDATE pages SET content = ?2, summary = ?3, updated_at = ?4, version = version + 1,
                              title = ?5, tags = ?6, projects = ?7, from_view = ?8
             WHERE id = ?1",
            params![id, content, summary, now, new_title, new_tags, new_projects, new_from_view],
        )?;
        // 标签改了就要同步 card_tags（标签侧关联），两边保持一致。
        if let Some(tag_ids) = tags {
            tx.execute("DELETE FROM card_tags WHERE card_id = ?1", params![id])?;
            for tid in tag_ids {
                tx.execute(
                    "INSERT OR IGNORE INTO card_tags (card_id, tag_id) VALUES (?1, ?2)",
                    params![id, tid],
                )?;
            }
        }
        let version: i64 =
            tx.query_row("SELECT version FROM pages WHERE id = ?1", params![id], |r| r.get(0))?;
        tx.commit()?;
        Ok(Some(UpdateResult {
            id: id.to_string(),
            version,
            summary,
            updated_at: now,
        }))
    }

    /// 软删除。返回 (新 version, deleted_at)。
    pub fn soft_delete(&self, id: &str) -> rusqlite::Result<Option<(i64, i64)>> {
        let conn = self.conn.lock().unwrap();
        let now = now_ms();
        let n = conn.execute(
            "UPDATE pages SET deleted_at = ?2, version = version + 1
             WHERE id = ?1 AND deleted_at IS NULL",
            params![id, now],
        )?;
        if n == 0 {
            return Ok(None);
        }
        let version: i64 =
            conn.query_row("SELECT version FROM pages WHERE id = ?1", params![id], |r| r.get(0))?;
        Ok(Some((version, now)))
    }

    /// 恢复（从回收站移回主列表）。返回恢复后的完整页。
    pub fn restore(&self, id: &str) -> rusqlite::Result<Option<Page>> {
        let conn = self.conn.lock().unwrap();
        let n = conn.execute(
            "UPDATE pages SET deleted_at = NULL, version = version + 1
             WHERE id = ?1 AND deleted_at IS NOT NULL",
            params![id],
        )?;
        if n == 0 {
            return Ok(None);
        }
        Self::get_locked(&conn, id)
    }

    /// 彻底删除。返回被删除页的 version（None 表示不在回收站）。
    /// 关联表 card_tags 与页面同事务删除，避免孤儿记录污染计数。
    pub fn purge(&self, id: &str) -> rusqlite::Result<Option<i64>> {
        let mut conn = self.conn.lock().unwrap();
        let version: Option<i64> = conn
            .query_row(
                "SELECT version FROM pages WHERE id = ?1 AND deleted_at IS NOT NULL",
                params![id],
                |r| r.get(0),
            )
            .optional()?;
        if version.is_none() {
            return Ok(None);
        }
        let tx = conn.transaction()?;
        tx.execute("DELETE FROM card_tags WHERE card_id = ?1", params![id])?;
        tx.execute("DELETE FROM pages WHERE id = ?1", params![id])?;
        tx.commit()?;
        Ok(version)
    }

    pub fn stats(&self) -> rusqlite::Result<Stats> {
        let conn = self.conn.lock().unwrap();
        let active: i64 = conn.query_row(
            &format!("SELECT COUNT(*) FROM pages WHERE {NOTE_ACTIVE}"),
            [],
            |r| r.get(0),
        )?;
        let outline: i64 = conn.query_row(
            &format!("SELECT COUNT(*) FROM pages WHERE {OUTLINE_ACTIVE}"),
            [],
            |r| r.get(0),
        )?;
        let trash: i64 = conn.query_row(
            &format!("SELECT COUNT(*) FROM pages WHERE {NOTE_TRASH}"),
            [],
            |r| r.get(0),
        )?;
        let trash_outline: i64 = conn.query_row(
            &format!("SELECT COUNT(*) FROM pages WHERE {OUTLINE_TRASH}"),
            [],
            |r| r.get(0),
        )?;
        Ok(Stats {
            active,
            trash,
            outline,
            trash_outline,
        })
    }

    /// 聚合当前所有标签（含熄灭）与项目，作为下拉候选。
    pub fn meta_index(&self) -> rusqlite::Result<MetaIndex> {
        let conn = self.conn.lock().unwrap();
        let tags = Self::list_tags_locked(&conn)?;
        let projects = distinct_json_values(&conn, "projects")?;
        // default_project 是编译期路径常量，与数据库无关，由上层命令填充。
        Ok(MetaIndex {
            tags,
            projects,
            default_project: None,
        })
    }

    // ---------- 标签 ----------

    fn row_to_tag(row: &Row) -> rusqlite::Result<Tag> {
        let ext: i64 = row.get(3)?;
        Ok(Tag {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            extinguished: ext != 0,
            deleted_at: row.get(4)?,
            card_count: row.get(5)?,
        })
    }

    fn list_tags_locked(conn: &Connection) -> rusqlite::Result<Vec<Tag>> {
        let mut stmt = conn.prepare(
            "SELECT t.id, t.name, t.description, t.extinguished, t.deleted_at,
                    (SELECT COUNT(*) FROM card_tags c
                     JOIN pages p ON c.card_id = p.id
                     WHERE c.tag_id = t.id AND p.deleted_at IS NULL AND p.kind = 'note') AS card_count
             FROM tags t
             ORDER BY t.extinguished ASC, t.name ASC",
        )?;
        let rows = stmt.query_map([], Self::row_to_tag)?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    /// 列出所有标签（含熄灭），每个标签带引用卡片数。
    pub fn list_tags(&self) -> rusqlite::Result<Vec<Tag>> {
        let conn = self.conn.lock().unwrap();
        Self::list_tags_locked(&conn)
    }

    /// 新建激活标签。若已有激活标签同名则报错。
    pub fn create_tag(&self, name: &str, description: &str) -> Result<Tag, String> {
        let conn = self.conn.lock().unwrap();
        let name = name.trim().to_string();
        if name.is_empty() {
            return Err("标签名不能为空".to_string());
        }
        let dup: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM tags WHERE name = ?1 AND extinguished = 0",
                params![name],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        if dup > 0 {
            return Err(format!("已有同名标签「{name}」"));
        }
        let id = nanoid(21);
        conn.execute(
            "INSERT INTO tags (id, name, description, extinguished) VALUES (?1, ?2, ?3, 0)",
            params![id, name, description],
        )
        .map_err(|e| e.to_string())?;
        Ok(Tag {
            id,
            name,
            description: description.to_string(),
            extinguished: false,
            deleted_at: None,
            card_count: 0,
        })
    }

    /// 重命名。活标签之间不能重名。
    pub fn rename_tag(&self, id: &str, name: &str) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        let name = name.trim().to_string();
        if name.is_empty() {
            return Err("标签名不能为空".to_string());
        }
        let dup: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM tags WHERE name = ?1 AND extinguished = 0 AND id <> ?2",
                params![name, id],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        if dup > 0 {
            return Err(format!("已有同名标签「{name}」"));
        }
        let n = conn
            .execute("UPDATE tags SET name = ?2 WHERE id = ?1", params![id, name])
            .map_err(|e| e.to_string())?;
        if n == 0 {
            return Err("标签不存在".to_string());
        }
        Ok(())
    }

    /// 更新标签说明。
    pub fn set_tag_description(&self, id: &str, description: &str) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        let n = conn
            .execute(
                "UPDATE tags SET description = ?2 WHERE id = ?1",
                params![id, description],
            )
            .map_err(|e| e.to_string())?;
        if n == 0 {
            return Err("标签不存在".to_string());
        }
        Ok(())
    }

    /// 删除（熄灭）：状态改熄灭，关联保留，卡片立即不显示。
    pub fn delete_tag(&self, id: &str) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        let now = now_ms();
        let n = conn
            .execute(
                "UPDATE tags SET extinguished = 1, deleted_at = ?2 WHERE id = ?1 AND extinguished = 0",
                params![id, now],
            )
            .map_err(|e| e.to_string())?;
        if n == 0 {
            return Err("标签不存在或已熄灭".to_string());
        }
        Ok(())
    }

    /// 恢复（激活）：状态改回激活，关联一直在。若已有活标签同名则报错，需先改名。
    pub fn restore_tag(&self, id: &str) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        let name: String = conn
            .query_row("SELECT name FROM tags WHERE id = ?1", params![id], |r| r.get(0))
            .map_err(|_| "标签不存在".to_string())?;
        let dup: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM tags WHERE name = ?1 AND extinguished = 0 AND id <> ?2",
                params![name, id],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        if dup > 0 {
            return Err(format!("已有同名标签「{name}」，请先改名再恢复"));
        }
        let n = conn
            .execute(
                "UPDATE tags SET extinguished = 0, deleted_at = NULL WHERE id = ?1 AND extinguished = 1",
                params![id],
            )
            .map_err(|e| e.to_string())?;
        if n == 0 {
            return Err("标签不存在或已是激活状态".to_string());
        }
        Ok(())
    }

    /// 合并：新建平级标签 C（C.cards = A∪B...），A/B 熄灭，C 加进 A∪B 的每张卡片。
    pub fn merge_tags(&self, from_ids: &[String], to_name: &str, to_description: &str) -> Result<Tag, String> {
        let mut conn = self.conn.lock().unwrap();
        let to_name = to_name.trim().to_string();
        if to_name.is_empty() {
            return Err("合并后的标签名不能为空".to_string());
        }
        if from_ids.len() < 2 {
            return Err("至少选择两个标签".to_string());
        }
        let from_json = serde_json::to_string(from_ids).unwrap_or_else(|_| "[]".to_string());
        let dup: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM tags WHERE name = ?1 AND extinguished = 0
                 AND id NOT IN (SELECT value FROM json_each(?2))",
                params![to_name, from_json],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        if dup > 0 {
            return Err(format!("已有同名标签「{to_name}」"));
        }

        let tx = conn.transaction().map_err(|e| e.to_string())?;

        // C.cards = A∪B 的卡片并集。
        let mut card_set: std::collections::HashSet<String> = std::collections::HashSet::new();
        {
            let mut stmt = tx
                .prepare("SELECT card_id FROM card_tags WHERE tag_id = ?1")
                .map_err(|e| e.to_string())?;
            for fid in from_ids {
                let rows = stmt
                    .query_map(params![fid], |r| r.get::<_, String>(0))
                    .map_err(|e| e.to_string())?;
                for r in rows {
                    card_set.insert(r.map_err(|e| e.to_string())?);
                }
            }
        }

        let cid = nanoid(21);
        tx.execute(
            "INSERT INTO tags (id, name, description, extinguished) VALUES (?1, ?2, ?3, 0)",
            params![cid, to_name, to_description],
        )
        .map_err(|e| e.to_string())?;

        let now = now_ms();
        for fid in from_ids {
            tx.execute(
                "UPDATE tags SET extinguished = 1, deleted_at = ?2 WHERE id = ?1",
                params![fid, now],
            )
            .map_err(|e| e.to_string())?;
        }

        for card_id in &card_set {
            tx.execute(
                "INSERT OR IGNORE INTO card_tags (card_id, tag_id) VALUES (?1, ?2)",
                params![card_id, cid],
            )
            .map_err(|e| e.to_string())?;
            let tags_json: String = tx
                .query_row("SELECT tags FROM pages WHERE id = ?1", params![card_id], |r| r.get(0))
                .map_err(|e| e.to_string())?;
            let mut ids: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
            if !ids.iter().any(|x| x == &cid) {
                ids.push(cid.clone());
                let new_json = serde_json::to_string(&ids).unwrap_or_else(|_| "[]".to_string());
                tx.execute("UPDATE pages SET tags = ?2 WHERE id = ?1", params![card_id, new_json])
                    .map_err(|e| e.to_string())?;
            }
        }

        tx.commit().map_err(|e| e.to_string())?;

        Ok(Tag {
            id: cid,
            name: to_name,
            description: to_description.to_string(),
            extinguished: false,
            deleted_at: None,
            card_count: card_set.len() as i64,
        })
    }

    /// 列出视图（outline）：active 返回未删除的，trash 返回已删除的。
    pub fn list_outlines(&self, view: View) -> rusqlite::Result<Vec<PageMeta>> {
        let conn = self.conn.lock().unwrap();
        let filter = match view {
            View::Active => OUTLINE_ACTIVE,
            View::Trash => OUTLINE_TRASH,
        };
        let sql = format!(
            "SELECT id, summary, created_at, deleted_at, version, title, kind, tags, projects, from_view FROM pages
             WHERE {filter} ORDER BY created_at DESC, id DESC"
        );
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map([], row_to_meta)?;
        let mut items = Vec::new();
        for r in rows {
            items.push(r?);
        }
        Ok(items)
    }

    /// 保存视图的整棵树（nodes_json）。
    pub fn save_outline(&self, id: &str, nodes_json: &str) -> rusqlite::Result<bool> {
        let conn = self.conn.lock().unwrap();
        let now = now_ms();
        let n = conn.execute(
            "UPDATE pages SET nodes_json = ?2, updated_at = ?3, version = version + 1
             WHERE id = ?1 AND kind = 'outline'",
            params![id, nodes_json, now],
        )?;
        Ok(n > 0)
    }

    // ---------- 导入（外部文本源 → 卡片） ----------

    /// 把上游文本落成【普通卡片】：一次事务，要么全进要么全不进。
    ///
    /// 三条纪律写在这里，别在调用方重复：
    /// - **认 `text_id` 去重**：账本 `imported_texts` 里有的直接跳过 —— 这是「只新增」的
    ///   全部实现。既然从不重导，就不可能覆盖用户改过的卡片。
    /// - **账本与卡片同事务、但不同生命周期**：`purge` 不碰账本，所以卡片彻底删掉之后
    ///   那段文本也不会再进来。
    /// - **标签镜像上游原名**：活标签重名即合并（复用原 ID），没有才新建；
    ///   每张卡片另外自动挂一枚 [`super::import::MOBILE_TAG`]。
    pub fn import_texts(&self, rows: &[ImportedText]) -> Result<ImportOutcome, String> {
        let mut conn = self.conn.lock().map_err(|e| e.to_string())?;
        let now = now_ms();
        let tx = conn.transaction().map_err(|e| e.to_string())?;

        // 账本：本批之外的已导入 text_id。
        let mut known: std::collections::HashSet<String> = std::collections::HashSet::new();
        {
            let mut stmt = tx
                .prepare("SELECT text_id FROM imported_texts")
                .map_err(|e| e.to_string())?;
            let it = stmt
                .query_map([], |r| r.get::<_, String>(0))
                .map_err(|e| e.to_string())?;
            for r in it {
                known.insert(r.map_err(|e| e.to_string())?);
            }
        }

        // 标签名 → 活标签 ID。一次建图，本批新建的也进图（同一批里重名即合并）。
        let mut tag_ids: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        {
            let mut stmt = tx
                .prepare("SELECT name, id FROM tags WHERE extinguished = 0")
                .map_err(|e| e.to_string())?;
            let it = stmt
                .query_map([], |r| {
                    Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
                })
                .map_err(|e| e.to_string())?;
            for r in it {
                let (name, id) = r.map_err(|e| e.to_string())?;
                tag_ids.entry(name).or_insert(id);
            }
        }

        let projects_json = default_projects_json();
        let mut outcome = ImportOutcome::default();

        for row in rows {
            if known.contains(&row.text_id) {
                outcome.skipped_existing += 1;
                continue;
            }
            // 空正文不落卡（会变成一张看不出内容的空白卡），也【不记账】——
            // 上游之后把正文补上时，它仍然是「第一次拉」。
            if row.content.trim().is_empty() {
                outcome.skipped_empty += 1;
                continue;
            }

            let id = nanoid(21);
            let summary = make_summary(&row.content);

            // 标签：自动挂「手机」在前，随后镜像上游原名（去重、去空）。
            let mut names: Vec<String> = vec![super::import::MOBILE_TAG.to_string()];
            for t in &row.tags {
                let t = t.trim();
                if !t.is_empty() && !names.iter().any(|n| n == t) {
                    names.push(t.to_string());
                }
            }
            let mut tag_id_list: Vec<String> = Vec::with_capacity(names.len());
            for name in &names {
                let tid = match tag_ids.get(name) {
                    Some(existing) => existing.clone(),
                    None => {
                        let fresh = nanoid(21);
                        tx.execute(
                            "INSERT INTO tags (id, name, description, extinguished) VALUES (?1, ?2, '', 0)",
                            params![fresh, name],
                        )
                        .map_err(|e| e.to_string())?;
                        tag_ids.insert(name.clone(), fresh.clone());
                        fresh
                    }
                };
                tag_id_list.push(tid);
            }

            let tags_json =
                serde_json::to_string(&tag_id_list).unwrap_or_else(|_| "[]".to_string());
            tx.execute(
                "INSERT INTO pages (id, content, summary, created_at, updated_at, deleted_at,
                                    version, title, kind, tags, projects, nodes_json)
                 VALUES (?1, ?2, ?3, ?4, ?5, NULL, 1, ?6, 'note', ?7, ?8, NULL)",
                params![
                    id,
                    row.content,
                    summary,
                    row.created_at,
                    row.updated_at,
                    row.title,
                    tags_json,
                    projects_json
                ],
            )
            .map_err(|e| e.to_string())?;

            for tid in &tag_id_list {
                tx.execute(
                    "INSERT OR IGNORE INTO card_tags (card_id, tag_id) VALUES (?1, ?2)",
                    params![id, tid],
                )
                .map_err(|e| e.to_string())?;
            }

            tx.execute(
                "INSERT INTO imported_texts (text_id, page_id, imported_at) VALUES (?1, ?2, ?3)",
                params![row.text_id, id, now],
            )
            .map_err(|e| e.to_string())?;

            known.insert(row.text_id.clone());
            outcome.imported += 1;
        }

        tx.commit().map_err(|e| e.to_string())?;
        Ok(outcome)
    }

    /// 账本里已导入的 `text_id` 条数（只用于诊断/自检）。
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn imported_text_count(&self) -> rusqlite::Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.query_row("SELECT COUNT(*) FROM imported_texts", [], |r| r.get(0))
    }
}

/// 一条待导入的上游文本（由 `import` 模块从外部库读出后交给 [`Database::import_texts`]）。
pub struct ImportedText {
    /// 上游主键，形如 `catcher:<源id>`。去重就认它。
    pub text_id: String,
    pub title: String,
    pub content: String,
    /// 上游标签**原名**（不是本库的标签 ID）。
    pub tags: Vec<String>,
    /// 上游自带的时间（epoch 毫秒），不是导入时刻。
    pub created_at: i64,
    pub updated_at: i64,
}

/// 一轮导入的结果。
#[derive(Default, Debug, PartialEq, Eq)]
pub struct ImportOutcome {
    pub imported: i64,
    pub skipped_existing: i64,
    pub skipped_empty: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn db() -> Database {
        Database::open_in_memory().unwrap()
    }

    fn fill(db: &Database, contents: &[&str]) -> Vec<Page> {
        contents
            .iter()
            .map(|c| {
                let p = db.create("note").unwrap();
                db.update(&p.id, c, None, None, None, None).unwrap();
                db.get(&p.id).unwrap().unwrap()
            })
            .collect()
    }

    #[test]
    fn create_and_get() {
        let db = db();
        let p = db.create("note").unwrap();
        assert_eq!(p.content, "");
        assert_eq!(p.summary, "");
        assert_eq!(p.title, "");
        assert_eq!(p.kind, "note");
        assert_eq!(p.version, 1);
        assert!(db.get(&p.id).unwrap().is_some());
    }

    #[test]
    fn update_increments_version_and_summary() {
        let db = db();
        let p = db.create("note").unwrap();
        let r = db.update(&p.id, "hello world", None, None, None, None).unwrap().unwrap();
        assert_eq!(r.version, 2);
        assert_eq!(r.summary, "hello world");
        let got = db.get(&p.id).unwrap().unwrap();
        assert_eq!(got.content, "hello world");
        assert_eq!(got.version, 2);
    }

    #[test]
    fn update_metadata_and_preserve_when_absent() {
        let db = db();
        let p = db.create("note").unwrap();
        db.update(
            &p.id,
            "body",
            Some("标题"),
            Some(&["a".to_string(), "b".to_string()]),
            Some(&["p1".to_string()]),
            None,
        )
        .unwrap();
        let got = db.get(&p.id).unwrap().unwrap();
        assert_eq!(got.title, "标题");
        assert_eq!(got.tags, vec!["a".to_string(), "b".to_string()]);
        assert_eq!(got.projects, vec!["p1".to_string()]);

        // 只改正文、不传元数据 → 保留原值。
        db.update(&p.id, "body2", None, None, None, None).unwrap();
        let got2 = db.get(&p.id).unwrap().unwrap();
        assert_eq!(got2.title, "标题");
        assert_eq!(got2.tags, vec!["a".to_string(), "b".to_string()]);
    }

    #[test]
    fn soft_delete_restore_purge_cycle() {
        let db = db();
        let p = db.create("note").unwrap();
        db.update(&p.id, "temp", None, None, None, None).unwrap();

        let (v, _) = db.soft_delete(&p.id).unwrap().unwrap();
        assert!(v >= 3);
        let stats = db.stats().unwrap();
        assert_eq!(stats.active, 0);
        assert_eq!(stats.trash, 1);

        let restored = db.restore(&p.id).unwrap().unwrap();
        assert_eq!(restored.deleted_at, None);
        assert_eq!(db.stats().unwrap().active, 1);

        db.soft_delete(&p.id).unwrap().unwrap();
        assert_eq!(db.purge(&p.id).unwrap(), Some(restored.version + 1));
        assert_eq!(db.stats().unwrap().trash, 0);
        assert!(db.get(&p.id).unwrap().is_none());
    }

    #[test]
    fn list_active_sorted_by_created_desc() {
        let db = db();
        let pages = fill(&db, &["a", "b", "c"]);
        let list = db.list(View::Active, None, "", &[], &[]).unwrap();
        assert_eq!(list.items.len(), 3);
        // 新建置顶：c > b > a
        assert_eq!(list.items[0].id, pages[2].id);
        assert_eq!(list.items[2].id, pages[0].id);
        assert!(!list.has_more);
    }

    #[test]
    fn search_matches_substring() {
        let db = db();
        fill(&db, &["apple pie", "banana", "pineapple"]);
        let list = db.list(View::Active, None, "apple", &[], &[]).unwrap();
        assert_eq!(list.items.len(), 2); // "apple pie" 与 "pineapple"
    }

    #[test]
    fn search_matches_title() {
        let db = db();
        let p = db.create("note").unwrap();
        db.update(&p.id, "正文内容", Some("项目计划"), None, None, None).unwrap();
        let by_title = db.list(View::Active, None, "项目计划", &[], &[]).unwrap();
        assert_eq!(by_title.items.len(), 1);
        assert_eq!(by_title.items[0].id, p.id);
        let by_body = db.list(View::Active, None, "正文", &[], &[]).unwrap();
        assert_eq!(by_body.items.len(), 1);
    }

    #[test]
    fn search_escapes_like_specials() {
        let db = db();
        fill(&db, &["100%", "a_b", r"a\b", "plain"]);
        // 传入原始字符，list 内部转义后按字面匹配。
        let list = db.list(View::Active, None, "%", &[], &[]).unwrap();
        assert_eq!(list.items.len(), 1);
        assert_eq!(list.items[0].summary, "100%");
        let list = db.list(View::Active, None, "_", &[], &[]).unwrap();
        assert_eq!(list.items.len(), 1);
        assert_eq!(list.items[0].summary, "a_b");
        let list = db.list(View::Active, None, "\\", &[], &[]).unwrap();
        assert_eq!(list.items.len(), 1);
        assert_eq!(list.items[0].summary, r"a\b");
    }

    #[test]
    fn filter_tags_and_projects() {
        let db = db();
        let p1 = db.create("note").unwrap();
        db.update(
            &p1.id,
            "a",
            None,
            Some(&["工作".to_string(), "重要".to_string()]),
            Some(&["P1".to_string()]),
            None,
        )
        .unwrap();
        let p2 = db.create("note").unwrap();
        db.update(&p2.id, "b", None, Some(&["工作".to_string()]), Some(&["P2".to_string()]), None).unwrap();
        let p3 = db.create("note").unwrap();
        db.update(&p3.id, "c", None, Some(&["生活".to_string()]), Some(&["P1".to_string()]), None).unwrap();

        // 标签 AND：工作 + 重要 → 仅 p1
        let r = db
            .list(View::Active, None, "", &["工作".to_string(), "重要".to_string()], &[])
            .unwrap();
        assert_eq!(r.items.len(), 1);
        assert_eq!(r.items[0].id, p1.id);

        // 项目 OR：P2 → 仅 p2
        let r = db.list(View::Active, None, "", &[], &["P2".to_string()]).unwrap();
        assert_eq!(r.items.len(), 1);
        assert_eq!(r.items[0].id, p2.id);

        // 项目 OR：P1 → p1 + p3
        let r = db.list(View::Active, None, "", &[], &["P1".to_string()]).unwrap();
        assert_eq!(r.items.len(), 2);
    }

    #[test]
    fn cursor_pagination_no_overlap() {
        let db = db();
        // 超过一页（PAGE_LIMIT=51）才会真正触发分页。
        let total = 60usize;
        for i in 0..total {
            let p = db.create("note").unwrap();
            db.update(&p.id, &format!("n{}", i), None, None, None, None).unwrap();
        }
        let mut seen: Vec<String> = Vec::new();
        let mut cursor: Option<Cursor> = None;
        loop {
            let list = db.list(View::Active, cursor.as_ref(), "", &[], &[]).unwrap();
            seen.extend(list.items.iter().map(|m| m.id.clone()));
            match list.next_cursor {
                Some(c) => cursor = Some(c),
                None => break,
            }
        }
        assert_eq!(seen.len(), total);
        let mut uniq = seen.clone();
        uniq.sort();
        uniq.dedup();
        assert_eq!(uniq.len(), total);
    }

    #[test]
    fn outline_excluded_from_active_and_counted_separately() {
        let db = db();
        let note = db.create("note").unwrap();
        db.create("outline").unwrap();
        let stats = db.stats().unwrap();
        assert_eq!(stats.active, 1);
        assert_eq!(stats.outline, 1);
        assert_eq!(stats.trash, 0);
        let list = db.list(View::Active, None, "", &[], &[]).unwrap();
        assert_eq!(list.items.len(), 1);
        assert_eq!(list.items[0].id, note.id);
    }

    #[test]
    fn meta_index_collects_distinct_tags_and_projects() {
        let db = db();
        let t1 = db.create_tag("工作", "").unwrap();
        let t2 = db.create_tag("重要", "").unwrap();
        let p1 = db.create("note").unwrap();
        db.update(
            &p1.id,
            "a",
            None,
            Some(&[t1.id.clone(), t2.id.clone()]),
            Some(&["P1".to_string()]),
            None,
        )
        .unwrap();
        let p2 = db.create("note").unwrap();
        db.update(&p2.id, "b", None, Some(&[t1.id.clone()]), Some(&["P2".to_string()]), None).unwrap();
        let mi = db.meta_index().unwrap();
        assert_eq!(mi.tags.len(), 2);
        assert_eq!(mi.tags[0].name, "工作");
        assert_eq!(mi.tags[0].card_count, 2);
        assert_eq!(mi.tags[1].name, "重要");
        assert_eq!(mi.tags[1].card_count, 1);
        assert_eq!(mi.projects, vec!["P1".to_string(), "P2".to_string()]);
    }

    #[test]
    fn outline_create_list_save() {
        let db = db();
        let o = db.create("outline").unwrap();
        assert_eq!(o.nodes, Some(serde_json::json!([])));
        assert_eq!(db.list_outlines(View::Active).unwrap().len(), 1);
        assert!(db.save_outline(&o.id, r#"[{"id":"n1","title":"根"}]"#).unwrap());
        assert!(db.get(&o.id).unwrap().unwrap().nodes.is_some());
    }

    #[test]
    fn tag_lifecycle_create_rename_delete_restore_merge() {
        let db = db();
        let a = db.create_tag("A", "说明A").unwrap();
        let b = db.create_tag("B", "").unwrap();
        assert_eq!(db.list_tags().unwrap().len(), 2);

        // 活标签之间重名报错。
        assert!(db.create_tag("A", "").is_err());
        assert!(db.rename_tag(&b.id, "A").is_err());
        // 改名 + 改说明。
        db.rename_tag(&b.id, "B2").unwrap();
        db.set_tag_description(&b.id, "说明B").unwrap();
        assert_eq!(db.list_tags().unwrap().iter().find(|t| t.id == b.id).unwrap().name, "B2");

        // 卡片打标签（传 ID）。
        let p1 = db.create("note").unwrap();
        db.update(&p1.id, "x", None, Some(&[a.id.clone(), b.id.clone()]), None, None).unwrap();

        // 删除（熄灭）：关联保留，卡片 tags 仍含该 ID。
        db.delete_tag(&b.id).unwrap();
        assert!(db.list_tags().unwrap().iter().find(|t| t.id == b.id).unwrap().extinguished);
        assert!(db.get(&p1.id).unwrap().unwrap().tags.iter().any(|x| x == &b.id));

        // 恢复。
        db.restore_tag(&b.id).unwrap();
        assert!(!db.list_tags().unwrap().iter().find(|t| t.id == b.id).unwrap().extinguished);

        // 合并 A + B → C：A/B 熄灭，C 激活，卡片 tags 含 C。
        let c = db.merge_tags(&[a.id.clone(), b.id.clone()], "C", "合并说明").unwrap();
        assert_eq!(c.name, "C");
        let tags = db.list_tags().unwrap();
        assert!(tags.iter().find(|t| t.id == a.id).unwrap().extinguished);
        assert!(tags.iter().find(|t| t.id == b.id).unwrap().extinguished);
        assert!(!tags.iter().find(|t| t.id == c.id).unwrap().extinguished);
        assert!(db.get(&p1.id).unwrap().unwrap().tags.iter().any(|x| x == &c.id));
    }

    #[test]
    fn migration_adds_columns_to_legacy_db() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE schema_migrations (version integer primary key, applied_at integer not null);
             CREATE TABLE pages (id text primary key, content text not null default '', summary text not null default '', created_at integer not null, updated_at integer not null, deleted_at integer, version integer not null default 1);
             INSERT INTO pages (id, content, summary, created_at, updated_at, deleted_at, version) VALUES ('legacy', 'hello', 'hello', 1, 1, NULL, 1);",
        )
        .unwrap();
        let d = Database {
            conn: Mutex::new(conn),
        };
        d.migrate().unwrap();
        let p = d.get("legacy").unwrap().unwrap();
        assert_eq!(p.title, "");
        assert_eq!(p.kind, "note");
        assert!(p.tags.is_empty());
        assert!(p.projects.is_empty());
        assert!(p.nodes.is_none());
        assert_eq!(p.content, "hello");
    }

    #[test]
    fn migration_v3_moves_tag_names_to_ids() {
        // 造一个 v2 老库：pages.tags 存的是标签「名字」。
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE schema_migrations (version integer primary key, applied_at integer not null);
             INSERT INTO schema_migrations (version, applied_at) VALUES (2, 1);
             CREATE TABLE pages (
               id text primary key, content text not null default '', summary text not null default '',
               created_at integer not null, updated_at integer not null, deleted_at integer,
               version integer not null default 1, title text not null default '',
               kind text not null default 'note', tags text not null default '[]',
               projects text not null default '[]', from_view text, nodes_json text);",
        )
        .unwrap();
        let tags1 = serde_json::to_string(&vec!["工作", "重要"]).unwrap();
        let tags2 = serde_json::to_string(&vec!["工作"]).unwrap();
        conn.execute(
            "INSERT INTO pages (id, content, summary, created_at, updated_at, deleted_at, version, tags)
             VALUES ('p1','a','a',1,1,NULL,1,?1)",
            params![tags1],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO pages (id, content, summary, created_at, updated_at, deleted_at, version, tags)
             VALUES ('p2','b','b',2,2,NULL,1,?1)",
            params![tags2],
        )
        .unwrap();
        let d = Database { conn: Mutex::new(conn) };
        d.migrate().unwrap();
        // 迁移后：tags 表有 2 个标签，pages.tags 已变成 ID（不再是名字）。
        let tags = d.list_tags().unwrap();
        assert_eq!(tags.len(), 2);
        assert_eq!(tags[0].name, "工作");
        assert_eq!(tags[0].card_count, 2);
        assert_eq!(tags[1].name, "重要");
        assert_eq!(tags[1].card_count, 1);
        let p1 = d.get("p1").unwrap().unwrap();
        assert_eq!(p1.tags.len(), 2);
        assert_ne!(p1.tags[0], "工作");
    }

    #[test]
    fn migration_v4_dedupes_duplicate_active_tags() {
        // 造一个 v3 库：两个同名活标签「工作」+ 一个熄灭的「工作」+ 一个「工作 (2)」+ 一条孤儿关联。
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE schema_migrations (version integer primary key, applied_at integer not null);
             INSERT INTO schema_migrations (version, applied_at) VALUES (3, 1);
             CREATE TABLE tags (
               id text primary key, name text not null, description text not null default '',
               extinguished integer not null default 0, deleted_at integer);
             CREATE TABLE card_tags (
               card_id text not null, tag_id text not null, PRIMARY KEY (card_id, tag_id));
             CREATE INDEX idx_card_tags_tag ON card_tags(tag_id);
             CREATE TABLE pages (
               id text primary key, content text not null default '', summary text not null default '',
               created_at integer not null, updated_at integer not null, deleted_at integer,
               version integer not null default 1, title text not null default '',
               kind text not null default 'note', tags text not null default '[]',
               projects text not null default '[]', from_view text, nodes_json text);
             INSERT INTO tags (id, name, extinguished) VALUES ('t1', '工作', 0);
             INSERT INTO tags (id, name, extinguished) VALUES ('t2', '工作', 0);
             INSERT INTO tags (id, name, extinguished, deleted_at) VALUES ('t3', '工作', 1, 1);
             INSERT INTO tags (id, name, extinguished) VALUES ('t4', '工作 (2)', 0);
             INSERT INTO card_tags (card_id, tag_id) VALUES ('ghost', 't1');",
        )
        .unwrap();
        let d = Database { conn: Mutex::new(conn) };
        d.migrate().unwrap(); // 不应崩溃

        // 活标签无重名（t1 保留「工作」，t2 改名为「工作 (3)」，t4 保留「工作 (2)」）。
        let tags = d.list_tags().unwrap();
        let mut active: Vec<String> = tags
            .iter()
            .filter(|t| !t.extinguished)
            .map(|t| t.name.clone())
            .collect();
        active.sort();
        assert!(active.windows(2).all(|w| w[0] != w[1]));
        assert!(active.contains(&"工作".to_string()));
        assert!(active.contains(&"工作 (2)".to_string()));
        assert!(active.contains(&"工作 (3)".to_string()));

        // 孤儿关联被清理。
        let orphans: i64 = d
            .conn
            .lock()
            .unwrap()
            .query_row(
                "SELECT COUNT(*) FROM card_tags WHERE card_id NOT IN (SELECT id FROM pages)",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(orphans, 0);

        // 唯一索引已生效：再插同名活标签会失败。
        let r = d.conn.lock().unwrap().execute(
            "INSERT INTO tags (id, name, extinguished) VALUES ('dup', '工作', 0)",
            [],
        );
        assert!(r.is_err());
    }

    // ---------- 导入 ----------

    fn text(id: &str, content: &str, tags: &[&str], created_at: i64) -> ImportedText {
        ImportedText {
            text_id: id.to_string(),
            title: String::new(),
            content: content.to_string(),
            tags: tags.iter().map(|s| s.to_string()).collect(),
            created_at,
            updated_at: created_at,
        }
    }

    fn tag_names_of(db: &Database, page_id: &str) -> Vec<String> {
        let page = db.get(page_id).unwrap().unwrap();
        let all = db.list_tags().unwrap();
        page.tags
            .iter()
            .map(|id| {
                all.iter()
                    .find(|t| &t.id == id)
                    .map(|t| t.name.clone())
                    .unwrap_or_else(|| format!("<{id}?>"))
            })
            .collect()
    }

    #[test]
    fn import_texts_creates_normal_cards_and_ledger() {
        let db = db();
        let rows = vec![
            text("catcher:rec_1", "第一条", &["待作"], 1_760_000_000_000),
            text("catcher:rec_2", "第二条", &[], 1_760_000_001_000),
        ];
        let out = db.import_texts(&rows).unwrap();
        assert_eq!(out.imported, 2);
        assert_eq!(out.skipped_existing, 0);
        assert_eq!(db.imported_text_count().unwrap(), 2);

        // 是普通便签，不是视图。
        let stats = db.stats().unwrap();
        assert_eq!(stats.active, 2);
        assert_eq!(stats.outline, 0);

        let list = db.list(View::Active, None, "", &[], &[]).unwrap();
        let ids: Vec<&str> = list.items.iter().map(|m| m.id.as_str()).collect();
        let one = ids.iter().find(|id| tag_names_of(&db, id).contains(&"待作".to_string()));
        assert!(one.is_some(), "导入卡片应带上游标签");
        // 每张卡片都自动挂「手机」。
        for id in &ids {
            assert!(tag_names_of(&db, id).contains(&"手机".to_string()));
        }
    }

    #[test]
    fn import_texts_is_idempotent_by_text_id() {
        let db = db();
        let rows = vec![text("catcher:rec_1", "只此一条", &[], 1_760_000_000_000)];
        assert_eq!(db.import_texts(&rows).unwrap().imported, 1);
        // 同一份数据再导一次：一条都不进（认 text_id，不认内容）。
        let again = db.import_texts(&rows).unwrap();
        assert_eq!(again.imported, 0);
        assert_eq!(again.skipped_existing, 1);
        assert_eq!(db.stats().unwrap().active, 1);
        assert_eq!(db.imported_text_count().unwrap(), 1);
    }

    #[test]
    fn import_texts_skips_empty_content_without_ledger_row() {
        let db = db();
        let rows = vec![
            text("catcher:empty", "   \n ", &[], 1_760_000_000_000),
            text("catcher:full", "有内容", &[], 1_760_000_001_000),
        ];
        let out = db.import_texts(&rows).unwrap();
        assert_eq!(out.imported, 1);
        assert_eq!(out.skipped_empty, 1);
        // 空正文不记账 —— 上游补齐正文后它仍算「第一次拉」。
        assert_eq!(db.imported_text_count().unwrap(), 1);
    }

    #[test]
    fn import_texts_mirrors_tag_names_merging_duplicates() {
        let db = db();
        // 已有一枚活标签「洞察」——导入必须复用它，而不是新建同名的。
        let existing = db.create_tag("洞察", "").unwrap();
        let rows = vec![
            text("catcher:rec_1", "甲", &["洞察", "待作"], 1_760_000_000_000),
            text("catcher:rec_2", "乙", &["洞察"], 1_760_000_001_000),
        ];
        assert_eq!(db.import_texts(&rows).unwrap().imported, 2);

        let tags = db.list_tags().unwrap();
        // 「洞察」只有一枚（重名即合并）；名字集合 = 洞察 / 待作 / 手机。
        let mut names: Vec<String> = tags.iter().map(|t| t.name.clone()).collect();
        names.sort();
        assert_eq!(names, vec!["待作", "手机", "洞察"]); // 按码点排，不是拼音

        // 两张卡都指向同一枚「洞察」ID。
        let list = db.list(View::Active, None, "", &[], &[]).unwrap();
        for m in &list.items {
            assert!(m.tags.iter().any(|t| t == &existing.id));
        }
        assert_eq!(
            tags.iter().find(|t| t.name == "洞察").unwrap().card_count,
            2
        );
    }

    #[test]
    fn import_texts_survives_card_purge() {
        let db = db();
        let rows = vec![text("catcher:rec_1", "删了也不重来", &[], 1_760_000_000_000)];
        db.import_texts(&rows).unwrap();
        let id = db.list(View::Active, None, "", &[], &[]).unwrap().items[0].id.clone();

        // 用户在 Note 里把这张卡彻底删掉。
        db.soft_delete(&id).unwrap().unwrap();
        db.purge(&id).unwrap().unwrap();
        assert_eq!(db.stats().unwrap().active, 0);

        // 账本还在 → 下一轮不会再把它拉回来（「本地删除只删本地」）。
        let again = db.import_texts(&rows).unwrap();
        assert_eq!(again.imported, 0);
        assert_eq!(again.skipped_existing, 1);
        assert_eq!(db.stats().unwrap().active, 0);
    }

    #[test]
    fn import_texts_keeps_source_time_order() {
        let db = db();
        // 上游给你的时候顺序是乱的：按 src_created_at 排，不是导入时刻。
        let rows = vec![
            text("catcher:c", "丙", &[], 1_760_000_200_000),
            text("catcher:a", "甲", &[], 1_760_000_000_000),
            text("catcher:b", "乙", &[], 1_760_000_100_000),
        ];
        db.import_texts(&rows).unwrap();
        let list = db.list(View::Active, None, "", &[], &[]).unwrap();
        let summaries: Vec<&str> = list.items.iter().map(|m| m.summary.as_str()).collect();
        assert_eq!(summaries, vec!["丙", "乙", "甲"]); // created_at 降序
    }
}
