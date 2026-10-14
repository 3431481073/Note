//! 上游文本 → 便签卡片：**后台拉，不在启动路径上**。
//!
//! 唯一的来源是 `config/sources.json` 里配置的那个外部 SQLite 库（路径不写在代码里），
//! 目前就是 `_data` 的 `texts` 表。窗口照常先出来；这个线程在背景里读、写、然后广播一个
//! 事件让列表刷新。
//!
//! 三条底线（照着需求逐条落的）：
//! - **窗口起不来也不能是因为它** —— 读源失败一律只是「这一轮不导」，绝不冒泡到启动路径。
//! - **上游库可能在写** —— 那个库是 `journal_mode=delete`，计划任务每 30 分钟写一次；
//!   撞上就是 `database is locked`，**隔一会儿重试，不当库坏了**。
//! - **写的是用户的数据** —— 每张卡都进用户的标签体系，所以去重（账本 `imported_texts`）
//!   必须是硬的：认 `text_id`，一次就是一次。
use std::path::Path;
use std::sync::Mutex;
use std::time::Duration;

use rusqlite::{Connection, OpenFlags};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use super::config;
use super::db::{Database, ImportedText};
use super::summary::now_ms;
use super::trace;

/// 自动挂到每张导入卡片上的标签（用户一眼能认出「这是手机来的」）。
pub const MOBILE_TAG: &str = "手机";

/// 上游库被写住时的重试次数与间隔。
const RETRY_ATTEMPTS: usize = 4;
const RETRY_DELAY: Duration = Duration::from_millis(1500);
/// 交给 SQLite 自己的等锁时间（在重试之前先让库自己等一等）。
const BUSY_TIMEOUT: Duration = Duration::from_secs(2);

/// 这一轮导入走到哪了。前端 init 收尾时对一次账（事件可能已经发过、没人听见）。
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ImportState {
    /// `pending` / `running` / `done` / `failed` / `skipped`
    pub state: String,
    pub imported: i64,
    pub skipped: i64,
    pub message: Option<String>,
}

impl Default for ImportState {
    fn default() -> Self {
        Self {
            state: "pending".to_string(),
            imported: 0,
            skipped: 0,
            message: None,
        }
    }
}

/// 托管状态：读它不会碰数据库，所以前端随时可以问。
pub struct ImportStatus(pub Mutex<ImportState>);

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ImportedEvent {
    imported: i64,
    skipped: i64,
}

/// 起一个后台线程去拉。**立刻返回** —— 调用点在启动路径上，这里不做任何等待。
pub fn spawn(app: AppHandle) {
    let _ = app.manage(ImportStatus(Mutex::new(ImportState::default())));
    std::thread::spawn(move || run(&app));
}

/// 当前状态（给前端的 `catcher_import_status` 命令）。进程里没有托管状态时返回 `pending`。
pub fn status(app: &AppHandle) -> ImportState {
    app.try_state::<ImportStatus>()
        .map(|s| s.0.lock().unwrap_or_else(|e| e.into_inner()).clone())
        .unwrap_or_default()
}

fn set_state(app: &AppHandle, f: impl FnOnce(&mut ImportState)) {
    if let Some(st) = app.try_state::<ImportStatus>() {
        let mut guard = st.0.lock().unwrap_or_else(|e| e.into_inner());
        f(&mut guard);
    }
}

