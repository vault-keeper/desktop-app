use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;
use crate::db::connection::AppState;

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

#[derive(Debug, Deserialize)]
pub struct CreateGroupPayload {
    pub name: String,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub parent_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateGroupPayload {
    pub name: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
}

fn row_to_bookmark(row: &rusqlite::Row) -> rusqlite::Result<Bookmark> {
    Ok(Bookmark {
        id: row.get(0)?,
        title: row.get(1)?,
        url: row.get(2)?,
        description: row.get(3)?,
        favicon_url: row.get(4)?,
        group_id: row.get(5)?,
        sort_order: row.get(6)?,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

fn row_to_group(row: &rusqlite::Row) -> rusqlite::Result<BookmarkGroup> {
    Ok(BookmarkGroup {
        id: row.get(0)?,
        name: row.get(1)?,
        icon: row.get(2)?,
        color: row.get(3)?,
        parent_id: row.get(4)?,
        sort_order: row.get(5)?,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
    })
}

#[tauri::command]
pub async fn list_bookmarks(
    state: State<'_, AppState>,
    group_id: Option<String>,
) -> Result<Vec<Bookmark>, String> {
    let conn = state.get_workspace_connection().await?;

    let mut bookmarks = Vec::new();
    if let Some(ref gid) = group_id {
        let mut stmt = conn.prepare(
            "SELECT id, title, url, description, favicon_url, group_id, sort_order, created_at, updated_at
             FROM bookmarks WHERE group_id=?1 ORDER BY sort_order, created_at"
        ).map_err(|e| e.to_string())?;
        let rows = stmt.query_map(rusqlite::params![gid], row_to_bookmark)
            .map_err(|e| e.to_string())?;
        for row in rows { if let Ok(b) = row { bookmarks.push(b); } }
    } else {
        let mut stmt = conn.prepare(
            "SELECT id, title, url, description, favicon_url, group_id, sort_order, created_at, updated_at
             FROM bookmarks ORDER BY sort_order, created_at"
        ).map_err(|e| e.to_string())?;
        let rows = stmt.query_map([], row_to_bookmark)
            .map_err(|e| e.to_string())?;
        for row in rows { if let Ok(b) = row { bookmarks.push(b); } }
    }

    Ok(bookmarks)
}

#[tauri::command]
pub async fn get_bookmark(
    state: State<'_, AppState>,
    id: String,
) -> Result<Bookmark, String> {
    let conn = state.get_workspace_connection().await?;
    conn.query_row(
        "SELECT id, title, url, description, favicon_url, group_id, sort_order, created_at, updated_at
         FROM bookmarks WHERE id=?1",
        rusqlite::params![&id],
        row_to_bookmark,
    ).map_err(|e| format!("Bookmark not found: {}", e))
}

#[tauri::command]
pub async fn create_bookmark(
    state: State<'_, AppState>,
    data: CreateBookmark,
) -> Result<Bookmark, String> {
    let conn = state.get_workspace_connection().await?;
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO bookmarks (id, title, url, description, favicon_url, group_id, sort_order, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, ?7, ?8)",
        rusqlite::params![&id, &data.title, &data.url, &data.description, &data.favicon_url, &data.group_id, &now, &now],
    ).map_err(|e| format!("Failed to create bookmark: {}", e))?;

    Ok(Bookmark {
        id,
        title: data.title,
        url: data.url,
        description: data.description,
        favicon_url: data.favicon_url,
        group_id: data.group_id,
        sort_order: 0,
        created_at: now.clone(),
        updated_at: now,
    })
}

#[tauri::command]
pub async fn update_bookmark(
    state: State<'_, AppState>,
    id: String,
    data: UpdateBookmark,
) -> Result<Bookmark, String> {
    let conn = state.get_workspace_connection().await?;
    let now = chrono::Utc::now().to_rfc3339();

    if let Some(ref v) = data.title {
        conn.execute("UPDATE bookmarks SET title=?1, updated_at=?2 WHERE id=?3", rusqlite::params![v, &now, &id]).map_err(|e| e.to_string())?;
    }
    if let Some(ref v) = data.url {
        conn.execute("UPDATE bookmarks SET url=?1, updated_at=?2 WHERE id=?3", rusqlite::params![v, &now, &id]).map_err(|e| e.to_string())?;
    }
    if data.description.is_some() {
        conn.execute("UPDATE bookmarks SET description=?1, updated_at=?2 WHERE id=?3", rusqlite::params![&data.description, &now, &id]).map_err(|e| e.to_string())?;
    }
    if data.group_id.is_some() {
        conn.execute("UPDATE bookmarks SET group_id=?1, updated_at=?2 WHERE id=?3", rusqlite::params![&data.group_id, &now, &id]).map_err(|e| e.to_string())?;
    }
    if let Some(v) = data.sort_order {
        conn.execute("UPDATE bookmarks SET sort_order=?1, updated_at=?2 WHERE id=?3", rusqlite::params![v, &now, &id]).map_err(|e| e.to_string())?;
    }

    get_bookmark(state, id).await
}

#[tauri::command]
pub async fn delete_bookmark(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let conn = state.get_workspace_connection().await?;
    // Remove from search index
    conn.execute("DELETE FROM search_index WHERE entity_type='bookmark' AND entity_id=?1", rusqlite::params![&id])
        .ok();
    conn.execute("DELETE FROM bookmarks WHERE id=?1", rusqlite::params![&id])
        .map_err(|e| format!("Failed to delete bookmark: {}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn list_bookmark_groups(
    state: State<'_, AppState>,
) -> Result<Vec<BookmarkGroup>, String> {
    let conn = state.get_workspace_connection().await?;
    let mut stmt = conn.prepare(
        "SELECT id, name, icon, color, parent_id, sort_order, created_at, updated_at
         FROM bookmark_groups ORDER BY sort_order, name"
    ).map_err(|e| e.to_string())?;
    let groups = stmt.query_map([], row_to_group)
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok()).collect();
    Ok(groups)
}

#[tauri::command]
pub async fn create_bookmark_group(
    state: State<'_, AppState>,
    data: CreateGroupPayload,
) -> Result<BookmarkGroup, String> {
    let conn = state.get_workspace_connection().await?;
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO bookmark_groups (id, name, icon, color, parent_id, sort_order, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6, ?7)",
        rusqlite::params![&id, &data.name, &data.icon, &data.color, &data.parent_id, &now, &now],
    ).map_err(|e| format!("Failed to create bookmark group: {}", e))?;

    Ok(BookmarkGroup {
        id,
        name: data.name,
        icon: data.icon,
        color: data.color,
        parent_id: data.parent_id,
        sort_order: 0,
        created_at: now.clone(),
        updated_at: now,
    })
}

#[tauri::command]
pub async fn update_bookmark_group(
    state: State<'_, AppState>,
    id: String,
    data: UpdateGroupPayload,
) -> Result<BookmarkGroup, String> {
    let conn = state.get_workspace_connection().await?;
    let now = chrono::Utc::now().to_rfc3339();

    if let Some(ref v) = data.name {
        conn.execute("UPDATE bookmark_groups SET name=?1, updated_at=?2 WHERE id=?3", rusqlite::params![v, &now, &id]).map_err(|e| e.to_string())?;
    }
    if data.icon.is_some() {
        conn.execute("UPDATE bookmark_groups SET icon=?1, updated_at=?2 WHERE id=?3", rusqlite::params![&data.icon, &now, &id]).map_err(|e| e.to_string())?;
    }
    if data.color.is_some() {
        conn.execute("UPDATE bookmark_groups SET color=?1, updated_at=?2 WHERE id=?3", rusqlite::params![&data.color, &now, &id]).map_err(|e| e.to_string())?;
    }

    conn.query_row(
        "SELECT id, name, icon, color, parent_id, sort_order, created_at, updated_at FROM bookmark_groups WHERE id=?1",
        rusqlite::params![&id],
        row_to_group,
    ).map_err(|e| format!("Group not found: {}", e))
}

#[tauri::command]
pub async fn delete_bookmark_group(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let conn = state.get_workspace_connection().await?;
    conn.execute("DELETE FROM bookmark_groups WHERE id=?1", rusqlite::params![&id])
        .map_err(|e| format!("Failed to delete bookmark group: {}", e))?;
    Ok(())
}
