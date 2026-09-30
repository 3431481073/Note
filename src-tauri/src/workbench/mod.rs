// 工作台（主窗口）初始化。
pub mod action;

use tauri::{App, Manager};

pub fn setup(app: &App) {
    // 将工作台工具条停靠到主屏右缘、垂直居中。
    let Some(win) = app.get_webview_window("main") else {
        return;
    };
    let Ok(Some(monitor)) = app.primary_monitor() else {
        return;
    };
    let area = monitor.work_area();
    let Ok(size) = win.outer_size() else {
        return;
    };
    let x = area.position.x + area.size.width.saturating_sub(size.width) as i32;
    let y = area.position.y + (area.size.height.saturating_sub(size.height) / 2) as i32;
    let _ = win.set_position(tauri::PhysicalPosition::new(x, y));
}
