pub mod commands;
pub mod config;
pub mod db;
pub mod events;
pub mod hittest;
pub mod http;
pub mod import;
pub mod models;
pub mod summary;
pub mod trace;
pub mod window;

use tauri::{App, Manager};

/// 应用启动时的便签初始化：打开数据库、挂载状态、创建便签窗口、启动配置热重载。
pub fn setup(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    trace::init(app.handle());
    trace::log("setup: app started");

    let db_path = app.path().app_data_dir()?.join("notes.db");
    let database = db::Database::open(&db_path)?;
    app.manage(database);

    window::create_notes_window(app.handle())?;
    config::spawn_config_watcher(app.handle().clone());
    http::spawn(app.handle().clone());
    // 手机 catcher 文本 → 卡片：**建后台线程，立刻返回**，绝不挂在启动路径上。
    import::spawn(app.handle().clone());

    // 便签窗口由前端在首屏渲染完成后自行 show()，避免先露出空白 webview 的闪烁。
    Ok(())
}
