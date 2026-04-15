use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

use crate::db::connection::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub db_file: String,
    pub sort_order: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateWorkspacePayload {
    pub name: String,
    pub icon: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWorkspacePayload {
    pub id: String,
    pub name: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
}

#[tauri::command]
pub async fn list_workspaces(
    state: State<'_, AppState>,
) -> Result<Vec<Workspace>, String> {
    let conn = state.get_meta_connection().await?;

    let mut stmt = conn.prepare(
        "SELECT id, name, icon, color, db_file, sort_order, created_at, updated_at
         FROM workspaces ORDER BY sort_order, created_at"
    ).map_err(|e| format!("Failed to prepare query: {}", e))?;

    let workspaces = stmt.query_map([], |row| {
        Ok(Workspace {
            id: row.get(0)?,
            name: row.get(1)?,
            icon: row.get(2)?,
            color: row.get(3)?,
            db_file: row.get(4)?,
            sort_order: row.get(5)?,
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
        })
    }).map_err(|e| format!("Failed to query workspaces: {}", e))?
    .filter_map(|ws| ws.ok())
    .collect();

    Ok(workspaces)
}

#[tauri::command]
pub async fn create_workspace(
    state: State<'_, AppState>,
    payload: CreateWorkspacePayload,
) -> Result<Workspace, String> {
    if !state.is_unlocked() {
        return Err("Vault is locked".to_string());
    }

    let id = Uuid::new_v4().to_string();
    let db_file = format!("workspace_{}.db", &id[..8]);

    // Create the workspace database
    state.create_workspace_db(&db_file).await?;

    let now = chrono::Utc::now().to_rfc3339();
    let workspace = Workspace {
        id,
        name: payload.name,
        icon: payload.icon,
        color: payload.color,
        db_file,
        sort_order: 0,
        created_at: now.clone(),
        updated_at: now,
    };

    let conn = state.get_meta_connection().await?;
    conn.execute(
        "INSERT INTO workspaces (id, name, icon, color, db_file, sort_order, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![
            &workspace.id,
            &workspace.name,
            &workspace.icon,
            &workspace.color,
            &workspace.db_file,
            workspace.sort_order,
            &workspace.created_at,
            &workspace.updated_at,
        ],
    ).map_err(|e| format!("Failed to insert workspace: {}", e))?;

    Ok(workspace)
}

#[tauri::command]
pub async fn update_workspace(
    state: State<'_, AppState>,
    payload: UpdateWorkspacePayload,
) -> Result<Workspace, String> {
    let conn = state.get_meta_connection().await?;
    let now = chrono::Utc::now().to_rfc3339();

    if let Some(ref name) = payload.name {
        conn.execute("UPDATE workspaces SET name=?1, updated_at=?2 WHERE id=?3",
            rusqlite::params![name, &now, &payload.id])
            .map_err(|e| format!("Failed to update workspace name: {}", e))?;
    }
    if let Some(ref icon) = payload.icon {
        conn.execute("UPDATE workspaces SET icon=?1, updated_at=?2 WHERE id=?3",
            rusqlite::params![icon, &now, &payload.id])
            .map_err(|e| format!("Failed to update workspace icon: {}", e))?;
    }
    if let Some(ref color) = payload.color {
        conn.execute("UPDATE workspaces SET color=?1, updated_at=?2 WHERE id=?3",
            rusqlite::params![color, &now, &payload.id])
            .map_err(|e| format!("Failed to update workspace color: {}", e))?;
    }

    let workspace = conn.query_row(
        "SELECT id, name, icon, color, db_file, sort_order, created_at, updated_at FROM workspaces WHERE id=?1",
        rusqlite::params![&payload.id],
        |row| Ok(Workspace {
            id: row.get(0)?,
            name: row.get(1)?,
            icon: row.get(2)?,
            color: row.get(3)?,
            db_file: row.get(4)?,
            sort_order: row.get(5)?,
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
        }),
    ).map_err(|e| format!("Failed to fetch updated workspace: {}", e))?;

    Ok(workspace)
}

#[tauri::command]
pub async fn delete_workspace(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let conn = state.get_meta_connection().await?;

    let db_file: String = conn.query_row(
        "SELECT db_file FROM workspaces WHERE id=?1",
        rusqlite::params![&id],
        |row| row.get(0),
    ).map_err(|e| format!("Workspace not found: {}", e))?;

    conn.execute("DELETE FROM workspaces WHERE id=?1", rusqlite::params![&id])
        .map_err(|e| format!("Failed to delete workspace: {}", e))?;

    state.delete_workspace_db(&db_file).await?;
    Ok(())
}

#[tauri::command]
pub async fn switch_workspace(
    state: State<'_, AppState>,
    id: String,
) -> Result<Workspace, String> {
    if !state.is_unlocked() {
        return Err("Vault is locked".to_string());
    }

    let conn = state.get_meta_connection().await?;
    let workspace = conn.query_row(
        "SELECT id, name, icon, color, db_file, sort_order, created_at, updated_at FROM workspaces WHERE id=?1",
        rusqlite::params![&id],
        |row| Ok(Workspace {
            id: row.get(0)?,
            name: row.get(1)?,
            icon: row.get(2)?,
            color: row.get(3)?,
            db_file: row.get(4)?,
            sort_order: row.get(5)?,
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
        }),
    ).map_err(|e| format!("Workspace not found: {}", e))?;

    state.switch_workspace_db(&workspace.db_file).await?;
    state.set_current_workspace(workspace.clone());

    Ok(workspace)
}

#[tauri::command]
pub async fn get_current_workspace(
    state: State<'_, AppState>,
) -> Result<Option<Workspace>, String> {
    Ok(state.get_current_workspace())
}
