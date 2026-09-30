// 便签窗口创建与显示控制。
use tauri::{AppHandle, PhysicalPosition, WebviewUrl, WebviewWindowBuilder};

pub fn create_notes_window(app: &AppHandle) -> tauri::Result<()> {
    let win = WebviewWindowBuilder::new(app, "notes", WebviewUrl::App("notes.html".into()))
        .title("便签")
        .inner_size(1300.0, 648.0)
        .decorations(false)
        .transparent(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .shadow(false)
        .visible(false)
        .build()?;

    // 居中到工作区（排除任务栏），与 workbench 的垂直居中逻辑一致，避免窗口整体偏下。
    if let Ok(Some(monitor)) = app.primary_monitor() {
        let area = monitor.work_area();
        if let Ok(size) = win.outer_size() {
            // 允许负 x：窗口比屏宽时居中到屏外一点点，右侧内容/叉叉才能完整落在屏内（避免被屏右缘裁掉）
            let x = area.position.x + (area.size.width as i32 - size.width as i32) / 2;
            let y = area.position.y + (area.size.height as i32 - size.height as i32) / 2;
            let _ = win.set_position(PhysicalPosition::new(x, y));
        }
    }

    // 固定 1300 宽不再 resize：安装 WM_NCHITTEST 子类化，透明区点击穿透、面板内正常响应。
    #[cfg(target_os = "windows")]
    if let Ok(hwnd) = win.hwnd() {
        super::hittest::install(hwnd.0);
    }

    Ok(())
}
