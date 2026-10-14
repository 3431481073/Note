use serde::{Deserialize, Serialize};

#[derive(Serialize, Clone)]
pub struct PageMeta {
    pub id: String,
    pub summary: String,
    pub created_at: i64,
    pub deleted_at: Option<i64>,
    pub version: i64,
    pub title: String,
    pub kind: String,
    pub tags: Vec<String>,
    pub projects: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_view: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct Page {
    pub id: String,
    pub content: String,
    pub summary: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub deleted_at: Option<i64>,
    pub version: i64,
    pub title: String,
    pub kind: String,
    pub tags: Vec<String>,
    pub projects: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_view: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nodes: Option<serde_json::Value>,
}

#[derive(Serialize)]
pub struct PageList {
    pub items: Vec<PageMeta>,
    pub has_more: bool,
    pub next_cursor: Option<Cursor>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Cursor {
    pub ts: i64,
    pub id: String,
}

#[derive(Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum View {
    Active,
    Trash,
}

#[derive(Serialize, Clone, Copy)]
pub struct Stats {
    pub active: i64,
    pub trash: i64,
    pub outline: i64,
    pub trash_outline: i64,
}

#[derive(Serialize, Clone)]
pub struct MetaIndex {
    pub tags: Vec<Tag>,
    pub projects: Vec<String>,
    /// 默认项目根（本 Note 项目的上一层，如 `D:\MyProject`）。前端据此隐藏指向根目录
    /// 本身的卡片标记——根目录对每张卡片都一样，显示出来没有信息量。
    pub default_project: Option<String>,
}

/// 标签：唯一 ID + 名字 + 说明 + 状态（激活/熄灭）。卡片通过 ID 引用，重命名不影响引用。
#[derive(Serialize, Clone)]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub description: String,
    pub extinguished: bool,
    pub deleted_at: Option<i64>,
    /// 引用该标签的卡片数（含回收站里尚未彻底删除的卡片）。
    pub card_count: i64,
}

#[derive(Serialize, Clone)]
pub struct UpdateResult {
    pub id: String,
    pub version: i64,
    pub summary: String,
    pub updated_at: i64,
}
