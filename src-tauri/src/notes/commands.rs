// Tauri 命令：数据操作 + 事件广播。
use tauri::{AppHandle, State};

use super::config::{self, SeasonsConfig};
use super::db::Database;
use super::events::{self, ChangedPayload};
use super::models::{Cursor, MetaIndex, Page, PageList, PageMeta, Stats, Tag, UpdateResult, View};
use super::summary::{default_project_dir, now_ms};

#[tauri::command]
pub fn get_config_dir(app: AppHandle) -> Result<String, String> {
    config::config_dir(&app).map(|p| p.to_string_lossy().to_string())
}

#[tauri::command]
pub fn read_seasons_config(app: AppHandle) -> Result<SeasonsConfig, String> {
    config::read_seasons(&app)
}

#[tauri::command]
pub fn get_stats(db: State<'_, Database>) -> Result<Stats, String> {
    db.stats().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_meta_index(db: State<'_, Database>) -> Result<MetaIndex, String> {
    db.meta_index().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_tag(db: State<'_, Database>, id: String) -> Result<(), String> {
    db.delete_tag(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_tags(db: State<'_, Database>) -> Result<Vec<Tag>, String> {
    db.list_tags().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_tag(
    db: State<'_, Database>,
    name: String,
    description: Option<String>,
) -> Result<Tag, String> {
    db.create_tag(&name, description.unwrap_or_default().as_str())
}

#[tauri::command]
pub fn rename_tag(db: State<'_, Database>, id: String, name: String) -> Result<(), String> {
    db.rename_tag(&id, &name)
}

#[tauri::command]
pub fn set_tag_description(
    db: State<'_, Database>,
    id: String,
    description: String,
) -> Result<(), String> {
    db.set_tag_description(&id, &description)
}

#[tauri::command]
pub fn restore_tag(db: State<'_, Database>, id: String) -> Result<(), String> {
    db.restore_tag(&id)
}

#[tauri::command]
pub fn merge_tags(
    db: State<'_, Database>,
    from_ids: Vec<String>,
    to_name: String,
    to_description: Option<String>,
) -> Result<Tag, String> {
    db.merge_tags(&from_ids, &to_name, to_description.unwrap_or_default().as_str())
}

#[tauri::command]
pub async fn pick_project_dir(app: AppHandle) -> Result<Option<String>, String> {
    // 阻塞式文件选择器必须在非主线程调用，否则会卡死整个界面。
    let result = tauri::async_runtime::spawn_blocking(move || {
        use tauri_plugin_dialog::DialogExt;
        app.dialog().file().blocking_pick_folder()
    })
    .await
    .map_err(|e| e.to_string())?;
    Ok(result.and_then(|f| f.as_path().map(|p| p.to_string_lossy().to_string())))
}

#[tauri::command]
pub fn list_outlines(db: State<'_, Database>, view: View) -> Result<Vec<PageMeta>, String> {
    db.list_outlines(view).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_outline(db: State<'_, Database>, id: String, nodes_json: String) -> Result<(), String> {
    db.save_outline(&id, &nodes_json)
        .map_err(|e| e.to_string())
        .and_then(|ok| if ok { Ok(()) } else { Err("outline not found".to_string()) })
}

#[tauri::command]
pub fn set_hit_rects(rects: Vec<super::hittest::HitRect>) -> Result<(), String> {
    super::hittest::set_rects(rects);
    Ok(())
}

#[tauri::command]
pub fn list_project_folders() -> Result<Vec<String>, String> {
    let root = default_project_dir().unwrap_or_default();
    let mut dirs = Vec::new();
    if !root.is_empty() {
        if let Ok(entries) = std::fs::read_dir(&root) {
            for e in entries.flatten() {
                let p = e.path();
                if p.is_dir() {
                    dirs.push(p.to_string_lossy().to_string());
                }
            }
        }
    }
    dirs.sort();
    Ok(dirs)
}

#[tauri::command]
pub fn create_page(
    db: State<'_, Database>,
    app: AppHandle,
    kind: Option<String>,
) -> Result<Page, String> {
    let k = kind.unwrap_or_else(|| "note".to_string());
    let page = db.create(&k).map_err(|e| e.to_string())?;
    events::emit_changed(
        &app,
        ChangedPayload {
            op: "create",
            id: page.id.clone(),
            version: page.version,
            view: View::Active,
            created_at: Some(page.created_at),
            deleted_at: None,
            summary: Some(page.summary.clone()),
            title: Some(page.title.clone()),
            kind: Some(page.kind.clone()),
            tags: Some(page.tags.clone()),
            projects: Some(page.projects.clone()),
            ts: now_ms(),
        },
    );
    emit_stats(&db, &app)?;
    Ok(page)
}

#[tauri::command]
pub fn list_pages(
    db: State<'_, Database>,
    view: View,
    cursor: Option<Cursor>,
) -> Result<PageList, String> {
    db.list(view, cursor.as_ref(), "", &[], &[])
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn search(
    db: State<'_, Database>,
    query: String,
    view: View,
    cursor: Option<Cursor>,
    tags: Option<Vec<String>>,
    projects: Option<Vec<String>>,
) -> Result<PageList, String> {
    db.list(
        view,
        cursor.as_ref(),
        &query,
        tags.as_deref().unwrap_or(&[]),
        projects.as_deref().unwrap_or(&[]),
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_page(db: State<'_, Database>, id: String) -> Result<Option<Page>, String> {
    db.get(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_page(
    db: State<'_, Database>,
    app: AppHandle,
    id: String,
    content: String,
    title: Option<String>,
    tags: Option<Vec<String>>,
    projects: Option<Vec<String>>,
    from_view: Option<String>,
) -> Result<UpdateResult, String> {
    let res = db
        .update(
            &id,
            &content,
            title.as_deref(),
            tags.as_deref(),
            projects.as_deref(),
            from_view.as_deref(),
        )
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "page not found".to_string())?;
    events::emit_changed(
        &app,
        ChangedPayload {
            op: "update",
            id: res.id.clone(),
            version: res.version,
            view: View::Active,
            created_at: None,
            deleted_at: None,
            summary: Some(res.summary.clone()),
            title: None,
            kind: None,
            tags: None,
            projects: None,
            ts: now_ms(),
        },
    );
    Ok(res)
}

#[tauri::command]
pub fn soft_delete(db: State<'_, Database>, app: AppHandle, id: String) -> Result<Stats, String> {
    if let Some((version, deleted_at)) = db.soft_delete(&id).map_err(|e| e.to_string())? {
        events::emit_changed(
            &app,
            ChangedPayload {
                op: "delete",
                id: id.clone(),
                version,
                view: View::Trash,
                created_at: None,
                deleted_at: Some(deleted_at),
                summary: None,
                title: None,
                kind: None,
                tags: None,
                projects: None,
                ts: now_ms(),
            },
        );
    }
    emit_stats(&db, &app)
}

#[tauri::command]
pub fn restore_page(db: State<'_, Database>, app: AppHandle, id: String) -> Result<Page, String> {
    let page = db
        .restore(&id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "page not in trash".to_string())?;
    events::emit_changed(
        &app,
        ChangedPayload {
            op: "restore",
            id: page.id.clone(),
            version: page.version,
            view: View::Active,
            created_at: Some(page.created_at),
            deleted_at: None,
            summary: Some(page.summary.clone()),
            title: Some(page.title.clone()),
            kind: Some(page.kind.clone()),
            tags: Some(page.tags.clone()),
            projects: Some(page.projects.clone()),
            ts: now_ms(),
        },
    );
    emit_stats(&db, &app)?;
    Ok(page)
}

#[tauri::command]
pub fn purge_page(db: State<'_, Database>, app: AppHandle, id: String) -> Result<Stats, String> {
    if let Some(version) = db.purge(&id).map_err(|e| e.to_string())? {
        events::emit_changed(
            &app,
            ChangedPayload {
                op: "purge",
                id: id.clone(),
                version,
                view: View::Trash,
                created_at: None,
                deleted_at: None,
                summary: None,
                title: None,
                kind: None,
                tags: None,
                projects: None,
                ts: now_ms(),
            },
        );
    }
    emit_stats(&db, &app)
}

fn emit_stats(db: &Database, app: &AppHandle) -> Result<Stats, String> {
    let stats = db.stats().map_err(|e| e.to_string())?;
    events::emit_stats(app, stats);
    Ok(stats)
}
