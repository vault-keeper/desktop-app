use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkGroup {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub parent_id: Option<String>,
    pub sort_order: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub id: String,
    pub title: String,
    pub url: String,
    pub description: Option<String>,
    pub favicon_url: Option<String>,
    pub group_id: Option<String>,
    pub sort_order: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateBookmark {
    pub title: String,
    pub url: String,
    pub description: Option<String>,
    pub favicon_url: Option<String>,
    pub group_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateBookmark {
    pub title: Option<String>,
    pub url: Option<String>,
    pub description: Option<String>,
    pub favicon_url: Option<String>,
    pub group_id: Option<String>,
    pub sort_order: Option<i32>,
}