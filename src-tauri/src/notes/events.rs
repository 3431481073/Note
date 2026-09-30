// 事件广播：所有事件由 Rust 发出，前端只监听。
use serde::Serialize;
use tauri::{AppHandle, Emitter};

use super::models::{Stats, View};

#[derive(Serialize, Clone)]
pub struct ChangedPayload {
    pub op: &'static str,
    pub id: String,
    pub version: i64,
    pub view: View,
    pub created_at: Option<i64>,
    pub deleted_at: Option<i64>,
    pub summary: Option<String>,
    pub title: Option<String>,
    pub kind: Option<String>,
    pub tags: Option<Vec<String>>,
    pub projects: Option<Vec<String>>,
    pub ts: i64,
}

pub fn emit_changed(app: &AppHandle, p: ChangedPayload) {
    let _ = app.emit("notes://changed", p);
}

pub fn emit_stats(app: &AppHandle, stats: Stats) {
    let _ = app.emit("notes://stats", stats);
}
