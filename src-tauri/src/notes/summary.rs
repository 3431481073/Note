// 摘要、ID、时间戳等纯工具函数。

/// 权威摘要：取前 20 个 Unicode 字符，换行替换为空格，去首尾空白。
pub fn make_summary(content: &str) -> String {
    let chars: Vec<char> = content.chars().collect();
    let first20: String = chars.iter().take(20).collect();
    first20.replace('\n', " ").trim().to_string()
}

/// 转义 LIKE 特殊字符（`\` `%` `_`），配合 SQL 端 `ESCAPE '\'` 使用。
/// 顺序：先转义反斜杠本身，再转义通配符，避免二次转义。
pub fn escape_like(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

/// 本 Note 项目的上一层文件夹（编译期路径，如 `D:\MyProject`），作为默认项目根。
pub fn default_project_dir() -> Option<String> {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.to_string_lossy().to_string())
        .filter(|d| !d.is_empty())
}

/// nanoid：URL 安全字母表，默认 21 字符。
pub fn nanoid(len: usize) -> String {
    use rand::Rng;
    const ALPHABET: &[u8] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut rng = rand::thread_rng();
    (0..len)
        .map(|_| ALPHABET[rng.gen_range(0..ALPHABET.len())] as char)
        .collect()
}

/// 当前毫秒时间戳（进程内严格单调递增）。
///
/// 毫秒分辨率下，同一毫秒内连续调用会返回相同值，导致「新建置顶」排序与游标分页在时间戳并列时退化为随机 ID 排序。
/// 这里用 CAS 保证每次调用都返回比上次更大的值，使 created_at/deleted_at 在同一进程内天然唯一且递增。
pub fn now_ms() -> i64 {
    use std::sync::atomic::{AtomicI64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};
    static LAST: AtomicI64 = AtomicI64::new(0);

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);

    let mut last = LAST.load(Ordering::Relaxed);
    loop {
        let next = if now > last { now } else { last + 1 };
        match LAST.compare_exchange(last, next, Ordering::SeqCst, Ordering::Relaxed) {
            Ok(_) => return next,
            Err(actual) => last = actual,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_truncates_to_20_chars() {
        let s = make_summary(&"a".repeat(100));
        assert_eq!(s.chars().count(), 20);
        assert_eq!(s, "a".repeat(20));
    }

    #[test]
    fn summary_replaces_newlines_with_space() {
        assert_eq!(make_summary("ab\ncd"), "ab cd");
        assert_eq!(make_summary("a\nb\nc"), "a b c");
    }

    #[test]
    fn summary_trims_whitespace() {
        assert_eq!(make_summary("  hello  "), "hello");
        assert_eq!(make_summary("\n  world \n"), "world");
    }

    #[test]
    fn summary_handles_unicode() {
        // 中文按字符计数，不按字节
        let s = make_summary(&"中".repeat(40));
        assert_eq!(s.chars().count(), 20);
    }

    #[test]
    fn escape_like_specials() {
        assert_eq!(escape_like("a\\b%c_d"), "a\\\\b\\%c\\_d");
    }

    #[test]
    fn nanoid_has_requested_length_and_alphabet() {
        let id = nanoid(21);
        assert_eq!(id.chars().count(), 21);
        const ALPHABET: &str =
            "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
        assert!(id.chars().all(|c| ALPHABET.contains(c)));
    }
}
