use serde::{Deserialize, Serialize};
use tauri::State;
use crate::db::connection::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub auto_lock_minutes: i64,
    pub theme: String,
    pub language: String,
}

#[tauri::command]
pub async fn get_settings(
    state: State<'_, AppState>,
) -> Result<AppSettings, String> {
    let conn = state.get_meta_connection().await?;

    let auto_lock: i64 = conn.query_row(
        "SELECT value FROM app_settings WHERE key='auto_lock_minutes'",
        [], |row| row.get::<_, String>(0),
    ).map(|v| v.parse().unwrap_or(5)).unwrap_or(5);

    let theme: String = conn.query_row(
        "SELECT value FROM app_settings WHERE key='theme'",
        [], |row| row.get(0),
    ).unwrap_or_else(|_| "system".to_string());

    let language: String = conn.query_row(
        "SELECT value FROM app_settings WHERE key='language'",
        [], |row| row.get(0),
    ).unwrap_or_else(|_| "zh-CN".to_string());

    Ok(AppSettings { auto_lock_minutes: auto_lock, theme, language })
}

#[tauri::command]
pub async fn update_setting(
    state: State<'_, AppState>,
    key: String,
    value: String,
) -> Result<(), String> {
    let conn = state.get_meta_connection().await?;
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO app_settings (key, value, updated_at) VALUES (?1, ?2, ?3)
         ON CONFLICT(key) DO UPDATE SET value=excluded.value, updated_at=excluded.updated_at",
        rusqlite::params![&key, &value, &now],
    ).map_err(|e| format!("Failed to update setting: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn get_auto_lock_timeout(
    state: State<'_, AppState>,
) -> Result<i64, String> {
    let settings = get_settings(state).await?;
    Ok(settings.auto_lock_minutes)
}

#[tauri::command]
pub async fn set_auto_lock_timeout(
    state: State<'_, AppState>,
    minutes: i64,
) -> Result<(), String> {
    update_setting(state, "auto_lock_minutes".to_string(), minutes.to_string()).await
}
