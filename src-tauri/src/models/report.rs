use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportTemplate {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub report_type: String,
    pub template_content: String,
    pub is_default: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub id: String,
    pub title: String,
    #[serde(rename = "type")]
    pub report_type: String,
    pub content: String,
    pub date: String,
    pub week_start: Option<String>,
    pub week_end: Option<String>,
    pub month: Option<String>,
    pub template_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateReport {
    pub title: String,
    #[serde(rename = "type")]
    pub report_type: String,
    pub content: String,
    pub date: String,
    pub week_start: Option<String>,
    pub week_end: Option<String>,
    pub month: Option<String>,
    pub template_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateReport {
    pub title: Option<String>,
    pub content: Option<String>,
    pub date: Option<String>,
    pub week_start: Option<String>,
    pub week_end: Option<String>,
    pub month: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DateRange {
    pub start: String,
    pub end: String,
}