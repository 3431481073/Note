// 轻量诊断日志：往 `<app_data_dir>/trace.log` 追加一行 `<epoch_ms>\t<msg>`。
//
// 为什么要有它：曾出现过一次「界面像是卡住、随后凭空多出很多空白便签」的事故。
// 事后重启，所有现场证据都没了，只能靠猜——而静态排查连续推翻了七个假设。
// 这个日志就是为了把最关键的三件事留下来，下次再出现能直接看文件：
//   1. 页面到底被加载了几次（重载风暴？）—— 这是判断一切的分叉点
//   2. init 跑了几次、每次为什么做出「新建便签」的判断
//   3. 谁在什么时候建/删了便签
//
// 刻意保持极简：只追加、无缓冲、无依赖、任何失败都静默忽略——诊断设施绝不能
// 反过来成为故障源。
//
// 读法（时间戳是 epoch 毫秒，用这个换算成本地时间）：
//   Get-Content "$env:APPDATA\com.myworkbench.notes\trace.log" |
//     ForEach-Object { $p = $_ -split "`t",2;
//       "{0}  {1}" -f ([DateTimeOffset]::FromUnixTimeMilliseconds([int64]$p[0]).ToLocalTime().ToString("MM-dd HH:mm:ss.fff")), $p[1] }
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use tauri::{AppHandle, Manager};

static LOG_PATH: Mutex<Option<PathBuf>> = Mutex::new(None);

/// 定位日志文件（`<app_data_dir>/trace.log`）。由 setup 调用一次。
pub fn init(app: &AppHandle) {
    if let Ok(dir) = app.path().app_data_dir() {
        let _ = std::fs::create_dir_all(&dir);
        if let Ok(mut guard) = LOG_PATH.lock() {
            *guard = Some(dir.join("trace.log"));
        }
    }
}

/// 追加一行 `<epoch_ms>\t<msg>`。任何失败静默忽略。
pub fn log(msg: &str) {
    let path = match LOG_PATH.lock() {
        Ok(guard) => match guard.as_ref() {
            Some(p) => p.clone(),
            None => return,
        },
        Err(_) => return,
    };
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(&path) {
        let _ = writeln!(f, "{ms}\t{msg}");
    }
}
