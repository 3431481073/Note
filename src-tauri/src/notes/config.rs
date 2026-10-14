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

/// `sources.json`：本项目**从哪些外部池子拿数据**。
///
/// 路径写在这里而不是代码里 —— 换机器、挪目录只改配置，不用重编译。
#[derive(Deserialize, Serialize, Clone)]
pub struct SourcesConfig {
    pub version: i32,
    #[serde(default)]
    pub catcher: Option<CatcherSource>,
}

/// 手机 catcher 的中枢库（`_data` 的 `index.db`）。
#[derive(Deserialize, Serialize, Clone)]
pub struct CatcherSource {
    pub index_db: String,
}

/// `read_sources` 的结论。**「没得导」不是错误，但要能说出为什么。**
///
/// 三种"没配"以前都压成 `None`，等于什么都不说 —— 真机上表现为「静默跳过」。
/// 现在每种都带一句人话，进 `catcher_import_status` 与 `trace.log`。
pub enum SourceConfig {
    Ready(PathBuf),
    /// 配不了，附原因。
    Unconfigured(String),
}

/// `config_dir()` 会往 appConfigDir 里放文件。debug 是覆盖复制、release 是补缺复制，
/// 两边都可能有后台线程（导入）和前端同时进来 —— 同时复制同一批文件可能落下半截内容
/// （`seasons.json` 922 B，坏了就是主题失效）。**只锁复制，不锁读取**，改动最小。
static COPY_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// 统一配置目录：appConfigDir()/config。
pub fn config_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let config = app
        .path()
        .app_config_dir()
        .map_err(|e| e.to_string())?
        .join("config");

    let _guard = COPY_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    #[cfg(debug_assertions)]
    {
        // 开发模式：从项目根复制到 appConfigDir（覆盖 —— 改完立刻生效）
        let src = std::env::current_dir()
            .map_err(|e| e.to_string())?
            .join("config");
        if src.exists() {
            copy_dir(&src, &config).map_err(|e| e.to_string())?;
        }
    }

    #[cfg(not(debug_assertions))]
    {
        // 生产模式：从内置 resource **补缺**（不覆盖）。
        //
        // 🔴 以前这里只有 `if !config.exists()` 才复制 —— 于是**目录一旦存在，
        // 后来新增的默认配置就永远到不了真机**（`sources.json` 就是第一个受害者，
        // 实测：项目 `config\` 有、`%APPDATA%\...\config\` 没有 ⇒ 导入静默跳过）。
        // 现在每次都补缺：已存在的文件一律不动（运行时那份是用户的，人改过的不许盖回去）。
        std::fs::create_dir_all(&config).map_err(|e| e.to_string())?;
        copy_default_config(app, &config)?;
    }

    Ok(config)
}

/// 递归复制**缺失的**文件与目录；目标已存在的一律不动。
fn copy_dir_missing(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_p = entry.path();
        let dst_p = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_missing(&src_p, &dst_p)?;
        } else if !dst_p.exists() {
            std::fs::copy(&src_p, &dst_p)?;
        }
    }
    Ok(())
}

pub fn read_seasons(app: &AppHandle) -> Result<SeasonsConfig, String> {
    let dir = config_dir(app)?;
    let path = dir.join("seasons.json");
    let content =
        std::fs::read_to_string(&path).map_err(|e| format!("read seasons.json: {e}"))?;
    serde_json::from_str(&content).map_err(|e| format!("parse seasons.json: {e}"))
}

/// 读 `sources.json` 里配的 catcher 中枢库路径。
///
/// 「没得导」有三种，**每一种都带回一句具体的原因**（不是笼统的 `None`）：
/// 文件不在、文件里没有 catcher 段、路径是空的。只有读不动/解不开这个文件本身才是 `Err`。
pub fn read_sources(app: &AppHandle) -> Result<SourceConfig, String> {
    let dir = config_dir(app)?;
    let path = dir.join("sources.json");
    if !path.exists() {
        return Ok(SourceConfig::Unconfigured(format!(
            "配置文件不存在：{}",
            path.display()
        )));
    }
    let content =
        std::fs::read_to_string(&path).map_err(|e| format!("read sources.json: {e}"))?;
    let cfg: SourcesConfig =
        serde_json::from_str(&content).map_err(|e| format!("parse sources.json: {e}"))?;
    match cfg.catcher {
        Some(c) if !c.index_db.trim().is_empty() => {
            Ok(SourceConfig::Ready(PathBuf::from(c.index_db.trim())))
        }
        Some(_) => Ok(SourceConfig::Unconfigured(format!(
            "{} 里 catcher.index_db 是空的",
            path.display()
        ))),
        None => Ok(SourceConfig::Unconfigured(format!(
            "{} 里没有 catcher 段",
            path.display()
        ))),
    }
}

