use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaAsset {
    pub id: String,
    pub filename: String,
    pub mime_type: String,
    pub size: i64,
    pub storage_type: String,
    pub storage_path: String,
    pub thumbnail_path: Option<String>,
    pub description: Option<String>,
    pub metadata: Option<String>,
    pub s3_bucket: Option<String>,
    pub s3_region: Option<String>,
    pub s3_endpoint: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct UploadMediaPayload {
    pub file_path: String,
    pub storage_type: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct S3Config {
    pub provider: String,
    pub endpoint: Option<String>,
    pub region: String,
    pub bucket: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub path_prefix: Option<String>,
    pub public_url: Option<String>,
}