fn run(app: &AppHandle) {
    set_state(app, |s| s.state = "running".to_string());
    trace::log("catcher_import: begin");

    let source = match config::read_sources(app) {
        Ok(config::SourceConfig::Ready(p)) => p,
        Ok(config::SourceConfig::Unconfigured(why)) => {
            // 「读不到源」必须说得出来 —— 状态里带原因，trace 里也留一笔，不静默跳过。
            set_state(app, |s| {
                s.state = "skipped".to_string();
                s.message = Some(why.clone());
            });
            trace::log(&format!("catcher_import: unconfigured: {why}"));
            return;
        }
        Err(e) => {
            set_state(app, |s| {
                s.state = "failed".to_string();
                s.message = Some(e.clone());
            });
            trace::log(&format!("catcher_import: config error: {e}"));
            return;
        }
    };

    if !source.exists() {
        let why = format!("源库不存在：{}", source.display());
        set_state(app, |s| {
            s.state = "failed".to_string();
            s.message = Some(why.clone());
        });
        trace::log(&format!("catcher_import: {why}"));
        return;
    }

    let rows = match read_new_texts(&source) {
        Ok(rows) => rows,
        Err(e) => {
            // 读不动上游 = 这一轮不导。便签本身照常起。
            set_state(app, |s| {
                s.state = "failed".to_string();
                s.message = Some(e.clone());
            });
            trace::log(&format!("catcher_import: source read failed: {e}"));
            return;
        }
    };

    let db = app.state::<Database>();
    match db.import_texts(&rows) {
        Ok(out) => {
            trace::log(&format!(
                "catcher_import: done imported={} existing={} empty={}",
                out.imported, out.skipped_existing, out.skipped_empty
            ));
            set_state(app, |s| {
                s.state = "done".to_string();
                s.imported = out.imported;
                s.skipped = out.skipped_existing + out.skipped_empty;
                s.message = None;
            });
            if out.imported > 0 {
                let _ = app.emit(
                    "notes://catcher-imported",
                    ImportedEvent {
                        imported: out.imported,
                        skipped: out.skipped_existing + out.skipped_empty,
                    },
                );
            }
        }
        Err(e) => {
            set_state(app, |s| {
                s.state = "failed".to_string();
                s.message = Some(e.clone());
            });
            trace::log(&format!("catcher_import: write failed: {e}"));
        }
    }
}

/// 带着重试地读上游。只有「库被锁住」才重试 —— 别的错误重试也没用。
fn read_new_texts(source: &Path) -> Result<Vec<ImportedText>, String> {
    let mut last = String::new();
    for attempt in 0..RETRY_ATTEMPTS {
        match read_once(source) {
            Ok(rows) => return Ok(rows),
            Err((retryable, msg)) => {
                last = msg;
                if !retryable || attempt + 1 == RETRY_ATTEMPTS {
                    break;
                }
                trace::log(&format!(
                    "catcher_import: source locked, retry {}/{}",
                    attempt + 1,
                    RETRY_ATTEMPTS - 1
                ));
                std::thread::sleep(RETRY_DELAY);
            }
        }
    }
    Err(last)
}

/// 只读打开上游库，取 `source='catcher' AND deleted=0` 的全部文本。
///
/// 返回 `Err((是否值得重试, 说明))`。**只读**：`mode=ro` + `SQLITE_OPEN_READ_ONLY`，
/// 库不存在就是打不开，绝不会把它建出来。
fn read_once(source: &Path) -> Result<Vec<ImportedText>, (bool, String)> {
    let uri = format!("file:{}?mode=ro", source.to_string_lossy().replace('\\', "/"));
    let conn = Connection::open_with_flags(
        &uri,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_URI,
    )
    .map_err(|e| (is_busy(&e), format!("open {uri}: {e}")))?;
    let _ = conn.busy_timeout(BUSY_TIMEOUT);

    let mut stmt = conn
        .prepare(
            "SELECT text_id, title, content, tags, src_created_at, src_updated_at
             FROM texts WHERE source = 'catcher' AND deleted = 0",
        )
        .map_err(|e| (is_busy(&e), format!("prepare texts: {e}")))?;

    let rows = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, Option<String>>(1)?,
                r.get::<_, Option<String>>(2)?,
                r.get::<_, Option<String>>(3)?,
                r.get::<_, Option<String>>(4)?,
                r.get::<_, Option<String>>(5)?,
            ))
        })
        .map_err(|e| (is_busy(&e), format!("query texts: {e}")))?;

    let mut out = Vec::new();
    let mut unparsed = 0u32;
    for r in rows {
        let (text_id, title, content, tags, src_created, src_updated) =
            r.map_err(|e| (is_busy(&e), format!("read texts row: {e}")))?;

        // 时间：拿上游自带的，不是导入时刻。解析不了才退回当前时刻（并记一笔）。
        let created_at = match src_created.as_deref().and_then(parse_iso_utc_ms) {
            Some(ms) => ms,
            None => {
                unparsed += 1;
                now_ms()
            }
        };
        let updated_at = src_updated
            .as_deref()
            .and_then(parse_iso_utc_ms)
            .unwrap_or(created_at);

        out.push(ImportedText {
            text_id,
            title: title.unwrap_or_default(),
            content: content.unwrap_or_default(),
            tags: parse_tag_names(tags.as_deref()),
            created_at,
            updated_at,
        });
    }

    if unparsed > 0 {
        trace::log(&format!(
            "catcher_import: {unparsed} row(s) had unparsable src_created_at, fell back to now"
        ));
    }
    Ok(out)
}

