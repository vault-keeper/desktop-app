use serde::{Deserialize, Serialize};
use tauri::State;
use crate::db::connection::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub entity_type: String,
    pub entity_id: String,
    pub title: String,
    pub snippet: String,
    pub tags: String,
}

#[derive(Debug, Deserialize)]
pub struct SearchFilters {
    pub entity_types: Option<Vec<String>>,
}

#[tauri::command]
pub async fn search(
    state: State<'_, AppState>,
    query: String,
    filters: Option<SearchFilters>,
) -> Result<Vec<SearchResult>, String> {
    if query.trim().is_empty() {
        return Ok(vec![]);
    }

    let conn = state.get_workspace_connection().await?;

    // Build the FTS5 query - wrap in quotes for phrase search, add * for prefix
    let fts_query = format!("\"{}\"*", query.replace('"', "\"\""));

    let type_filter = filters
        .and_then(|f| f.entity_types)
        .map(|types| {
            if types.is_empty() {
                String::new()
            } else {
                let quoted: Vec<String> = types.iter().map(|t| format!("'{}'", t)).collect();
                format!("AND entity_type IN ({})", quoted.join(","))
            }
        })
        .unwrap_or_default();

    let sql = format!(
        "SELECT entity_type, entity_id, title, snippet(search_index, 3, '<b>', '</b>', '...', 32), tags
         FROM search_index WHERE search_index MATCH ?1 {} LIMIT 50",
        type_filter
    );

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;

    let results = stmt.query_map(rusqlite::params![&fts_query], |row| {
        Ok(SearchResult {
            entity_type: row.get(0)?,
            entity_id: row.get(1)?,
            title: row.get(2)?,
            snippet: row.get(3)?,
            tags: row.get(4)?,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();

    Ok(results)
}

/// Simple keyword search fallback (uses LIKE instead of FTS5)
#[tauri::command]
pub async fn simple_search(
    state: State<'_, AppState>,
    query: String,
) -> Result<Vec<SearchResult>, String> {
    if query.trim().is_empty() {
        return Ok(vec![]);
    }

    let conn = state.get_workspace_connection().await?;
    let pattern = format!("%{}%", query);

    let mut results = Vec::new();

    // Search bookmarks
    let mut stmt = conn.prepare(
        "SELECT id, title, description FROM bookmarks WHERE title LIKE ?1 OR url LIKE ?1 OR description LIKE ?1 LIMIT 20"
    ).map_err(|e| e.to_string())?;
    let bookmark_results: Vec<SearchResult> = stmt.query_map(rusqlite::params![&pattern], |row| {
        Ok(SearchResult {
            entity_type: "bookmark".to_string(),
            entity_id: row.get(0)?,
            title: row.get(1)?,
            snippet: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
            tags: String::new(),
        })
    }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
    results.extend(bookmark_results);

    // Search notes (title only for encrypted notes)
    let mut stmt = conn.prepare(
        "SELECT id, title, CASE WHEN is_encrypted=0 THEN content ELSE '[已加密]' END FROM notes WHERE title LIKE ?1 OR (is_encrypted=0 AND content LIKE ?1) LIMIT 20"
    ).map_err(|e| e.to_string())?;
    let note_results: Vec<SearchResult> = stmt.query_map(rusqlite::params![&pattern], |row| {
        Ok(SearchResult {
            entity_type: "note".to_string(),
            entity_id: row.get(0)?,
            title: row.get(1)?,
            snippet: row.get::<_, String>(2)?.chars().take(100).collect(),
            tags: String::new(),
        })
    }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
    results.extend(note_results);

    // Search reports
    let mut stmt = conn.prepare(
        "SELECT id, title, content FROM reports WHERE title LIKE ?1 OR content LIKE ?1 LIMIT 20"
    ).map_err(|e| e.to_string())?;
    let report_results: Vec<SearchResult> = stmt.query_map(rusqlite::params![&pattern], |row| {
        Ok(SearchResult {
            entity_type: "report".to_string(),
            entity_id: row.get(0)?,
            title: row.get(1)?,
            snippet: row.get::<_, String>(2)?.chars().take(100).collect(),
            tags: String::new(),
        })
    }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
    results.extend(report_results);

    Ok(results)
}
