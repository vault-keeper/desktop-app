use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;
use std::path::Path;
use crate::db::connection::AppState;

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
    pub created_at: String,
    pub updated_at: String,
    /// Absolute filesystem path for local assets (not stored in DB, computed on query).
    pub absolute_path: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UploadMediaPayload {
    pub file_path: String,
    pub description: Option<String>,
}

fn row_to_media(row: &rusqlite::Row) -> rusqlite::Result<MediaAsset> {
    Ok(MediaAsset {
        id: row.get(0)?,
        filename: row.get(1)?,
        mime_type: row.get(2)?,
        size: row.get(3)?,
        storage_type: row.get(4)?,
        storage_path: row.get(5)?,
        thumbnail_path: row.get(6)?,
        description: row.get(7)?,
        metadata: row.get(8)?,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
        absolute_path: None,
    })
}

fn with_absolute_path(mut asset: MediaAsset, app_data_dir: &std::path::Path) -> MediaAsset {
    if asset.storage_type == "local" {
        let full = app_data_dir.join(&asset.storage_path);
        asset.absolute_path = full.to_str().map(|s| s.to_string());
    }
    asset
}

fn guess_mime_type(filename: &str) -> Option<&'static str> {
    let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "jpg" | "jpeg" => Some("image/jpeg"),
        "png"          => Some("image/png"),
        "gif"          => Some("image/gif"),
        "webp"         => Some("image/webp"),
        "svg"          => Some("image/svg+xml"),
        "mp3"          => Some("audio/mpeg"),
        "wav"          => Some("audio/wav"),
        "ogg"          => Some("audio/ogg"),
        "flac"         => Some("audio/flac"),
        "m4a"          => Some("audio/mp4"),
        "aac"          => Some("audio/aac"),
        "mp4"          => Some("video/mp4"),
        "webm"         => Some("video/webm"),
        "mov"          => Some("video/quicktime"),
        "mkv"          => Some("video/x-matroska"),
        _              => None,
    }
}

#[tauri::command]
pub async fn list_media_assets(
    state: State<'_, AppState>,
) -> Result<Vec<MediaAsset>, String> {
    let app_data_dir = state.get_app_data_dir()?;
    let conn = state.get_workspace_connection().await?;
    let mut stmt = conn.prepare(
        "SELECT id, filename, mime_type, size, storage_type, storage_path, thumbnail_path, description, metadata, created_at, updated_at
         FROM media_assets ORDER BY created_at DESC"
    ).map_err(|e| e.to_string())?;
    let assets: Vec<MediaAsset> = stmt.query_map([], row_to_media)
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .map(|a| with_absolute_path(a, &app_data_dir))
        .collect();
    Ok(assets)
}

#[tauri::command]
pub async fn get_media_asset(
    state: State<'_, AppState>,
    id: String,
) -> Result<MediaAsset, String> {
    let app_data_dir = state.get_app_data_dir()?;
    let conn = state.get_workspace_connection().await?;
    let asset = conn.query_row(
        "SELECT id, filename, mime_type, size, storage_type, storage_path, thumbnail_path, description, metadata, created_at, updated_at
         FROM media_assets WHERE id=?1",
        rusqlite::params![&id],
        row_to_media,
    ).map_err(|e| format!("Media asset not found: {}", e))?;
    Ok(with_absolute_path(asset, &app_data_dir))
}

#[tauri::command]
pub async fn upload_media(
    state: State<'_, AppState>,
    payload: UploadMediaPayload,
) -> Result<MediaAsset, String> {
    let source_path = Path::new(&payload.file_path);

    if !source_path.exists() {
        return Err(format!("File not found: {}", payload.file_path));
    }

    let filename = source_path.file_name()
        .and_then(|n| n.to_str())
        .ok_or("Invalid filename")?
        .to_string();

    let file_size = source_path.metadata()
        .map_err(|e| format!("Failed to read file metadata: {}", e))?.len() as i64;

    let mime_type = guess_mime_type(&filename)
        .ok_or_else(|| "Unsupported file type. Only images, audio, and video files are allowed.".to_string())?
        .to_string();

    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let app_data_dir = state.get_app_data_dir()?;

    let media_dir = app_data_dir.join("media");
    std::fs::create_dir_all(&media_dir)
        .map_err(|e| format!("Failed to create media directory: {}", e))?;

    let dest_filename = format!("{}_{}", &id[..8], filename);
    let dest_path = media_dir.join(&dest_filename);
    std::fs::copy(source_path, &dest_path)
        .map_err(|e| format!("Failed to copy file: {}", e))?;

    let storage_path = format!("media/{}", dest_filename);

    let conn = state.get_workspace_connection().await?;
    conn.execute(
        "INSERT INTO media_assets (id, filename, mime_type, size, storage_type, storage_path, description, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![
            &id, &filename, &mime_type, file_size,
            "local", &storage_path,
            &payload.description, &now, &now
        ],
    ).map_err(|e| format!("Failed to save media asset: {}", e))?;

    conn.execute(
        "INSERT OR REPLACE INTO search_index (entity_type, entity_id, title, content, tags) VALUES ('media', ?1, ?2, '', '')",
        rusqlite::params![&id, &filename],
    ).ok();

    Ok(MediaAsset {
        id, filename, mime_type, size: file_size,
        storage_type: "local".to_string(),
        storage_path: storage_path.clone(),
        thumbnail_path: None,
        description: payload.description,
        metadata: None,
        created_at: now.clone(),
        updated_at: now,
        absolute_path: app_data_dir.join(&storage_path).to_str().map(|s| s.to_string()),
    })
}

#[tauri::command]
pub async fn delete_media_asset(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let asset = get_media_asset(state.clone(), id.clone()).await?;

    if asset.storage_type == "local" {
        let app_data_dir = state.get_app_data_dir()?;
        let file_path = app_data_dir.join(&asset.storage_path);
        if file_path.exists() {
            std::fs::remove_file(&file_path)
                .map_err(|e| format!("Failed to delete file: {}", e))?;
        }
    }

    let conn = state.get_workspace_connection().await?;
    conn.execute("DELETE FROM search_index WHERE entity_type='media' AND entity_id=?1", rusqlite::params![&id]).ok();
    conn.execute("DELETE FROM media_assets WHERE id=?1", rusqlite::params![&id])
        .map_err(|e| format!("Failed to delete media asset: {}", e))?;
    Ok(())
}
