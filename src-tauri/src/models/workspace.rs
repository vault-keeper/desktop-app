use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceModel {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub db_file: String,
    pub salt: String,
    pub sort_order: i32,
    pub created_at: String,
    pub updated_at: String,
}