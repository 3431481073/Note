// 便签窗口点击穿透：透明区域返回 HTTRANSPARENT，可见面板矩形内正常响应。
// 窗口固定 1300 宽、透明、居中、不 resize；卡片态/视图态只是窗口内 CSS 布局不同。
use std::sync::Mutex;

use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::ScreenToClient;
use windows_sys::Win32::UI::Shell::{DefSubclassProc, SetWindowSubclass};
use windows_sys::Win32::UI::WindowsAndMessaging::{HTTRANSPARENT, WM_NCHITTEST};

/// 可见面板矩形（物理像素，客户区坐标）。前端 getBoundingClientRect() * devicePixelRatio 得到。
#[derive(Clone, Copy, Debug, serde::Deserialize)]
pub struct HitRect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

/// `None` = 未武装：前端从未成功上报过矩形（页面还没渲染出来）。
/// 此时**必须**按普通窗口处理。反过来（把空当成「全部穿透」）会让一个没渲染成功的
/// 窗口变成整块点不动、关不掉的死区——这是曾经的严重事故，不要再退回去。
static HIT_RECTS: Mutex<Option<Vec<HitRect>>> = Mutex::new(None);

const SUBCLASS_ID: usize = 0x6e6f7465_5f686974; // 任意唯一 id

unsafe extern "system" fn hit_test_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _uidsubclass: usize,
    _dwrefdata: usize,
) -> LRESULT {
    if msg == WM_NCHITTEST {
        // lparam 为屏幕坐标（物理像素），转客户区坐标后与命中矩形比较
        let sx = (lparam as i32 & 0xffff) as i16 as i32;
        let sy = ((lparam as i32 >> 16) & 0xffff) as i16 as i32;
        let mut pt = POINT { x: sx, y: sy };
        if ScreenToClient(hwnd, &mut pt) != 0 {
            // 先在锁内算出结论，放锁之后再调 DefSubclassProc——避免它在重入时再进本函数造成自我死锁。
            let inside = HIT_RECTS.lock().ok().map(|guard| match guard.as_ref() {
                None => true,
                Some(rects) => rects.iter().any(|r| {
                    pt.x >= r.x && pt.x < r.x + r.w && pt.y >= r.y && pt.y < r.y + r.h
                }),
            });
            // 只有明确判定「在命中矩形之外」才穿透；未武装/锁中毒一律当普通窗口。
            if inside == Some(false) {
                return HTTRANSPARENT as LRESULT;
            }
        }
    }
    DefSubclassProc(hwnd, msg, wparam, lparam)
}

pub fn install(hwnd: HWND) {
    unsafe {
        SetWindowSubclass(hwnd, Some(hit_test_proc), SUBCLASS_ID, 0);
    }
}

pub fn set_rects(rects: Vec<HitRect>) {
    // 真实界面至少有 CloseButton，不可能一个矩形都没有；空数组只可能是收集失败。
    // 此时保留上一份有效值——比「整窗穿透」安全得多。
    if rects.is_empty() {
        return;
    }
    if let Ok(mut guard) = HIT_RECTS.lock() {
        *guard = Some(rects);
    }
}