/// 覆盖复制。**只有 debug 用它** —— release 走 [`copy_dir_missing`]（不许盖用户改过的）。
#[cfg(debug_assertions)]
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
    let res = app
        .path()
        .resolve("config", tauri::path::BaseDirectory::Resource)
        .map_err(|e| e.to_string())?;
    if res.exists() {
        return copy_dir_missing(&res, dst).map_err(|e| e.to_string());
    }
    // 打包里没有默认配置 ⇒ 新装机器上 config/ 会是空的（历史遗留，见 docs/data.md 第六节）。
    // **别静默** —— 记一笔，下次有人问"为什么真机上没配到"时有据可查。
    super::trace::log("config: bundled defaults missing (resource/config not found)");
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

#[cfg(test)]
mod tests {
    use super::*;

    /// 补缺复制必须**只补缺、不覆盖** —— 运行时那份是用户的，人改过的不能被默认值盖回去。
    /// 这正是不再做 `if !config.exists()` 的前提：新增的默认文件要能到，
    /// 但已有的一律不动。
    #[test]
    fn copy_dir_missing_fills_gaps_but_never_overwrites() {
        let root = std::env::temp_dir().join(format!("note-cfgtest-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let src = root.join("src");
        let dst = root.join("dst");
        std::fs::create_dir_all(src.join("assets")).unwrap();
        std::fs::write(src.join("seasons.json"), "default").unwrap();
        std::fs::write(src.join("sources.json"), "the new one").unwrap();
        std::fs::write(src.join("assets/a.svg"), "svg").unwrap();

        // 目标目录已存在，且 seasons.json 是用户改过的。
        std::fs::create_dir_all(&dst).unwrap();
        std::fs::write(dst.join("seasons.json"), "user-edited").unwrap();

        copy_dir_missing(&src, &dst).unwrap();

        assert_eq!(
            std::fs::read_to_string(dst.join("seasons.json")).unwrap(),
            "user-edited",
            "已存在的文件被覆盖了"
        );
        assert_eq!(
            std::fs::read_to_string(dst.join("sources.json")).unwrap(),
            "the new one",
            "缺的新文件没补上"
        );
        assert_eq!(std::fs::read_to_string(dst.join("assets/a.svg")).unwrap(), "svg");

        // 再跑一次是幂等的。
        copy_dir_missing(&src, &dst).unwrap();
        assert_eq!(
            std::fs::read_to_string(dst.join("seasons.json")).unwrap(),
            "user-edited"
        );

        let _ = std::fs::remove_dir_all(&root);
    }

    /// **送货这一环的实测**：拿 `tauri build` 真正落在 `target/release/config/` 的
    /// 打包默认值，补进一个"已装机、只有旧文件"的目标目录 —— 断言新文件真的到得了。
    ///
    /// 这正是真机上失效的那一步：`%APPDATA%\...\config\` 早就存在，旧逻辑
    /// （`if !config.exists()`）于是永不复制。本机没构建过就跳过。
    #[test]
    fn bundled_defaults_fill_an_existing_install() {
        let res = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/release/config");
        if !res.exists() {
            return; // 没跑过 `tauri build`，无从取打包默认值
        }
        let dst = std::env::temp_dir().join(format!("note-install-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dst);
        std::fs::create_dir_all(&dst).unwrap();
        // 造出真机的形状：目录在、旧文件在、新文件不在。
        std::fs::write(dst.join("seasons.json"), "user-edited").unwrap();

        copy_dir_missing(&res, &dst).unwrap();

        assert!(
            dst.join("sources.json").exists(),
            "打包默认值里的 sources.json 没补进已装机的配置目录 —— 真机上就是这一步失效"
        );
        assert_eq!(
            std::fs::read_to_string(dst.join("seasons.json")).unwrap(),
            "user-edited",
            "已存在（人改过）的文件被覆盖了"
        );

        let _ = std::fs::remove_dir_all(&dst);
    }
}
