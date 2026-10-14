// 便签窗口创建与显示控制。
use tauri::webview::PageLoadEvent;
use tauri::{AppHandle, PhysicalPosition, WebviewUrl, WebviewWindowBuilder};

pub fn create_notes_window(app: &AppHandle) -> tauri::Result<()> {
    let win = WebviewWindowBuilder::new(app, "notes", WebviewUrl::App("notes.html".into()))
        .title("便签")
        .inner_size(1300.0, 648.0)
        .decorations(false)
        .transparent(true)
        // 不置顶：用户要求「点别的窗口就把便签盖住」。叫回便签靠右缘那根工作台条，
        // 它自身仍然置顶，所以入口不会被盖掉。
        .skip_taskbar(true)
        .resizable(false)
        .shadow(false)
        .visible(false)
        // 页面一开始导航就隐藏 + 解除命中武装：加载中/加载失败时窗口绝不可见。
        // 要害：**绝不能用 Finished 去显示窗口**——wry 在加载失败（dev server 挂了、
        // 404、DNS 失败）时同样会发 Finished（它直接忽略了 IsSuccess），
        // 用它 show 会把渲染不出来的坏页面重新露出来。显示只由前端渲染完成后自己调。
        .on_page_load(|_, payload| {
            // 只记日志，**不在这里做任何窗口操作**。
            // 曾经在这里 `hide()` 窗口（想做到「页面没渲染好就绝不显示」），但那是在
            // 启动路径上动窗口，引入了难以复现的卡顿/闪失。现在退回到纯观察：
            // 每次页面加载记一行，出事时靠 trace.log 判断到底加载了几次。
            match payload.event() {
                PageLoadEvent::Started => {
                    super::trace::log("page_load: Started");
                }
                PageLoadEvent::Finished => {
                    super::trace::log(&format!("page_load: Finished url={}", payload.url()));
                }
            }
        })
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