/// `tags` 列是原样的 JSON 字符串数组（元素是**名字**）。坏的/空的都当没有标签。
fn parse_tag_names(raw: Option<&str>) -> Vec<String> {
    raw.and_then(|s| serde_json::from_str::<Vec<String>>(s).ok())
        .unwrap_or_default()
}

/// SQLite 的「库被锁住/正忙」—— 这两种才值得重试。
fn is_busy(e: &rusqlite::Error) -> bool {
    matches!(
        e,
        rusqlite::Error::SqliteFailure(err, _)
            if matches!(
                err.code,
                rusqlite::ErrorCode::DatabaseBusy | rusqlite::ErrorCode::DatabaseLocked
            )
    )
}

/// 解析上游的 `YYYY-MM-DDTHH:MM:SS[.ffffff]Z` 为 epoch 毫秒。
///
/// 只认 UTC（尾随 `Z` 或 `±00:00`）——上游这一列有文档保证就是 UTC 带 `Z`。
/// 形状不对就返回 `None`，由调用方决定怎么退。
pub fn parse_iso_utc_ms(s: &str) -> Option<i64> {
    let b = s.as_bytes();
    if b.len() < 20
        || b[4] != b'-'
        || b[7] != b'-'
        || b[10] != b'T'
        || b[13] != b':'
        || b[16] != b':'
    {
        return None;
    }

    let num = |a: usize, b: usize| -> Option<i64> {
        let part = s.get(a..b)?;
        if !part.bytes().all(|c| c.is_ascii_digit()) {
            return None;
        }
        part.parse().ok()
    };
    let year = num(0, 4)?;
    let month = num(5, 7)?;
    let day = num(8, 10)?;
    let hour = num(11, 13)?;
    let minute = num(14, 16)?;
    let second = num(17, 19)?;
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) || hour > 23 || minute > 59 {
        return None;
    }
    if second > 60 {
        return None; // 60 是闰秒，允许
    }

    let rest = &s[19..];
    let frac_ms = if let Some(stripped) = rest.strip_prefix('.') {
        let digits: String = stripped.chars().take_while(|c| c.is_ascii_digit()).collect();
        if digits.is_empty() {
            return None;
        }
        let mut v: i64 = 0;
        for c in digits.chars().take(3) {
            v = v * 10 + (c as i64 - '0' as i64);
        }
        for _ in digits.len()..3 {
            v *= 10;
        }
        v
    } else {
        0
    };

    // 时区部分：`.740824Z` / `Z` / `+00:00`
    let tz = if rest.starts_with('.') {
        let offset = 1 + rest[1..].chars().take_while(|c| c.is_ascii_digit()).count();
        &rest[offset..]
    } else {
        rest
    };
    if !(tz == "Z" || tz == "+00:00" || tz == "-00:00") {
        return None;
    }

    let days = days_from_civil(year, month, day);
    Some(((days * 86_400 + hour * 3_600 + minute * 60 + second) * 1_000) + frac_ms)
}

