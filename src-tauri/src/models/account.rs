use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountGroup {
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
pub struct Account {
    pub id: String,
    pub title: String,
    pub url: Option<String>,
    pub username: Option<String>,
    pub password: String,
    pub notes: Option<String>,
    pub icon_url: Option<String>,
    pub group_id: Option<String>,
    pub favorite: i32,
    pub sort_order: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateAccount {
    pub title: String,
    pub url: Option<String>,
    pub username: Option<String>,
    pub password: String,
    pub notes: Option<String>,
    pub icon_url: Option<String>,
    pub group_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAccount {
    pub title: Option<String>,
    pub url: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub notes: Option<String>,
    pub icon_url: Option<String>,
    pub group_id: Option<String>,
    pub favorite: Option<i32>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct PasswordGenOptions {
    pub length: Option<u32>,
    pub uppercase: Option<bool>,
    pub lowercase: Option<bool>,
    pub numbers: Option<bool>,
    pub symbols: Option<bool>,
    pub exclude_ambiguous: Option<bool>,
}