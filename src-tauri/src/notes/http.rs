// 只读对外接口：本机 HTTP，绑定 127.0.0.1，固定端口。
//
// 设计边界（刻意收窄，改这个文件前先读一遍）
// ---------------------------------------------------------------
//   · **只有两个 GET 端点**，没有任何写路由 —— 没有 POST/PUT/PATCH/DELETE。
//   · **只绑回环地址** 127.0.0.1（不是 0.0.0.0），局域网内其它机器连不上。
//   · **无鉴权**。正因为无鉴权，端点必须只读、且只暴露便签内容本身。
//     加写接口前必须先加鉴权 —— 否则同机任何进程都能改用户数据。
//   · **不引入 HTTP 库**（不依赖 axum/hyper/tiny_http）。
//     这个文件是安全边界，代码越少越好审；服务端只需要处理两个 GET。
//
// 端点
// ---------------------------------------------------------------
//   GET /notes/changes   全量轻量清单，每条只有 id / updated_at / deleted_at
//   GET /notes/{id}      单条完整内容（十个字段，见 FullNote）
//
// 全部响应 `Connection: close`，不做 keep-alive、不读请求体。

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream};
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Manager};

use super::db::Database;
use super::models::Page;

/// 对外接口端口（固定，写死）。
pub const PORT: u16 = 17321;

/// 单行请求上限，防畸形请求撑爆内存。
const MAX_LINE: u64 = 8192;
/// 单个连接的总超时。
const IO_TIMEOUT: Duration = Duration::from_secs(5);

/// 轻量变更条目 —— **只有这三列**。
#[derive(Serialize)]
struct ChangeRow {
    id: String,
    updated_at: i64,
    deleted_at: Option<i64>,
}

/// 单条完整内容 —— 字段就是下面这十个，不多不少。
/// `Page` 里还有 version / from_view / nodes，刻意不往外交。
#[derive(Serialize)]
struct FullNote {
    id: String,
    title: String,
    summary: String,
    content: String,
    created_at: i64,
    updated_at: i64,
    deleted_at: Option<i64>,
    kind: String,
    tags: Vec<String>,
    projects: Vec<String>,
}

impl From<Page> for FullNote {
    fn from(p: Page) -> Self {
        Self {
            id: p.id,
            title: p.title,
            summary: p.summary,
            content: p.content,
            created_at: p.created_at,
            updated_at: p.updated_at,
            deleted_at: p.deleted_at,
            kind: p.kind,
            tags: p.tags,
            projects: p.projects,
        }
    }
}

/// 起服务线程。端口被占用时只打印一行并放弃，不影响应用其余功能。
pub fn spawn(app: AppHandle) {
    std::thread::spawn(move || {
        // 只绑回环：SocketAddrV4(127.0.0.1, PORT)。
        let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, PORT);
        let listener = match TcpListener::bind(addr) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("[notes-http] 无法绑定 {addr}：{e}（端口可能被占用，只读接口未启用）");
                return;
            }
        };
        eprintln!("[notes-http] 只读接口已就绪 http://127.0.0.1:{PORT}/notes/changes");

        for stream in listener.incoming() {
            let Ok(stream) = stream else { continue };
            let app = app.clone();
            // 每连接一个线程：处理是纯读 + 本地 SQLite，成本低且不会互相阻塞。
            std::thread::spawn(move || {
                let _ = serve(stream, &app);
            });
        }
    });
}

fn serve(mut stream: TcpStream, app: &AppHandle) -> std::io::Result<()> {
    stream.set_read_timeout(Some(IO_TIMEOUT))?;
    stream.set_write_timeout(Some(IO_TIMEOUT))?;

    let mut reader = BufReader::new(stream.try_clone()?);

    // 请求行
    let mut request_line = String::new();
    if read_line_capped(&mut reader, &mut request_line)? == 0 {
        return Ok(()); // 连接被直接关掉
    }
    // 把请求头读完再回，避免客户端还在写、我们已经关连接。
    let mut header = String::new();
    loop {
        header.clear();
        if read_line_capped(&mut reader, &mut header)? == 0 {
            break;
        }
        if header == "\r\n" || header == "\n" {
            break;
        }
    }

    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("");
    let target = parts.next().unwrap_or("");

    if method != "GET" {
        return respond(&mut stream, 405, &err_json("read-only endpoint: only GET is allowed"));
    }

    let path = target.split('?').next().unwrap_or("");
    let db = app.state::<Database>();

    if path == "/notes/changes" {
        return match db.changes_all() {
            Ok(rows) => {
                let body: Vec<ChangeRow> = rows
                    .into_iter()
                    .map(|(id, updated_at, deleted_at)| ChangeRow {
                        id,
                        updated_at,
                        deleted_at,
                    })
                    .collect();
                respond_json(&mut stream, 200, &body)
            }
            Err(e) => respond(&mut stream, 500, &err_json(&e.to_string())),
        };
    }

    if let Some(id) = path.strip_prefix("/notes/") {
        if id.is_empty() {
            return respond(&mut stream, 404, &err_json("not found"));
        }
        return match db.get(id) {
            Ok(Some(page)) => respond_json(&mut stream, 200, &FullNote::from(page)),
            Ok(None) => respond(&mut stream, 404, &err_json("no such note")),
            Err(e) => respond(&mut stream, 500, &err_json(&e.to_string())),
        };
    }

    respond(&mut stream, 404, &err_json("not found"))
}

/// 读一行，最多 MAX_LINE 字节（超出的部分留给上层当作行结束处理）。
fn read_line_capped<R: BufRead>(r: &mut R, out: &mut String) -> std::io::Result<usize> {
    r.by_ref().take(MAX_LINE).read_line(out)
}

fn err_json(msg: &str) -> String {
    serde_json::json!({ "error": msg }).to_string()
}

fn respond_json<T: Serialize>(stream: &mut TcpStream, status: u16, value: &T) -> std::io::Result<()> {
    let body = serde_json::to_string(value).unwrap_or_else(|_| "[]".to_string());
    respond(stream, status, &body)
}

fn respond(stream: &mut TcpStream, status: u16, body: &str) -> std::io::Result<()> {
    let reason = match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        405 => "Method Not Allowed",
        500 => "Internal Server Error",
        _ => "OK",
    };
    let head = format!(
        "HTTP/1.1 {status} {reason}\r\n\
         Content-Type: application/json; charset=utf-8\r\n\
         Content-Length: {len}\r\n\
         Cache-Control: no-store\r\n\
         Connection: close\r\n\
         \r\n",
        len = body.as_bytes().len()
    );
    stream.write_all(head.as_bytes())?;
    stream.write_all(body.as_bytes())?;
    stream.flush()
}
