// 工作台 → 便签窗口的打开/切换命令。
use tauri::{AppHandle, Manager};

fn window(app: &AppHandle) -> Option<tauri::WebviewWindow> {
    app.get_webview_window("notes")
}

#[tauri::command]
pub fn open_notes(app: AppHandle) -> Result<(), String> {
    let win = window(&app).ok_or("notes window not found")?;
    win.show().map_err(|e| e.to_string())?;
    win.set_focus().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn toggle_notes(app: AppHandle) -> Result<(), String> {
    let win = window(&app).ok_or("notes window not found")?;
    if win.is_visible().unwrap_or(false) {
        win.hide().map_err(|e| e.to_string())?;
    } else {
        win.show().map_err(|e| e.to_string())?;
        win.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}
