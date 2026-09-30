// 配置加载与热重载：seasons.json 统一位于 appConfigDir()/config/。
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SeasonsConfig {
    pub version: i32,
    pub default_opacity: f64,
    pub default_fit: String,
    pub default_align: String,
    pub months: BTreeMap<String, MonthConfig>,
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MonthConfig {
    pub src: String,
    pub ink: String,
    #[serde(default)]
    pub opacity: Option<f64>,
    #[serde(default)]
    pub fit: Option<String>,
    #[serde(default)]
    pub align: Option<String>,
}

/// 统一配置目录：appConfigDir()/config。
pub fn config_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let config = app
        .path()
        .app_config_dir()
        .map_err(|e| e.to_string())?
        .join("config");

    #[cfg(debug_assertions)]
    {
        // 开发模式：从项目根复制到 appConfigDir
        let src = std::env::current_dir()
            .map_err(|e| e.to_string())?
            .join("config");
        if src.exists() {
            copy_dir(&src, &config).map_err(|e| e.to_string())?;
        }
    }

    #[cfg(not(debug_assertions))]
    {
        // 生产模式：首次启动从内置 resource 复制
        if !config.exists() {
            std::fs::create_dir_all(&config).map_err(|e| e.to_string())?;
            copy_default_config(app, &config).map_err(|e| e.to_string())?;
        }
    }

    Ok(config)
}

pub fn read_seasons(app: &AppHandle) -> Result<SeasonsConfig, String> {
    let dir = config_dir(app)?;
    let path = dir.join("seasons.json");
    let content =
        std::fs::read_to_string(&path).map_err(|e| format!("read seasons.json: {e}"))?;
    serde_json::from_str(&content).map_err(|e| format!("parse seasons.json: {e}"))
}

fn copy_dir(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_p = entry.path();
        let dst_p = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir(&src_p, &dst_p)?;
        } else {
            std::fs::copy(&src_p, &dst_p)?;
        }
    }
    Ok(())
}

#[cfg(not(debug_assertions))]
fn copy_default_config(app: &AppHandle, dst: &Path) -> Result<(), String> {
    if let Ok(res) = app.path().resolve("config", tauri::path::BaseDirectory::Resource) {
        if res.exists() {
            return copy_dir(&res, dst).map_err(|e| e.to_string());
        }
    }
    Ok(())
}

/// 启动配置热重载线程：监听 seasons.json 变化，广播 config-changed。
pub fn spawn_config_watcher(app: AppHandle) {
    std::thread::spawn(move || {
        let dir = watch_source_dir(&app);
        let Some(dir) = dir else { return };

        use notify::{RecommendedWatcher, RecursiveMode, Watcher};
        let (tx, rx) = std::sync::mpsc::channel();
        let mut watcher: RecommendedWatcher = match notify::recommended_watcher(move |res| {
            let _ = tx.send(res);
        }) {
            Ok(w) => w,
            Err(_) => return,
        };
        if watcher.watch(&dir, RecursiveMode::NonRecursive).is_err() {
            return;
        }
        for res in rx {
            if let Ok(event) = res {
                let relevant = event
                    .paths
                    .iter()
                    .any(|p| p.file_name().and_then(|n| n.to_str()) == Some("seasons.json"));
                if relevant {
                    let _ = app.emit("notes://config-changed", ());
                }
            }
        }
    });
}

#[cfg(debug_assertions)]
fn watch_source_dir(_app: &AppHandle) -> Option<PathBuf> {
    std::env::current_dir().ok().map(|d| d.join("config"))
}

#[cfg(not(debug_assertions))]
fn watch_source_dir(app: &AppHandle) -> Option<PathBuf> {
    config_dir(app).ok()
}
