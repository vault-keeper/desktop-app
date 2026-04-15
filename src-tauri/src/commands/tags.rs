use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;
use crate::db::connection::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub created_at: String,
    /// Number of entities that reference this tag. Always populated by list_tags;
    /// 0 for tags returned by get_entity_tags.
    pub usage_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Taggable {
    pub tag_id: String,
    pub taggable_type: String,
    pub taggable_id: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateTag {
    pub name: String,
    pub color: Option<String>,
    pub icon: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTag {
    pub name: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
}

fn row_to_tag(row: &rusqlite::Row) -> rusqlite::Result<Tag> {
    Ok(Tag {
        id: row.get(0)?,
        name: row.get(1)?,
        color: row.get(2)?,
        icon: row.get(3)?,
        created_at: row.get(4)?,
        usage_count: row.get(5).unwrap_or(0),
    })
}

fn tag_usage_count(conn: &rusqlite::Connection, id: &str) -> i64 {
    conn.query_row(
        "SELECT COUNT(*) FROM taggables WHERE tag_id=?1",
        rusqlite::params![id],
        |row| row.get(0),
    ).unwrap_or(0)
}

#[tauri::command]
pub async fn list_tags(
    state: State<'_, AppState>,
) -> Result<Vec<Tag>, String> {
    let conn = state.get_workspace_connection().await?;
    let mut stmt = conn.prepare(
        "SELECT t.id, t.name, t.color, t.icon, t.created_at,
                COUNT(tg.tag_id) AS usage_count
         FROM tags t
         LEFT JOIN taggables tg ON t.id = tg.tag_id
         GROUP BY t.id
         ORDER BY t.name"
    ).map_err(|e| e.to_string())?;
    let tags = stmt.query_map([], row_to_tag)
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok()).collect();
    Ok(tags)
}

#[tauri::command]
pub async fn create_tag(
    state: State<'_, AppState>,
    data: CreateTag,
) -> Result<Tag, String> {
    let conn = state.get_workspace_connection().await?;
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO tags (id, name, color, icon, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![&id, &data.name, &data.color, &data.icon, &now],
    ).map_err(|e| format!("Failed to create tag: {}", e))?;

    Ok(Tag { id, name: data.name, color: data.color, icon: data.icon, created_at: now, usage_count: 0 })
}

#[tauri::command]
pub async fn update_tag(
    state: State<'_, AppState>,
    id: String,
    data: UpdateTag,
) -> Result<Tag, String> {
    let conn = state.get_workspace_connection().await?;

    let count = tag_usage_count(&conn, &id);
    if count > 0 {
        return Err(format!("标签已被 {} 项内容引用，不可修改", count));
    }

    if let Some(ref v) = data.name {
        conn.execute("UPDATE tags SET name=?1 WHERE id=?2",
            rusqlite::params![v, &id]).map_err(|e| e.to_string())?;
    }
    if data.color.is_some() {
        conn.execute("UPDATE tags SET color=?1 WHERE id=?2",
            rusqlite::params![&data.color, &id]).map_err(|e| e.to_string())?;
    }
    if data.icon.is_some() {
        conn.execute("UPDATE tags SET icon=?1 WHERE id=?2",
            rusqlite::params![&data.icon, &id]).map_err(|e| e.to_string())?;
    }

    conn.query_row(
        "SELECT t.id, t.name, t.color, t.icon, t.created_at, 0 as usage_count
         FROM tags t WHERE t.id=?1",
        rusqlite::params![&id],
        row_to_tag,
    ).map_err(|e| format!("Tag not found: {}", e))
}

#[tauri::command]
pub async fn delete_tag(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let conn = state.get_workspace_connection().await?;

    let count = tag_usage_count(&conn, &id);
    if count > 0 {
        return Err(format!("标签已被 {} 项内容引用，不可删除", count));
    }

    conn.execute("DELETE FROM tags WHERE id=?1", rusqlite::params![&id])
        .map_err(|e| format!("Failed to delete tag: {}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn tag_entity(
    state: State<'_, AppState>,
    tag_id: String,
    entity_type: String,
    entity_id: String,
) -> Result<(), String> {
    let conn = state.get_workspace_connection().await?;
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT OR IGNORE INTO taggables (tag_id, taggable_type, taggable_id, created_at)
         VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![&tag_id, &entity_type, &entity_id, &now],
    ).map_err(|e| format!("Failed to tag entity: {}", e))?;

    // Update search index
    let tag_name: String = conn.query_row(
        "SELECT name FROM tags WHERE id=?1", rusqlite::params![&tag_id],
        |row| row.get(0),
    ).unwrap_or_default();

    conn.execute(
        "UPDATE search_index SET tags = CASE WHEN tags='' THEN ?1 ELSE tags || ',' || ?1 END
         WHERE entity_type=?2 AND entity_id=?3",
        rusqlite::params![&tag_name, &entity_type, &entity_id],
    ).ok();

    Ok(())
}

#[tauri::command]
pub async fn untag_entity(
    state: State<'_, AppState>,
    tag_id: String,
    entity_type: String,
    entity_id: String,
) -> Result<(), String> {
    let conn = state.get_workspace_connection().await?;
    conn.execute(
        "DELETE FROM taggables WHERE tag_id=?1 AND taggable_type=?2 AND taggable_id=?3",
        rusqlite::params![&tag_id, &entity_type, &entity_id],
    ).map_err(|e| format!("Failed to untag entity: {}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn get_entity_tags(
    state: State<'_, AppState>,
    entity_type: String,
    entity_id: String,
) -> Result<Vec<Tag>, String> {
    let conn = state.get_workspace_connection().await?;
    let mut stmt = conn.prepare(
        "SELECT t.id, t.name, t.color, t.icon, t.created_at, 0 AS usage_count
         FROM tags t
         JOIN taggables tg ON t.id = tg.tag_id
         WHERE tg.taggable_type=?1 AND tg.taggable_id=?2
         ORDER BY t.name"
    ).map_err(|e| e.to_string())?;
    let tags = stmt.query_map(rusqlite::params![&entity_type, &entity_id], row_to_tag)
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok()).collect();
    Ok(tags)
}

/// A flattened row used to bulk-load all tags for every entity of a given type.
#[derive(Debug, Clone, Serialize)]
pub struct EntityTagEntry {
    pub entity_id: String,
    pub id: String,
    pub name: String,
    pub color: Option<String>,
}

/// Returns all tag associations for every entity of `entity_type` in one query.
/// The frontend groups the flat list by `entity_id` to build a lookup map.
#[tauri::command]
pub async fn list_all_entity_tags(
    state: State<'_, AppState>,
    entity_type: String,
) -> Result<Vec<EntityTagEntry>, String> {
    let conn = state.get_workspace_connection().await?;
    let mut stmt = conn.prepare(
        "SELECT tg.taggable_id, t.id, t.name, t.color
         FROM taggables tg
         JOIN tags t ON t.id = tg.tag_id
         WHERE tg.taggable_type = ?1
         ORDER BY tg.taggable_id, t.name"
    ).map_err(|e| e.to_string())?;
    let entries = stmt.query_map(rusqlite::params![&entity_type], |row| {
        Ok(EntityTagEntry {
            entity_id: row.get(0)?,
            id:        row.get(1)?,
            name:      row.get(2)?,
            color:     row.get(3)?,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();
    Ok(entries)
}

#[tauri::command]
pub async fn get_entities_by_tag(
    state: State<'_, AppState>,
    tag_id: String,
) -> Result<Vec<Taggable>, String> {
    let conn = state.get_workspace_connection().await?;
    let mut stmt = conn.prepare(
        "SELECT tag_id, taggable_type, taggable_id, created_at FROM taggables WHERE tag_id=?1"
    ).map_err(|e| e.to_string())?;
    let taggables = stmt.query_map(rusqlite::params![&tag_id], |row| {
        Ok(Taggable {
            tag_id: row.get(0)?,
            taggable_type: row.get(1)?,
            taggable_id: row.get(2)?,
            created_at: row.get(3)?,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok()).collect();
    Ok(taggables)
}