/// 公历日期 → 距 1970-01-01 的天数（Howard Hinnant 的 days_from_civil，整数版）。
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400; // [0, 399]
    let mp = (m + 9) % 12; // [0, 11]
    let doy = (153 * mp + 2) / 5 + d - 1; // [0, 365]
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy; // [0, 146096]
    era * 146_097 + doe - 719_468
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_upstream_timestamp_shapes() {
        // `_data.texts.src_created_at` 的实际形状：微秒 + Z。
        // 2026-10-12T09:51:33Z = 1791798693（2026-01-01 是 1767225600，+284 天 +9:51:33）。
        assert_eq!(parse_iso_utc_ms("2026-10-12T09:51:33.740824Z"), Some(1_791_798_693_740));
        // 秒级、无小数。
        assert_eq!(parse_iso_utc_ms("2026-10-12T09:51:33Z"), Some(1_791_798_693_000));
        // 毫秒 3 位。
        assert_eq!(parse_iso_utc_ms("2026-10-12T09:51:33.740Z"), Some(1_791_798_693_740));
        // 1 位小数 → 补齐成百毫秒。
        assert_eq!(parse_iso_utc_ms("2026-10-12T09:51:33.7Z"), Some(1_791_798_693_700));
        // 毫秒以下截断（不四舍五入）。
        assert_eq!(
            parse_iso_utc_ms("2026-10-12T09:51:33.740999Z"),
            Some(1_791_798_693_740)
        );
        // 明写的零偏移也认。
        assert_eq!(
            parse_iso_utc_ms("2026-10-12T09:51:33.000000+00:00"),
            Some(1_791_798_693_000)
        );
        // 纪元原点。
        assert_eq!(parse_iso_utc_ms("1970-01-01T00:00:00Z"), Some(0));
        assert_eq!(parse_iso_utc_ms("1969-12-31T23:59:59Z"), Some(-1_000));
    }

    #[test]
    fn rejects_shapes_it_cannot_trust() {
        assert_eq!(parse_iso_utc_ms(""), None);
        assert_eq!(parse_iso_utc_ms("2026-10-12 09:51:33"), None); // 空格而非 T、无时区
        assert_eq!(parse_iso_utc_ms("2026-10-12T09:51:33+08:00"), None); // 非 UTC 偏移
        assert_eq!(parse_iso_utc_ms("2026-13-12T09:51:33Z"), None); // 月份越界
        assert_eq!(parse_iso_utc_ms("not a timestamp at all"), None);
    }

    #[test]
    fn handles_leap_day_and_month_rollover() {
        // 2024-02-29 存在；2023-02-29 也能算（不崩），只是值无所谓。
        assert_eq!(parse_iso_utc_ms("2024-03-01T00:00:00Z"), Some(1_709_251_200_000));
        assert_eq!(parse_iso_utc_ms("2024-02-29T00:00:00Z"), Some(1_709_164_800_000));
    }

    #[test]
    fn parses_tag_names_leniently() {
        assert_eq!(parse_tag_names(Some(r#"["待作","洞察"]"#)), vec!["待作", "洞察"]);
        assert!(parse_tag_names(Some("")).is_empty());
        assert!(parse_tag_names(None).is_empty());
        assert!(parse_tag_names(Some("not json")).is_empty());
    }

    /// 真·端到端读：拿本机真实的 `_data\index.db`（只读）走一遍 `read_once`。
    ///
    /// 配置里没有源、或那个文件不在，就直接跳过（别的机器上跑得通）。
    /// **不写任何东西** —— 只验证「读得出来、时间解析得对」。
    #[test]
    fn reads_real_source_when_available() {
        let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let cfg_path = manifest.parent().unwrap().join("config").join("sources.json");
        let Ok(text) = std::fs::read_to_string(&cfg_path) else {
            return;
        };
        let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) else {
            return;
        };
        let Some(path) = v
            .get("catcher")
            .and_then(|c| c.get("index_db"))
            .and_then(|p| p.as_str())
        else {
            return;
        };
        let path = std::path::Path::new(path);
        if !path.exists() {
            return;
        }

        let rows = read_new_texts(path).expect("read real source");
        assert!(!rows.is_empty(), "真实源里应当有 catcher 文本");
        for r in &rows {
            assert!(r.text_id.starts_with("catcher:"), "text_id 形状: {}", r.text_id);
            assert!(!r.content.trim().is_empty(), "{} 正文为空", r.text_id);
            // 2026 年内的时刻 —— 证明 src_created_at 被解析、而不是退回了 now。
            assert!(
                r.created_at > 1_750_000_000_000 && r.created_at < 1_800_000_000_000,
                "{} 的时间戳不像 2026 年: {}",
                r.text_id,
                r.created_at
            );
        }
        eprintln!("real source: {} catcher texts read", rows.len());
    }
}
