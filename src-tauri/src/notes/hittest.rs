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

static HIT_RECTS: Mutex<Vec<HitRect>> = Mutex::new(Vec::new());

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
            if let Ok(rects) = HIT_RECTS.lock() {
                for r in rects.iter() {
                    if pt.x >= r.x && pt.x < r.x + r.w && pt.y >= r.y && pt.y < r.y + r.h {
                        return DefSubclassProc(hwnd, msg, wparam, lparam);
                    }
                }
            }
            return HTTRANSPARENT as LRESULT;
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
    if let Ok(mut guard) = HIT_RECTS.lock() {
        *guard = rects;
    }
}
