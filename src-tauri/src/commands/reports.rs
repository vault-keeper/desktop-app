use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;
use crate::db::connection::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub id: String,
    pub title: String,
    pub r#type: String,
    pub content: String,
    pub date: String,
    pub week_start: Option<String>,
    pub week_end: Option<String>,
    pub month: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateReport {
    pub title: String,
    pub r#type: String,
    pub content: String,
    pub date: String,
    pub week_start: Option<String>,
    pub week_end: Option<String>,
    pub month: Option<String>,
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

fn row_to_report(row: &rusqlite::Row) -> rusqlite::Result<Report> {
    Ok(Report {
        id: row.get(0)?,
        title: row.get(1)?,
        r#type: row.get(2)?,
        content: row.get(3)?,
        date: row.get(4)?,
        week_start: row.get(5)?,
        week_end: row.get(6)?,
        month: row.get(7)?,
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
    })
}

#[tauri::command]
pub async fn list_reports(
    state: State<'_, AppState>,
    r#type: Option<String>,
    year: Option<i32>,
    month: Option<String>,
) -> Result<Vec<Report>, String> {
    let conn = state.get_workspace_connection().await?;

    let mut conditions = Vec::new();
    if let Some(ref t) = r#type {
        if !t.is_empty() { conditions.push(format!("type='{}'", t)); }
    }
    if let Some(y) = year { conditions.push(format!("strftime('%Y', date)='{}'", y)); }
    if let Some(ref m) = month {
        if !m.is_empty() { conditions.push(format!("month='{}'", m)); }
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    let sql = format!(
        "SELECT id, title, type, content, date, week_start, week_end, month, created_at, updated_at
         FROM reports {} ORDER BY date DESC, created_at DESC",
        where_clause
    );

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let reports = stmt.query_map([], row_to_report)
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok()).collect();
    Ok(reports)
}

#[tauri::command]
pub async fn get_report(
    state: State<'_, AppState>,
    id: String,
) -> Result<Report, String> {
    let conn = state.get_workspace_connection().await?;
    conn.query_row(
        "SELECT id, title, type, content, date, week_start, week_end, month, created_at, updated_at FROM reports WHERE id=?1",
        rusqlite::params![&id],
        row_to_report,
    ).map_err(|e| format!("Report not found: {}", e))
}

#[tauri::command]
pub async fn create_report(
    state: State<'_, AppState>,
    data: CreateReport,
) -> Result<Report, String> {
    let conn = state.get_workspace_connection().await?;
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO reports (id, title, type, content, date, week_start, week_end, month, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        rusqlite::params![
            &id, &data.title, &data.r#type, &data.content, &data.date,
            &data.week_start, &data.week_end, &data.month, &now, &now
        ],
    ).map_err(|e| format!("Failed to create report: {}", e))?;

    // Update search index
    conn.execute(
        "INSERT OR REPLACE INTO search_index (entity_type, entity_id, title, content, tags) VALUES ('report', ?1, ?2, ?3, '')",
        rusqlite::params![&id, &data.title, &data.content],
    ).ok();

    Ok(Report {
        id, title: data.title, r#type: data.r#type, content: data.content,
        date: data.date, week_start: data.week_start, week_end: data.week_end,
        month: data.month,
        created_at: now.clone(), updated_at: now,
    })
}

#[tauri::command]
pub async fn update_report(
    state: State<'_, AppState>,
    id: String,
    data: UpdateReport,
) -> Result<Report, String> {
    let conn = state.get_workspace_connection().await?;
    let now = chrono::Utc::now().to_rfc3339();

    if let Some(ref v) = data.title {
        conn.execute("UPDATE reports SET title=?1, updated_at=?2 WHERE id=?3", rusqlite::params![v, &now, &id]).map_err(|e| e.to_string())?;
    }
    if let Some(ref v) = data.content {
        conn.execute("UPDATE reports SET content=?1, updated_at=?2 WHERE id=?3", rusqlite::params![v, &now, &id]).map_err(|e| e.to_string())?;
        conn.execute("UPDATE search_index SET content=?1 WHERE entity_type='report' AND entity_id=?2", rusqlite::params![v, &id]).ok();
    }
    if let Some(ref v) = data.date {
        conn.execute("UPDATE reports SET date=?1, updated_at=?2 WHERE id=?3", rusqlite::params![v, &now, &id]).map_err(|e| e.to_string())?;
    }
    if data.week_start.is_some() {
        conn.execute("UPDATE reports SET week_start=?1, updated_at=?2 WHERE id=?3", rusqlite::params![&data.week_start, &now, &id]).map_err(|e| e.to_string())?;
    }
    if data.week_end.is_some() {
        conn.execute("UPDATE reports SET week_end=?1, updated_at=?2 WHERE id=?3", rusqlite::params![&data.week_end, &now, &id]).map_err(|e| e.to_string())?;
    }
    if data.month.is_some() {
        conn.execute("UPDATE reports SET month=?1, updated_at=?2 WHERE id=?3", rusqlite::params![&data.month, &now, &id]).map_err(|e| e.to_string())?;
    }

    get_report(state, id).await
}

#[tauri::command]
pub async fn delete_report(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let conn = state.get_workspace_connection().await?;
    conn.execute("DELETE FROM search_index WHERE entity_type='report' AND entity_id=?1", rusqlite::params![&id]).ok();
    conn.execute("DELETE FROM reports WHERE id=?1", rusqlite::params![&id])
        .map_err(|e| format!("Failed to delete report: {}", e))?;
    Ok(())
}

