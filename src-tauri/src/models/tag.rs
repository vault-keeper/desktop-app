use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Taggable {
    pub tag_id: String,
    pub taggable_type: String,
    pub taggable_id: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub entity_type: String,
    pub entity_id: String,
    pub title: String,
    pub content: String,
    pub tags: String,
}

#[derive(Debug, Deserialize)]
pub struct SearchFilters {
    pub entity_types: Option<Vec<String>>,
}