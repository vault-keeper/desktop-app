use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;
use crate::db::connection::AppState;
use crate::crypto::kdf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteGroup {
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
pub struct Note {
    pub id: String,
    pub title: String,
    pub content: String,
    pub is_encrypted: i32,
    pub encrypted_content: Option<String>,
    pub encryption_nonce: Option<String>,
    pub secondary_salt: Option<String>,
    pub group_id: Option<String>,
    pub sort_order: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteSummary {
    pub id: String,
    pub title: String,
    pub is_encrypted: i32,
    pub group_id: Option<String>,
    pub sort_order: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateNote {
    pub title: String,
    pub content: String,
    pub group_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateNote {
    pub title: Option<String>,
    pub content: Option<String>,
    /// None = not included (no change); Some(None) = clear group; Some(Some(id)) = set group
    #[serde(default)]
    pub group_id: Option<Option<String>>,
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

fn row_to_note(row: &rusqlite::Row) -> rusqlite::Result<Note> {
    Ok(Note {
        id: row.get(0)?,
        title: row.get(1)?,
        content: row.get(2)?,
        is_encrypted: row.get(3)?,
        encrypted_content: row.get(4)?,
        encryption_nonce: row.get(5)?,
        secondary_salt: row.get(6)?,
        group_id: row.get(7)?,
        sort_order: row.get(8)?,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
    })
}

fn row_to_group(row: &rusqlite::Row) -> rusqlite::Result<NoteGroup> {
    Ok(NoteGroup {
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
pub async fn list_notes(
    state: State<'_, AppState>,
    group_id: Option<String>,
) -> Result<Vec<Note>, String> {
    let conn = state.get_workspace_connection().await?;

    let mut notes = Vec::new();
    if let Some(ref gid) = group_id {
        let mut stmt = conn.prepare(
            "SELECT id, title, content, is_encrypted, encrypted_content, encryption_nonce, secondary_salt, group_id, sort_order, created_at, updated_at
             FROM notes WHERE group_id=?1 ORDER BY sort_order, updated_at DESC"
        ).map_err(|e| e.to_string())?;
        let rows = stmt.query_map(rusqlite::params![gid], row_to_note)
            .map_err(|e| e.to_string())?;
        for row in rows { if let Ok(n) = row { notes.push(n); } }
    } else {
        let mut stmt = conn.prepare(
            "SELECT id, title, content, is_encrypted, encrypted_content, encryption_nonce, secondary_salt, group_id, sort_order, created_at, updated_at
             FROM notes ORDER BY sort_order, updated_at DESC"
        ).map_err(|e| e.to_string())?;
        let rows = stmt.query_map([], row_to_note)
            .map_err(|e| e.to_string())?;
        for row in rows { if let Ok(n) = row { notes.push(n); } }
    }

    Ok(notes)
}

#[tauri::command]
pub async fn get_note(
    state: State<'_, AppState>,
    id: String,
) -> Result<Note, String> {
    let conn = state.get_workspace_connection().await?;
    conn.query_row(
        "SELECT id, title, content, is_encrypted, encrypted_content, encryption_nonce, secondary_salt, group_id, sort_order, created_at, updated_at
         FROM notes WHERE id=?1",
        rusqlite::params![&id],
        row_to_note,
    ).map_err(|e| format!("Note not found: {}", e))
}

#[tauri::command]
pub async fn create_note(
    state: State<'_, AppState>,
    data: CreateNote,
) -> Result<Note, String> {
    let conn = state.get_workspace_connection().await?;
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO notes (id, title, content, is_encrypted, group_id, sort_order, created_at, updated_at)
         VALUES (?1, ?2, ?3, 0, ?4, 0, ?5, ?6)",
        rusqlite::params![&id, &data.title, &data.content, &data.group_id, &now, &now],
    ).map_err(|e| format!("Failed to create note: {}", e))?;

    // Update search index
    conn.execute(
        "INSERT OR REPLACE INTO search_index (entity_type, entity_id, title, content, tags)
         VALUES ('note', ?1, ?2, ?3, '')",
        rusqlite::params![&id, &data.title, &data.content],
    ).ok();

    Ok(Note {
        id,
        title: data.title,
        content: data.content,
        is_encrypted: 0,
        encrypted_content: None,
        encryption_nonce: None,
        secondary_salt: None,
        group_id: data.group_id,
        sort_order: 0,
        created_at: now.clone(),
        updated_at: now,
    })
}

#[tauri::command]
pub async fn update_note(
    state: State<'_, AppState>,
    id: String,
    data: UpdateNote,
) -> Result<Note, String> {
    let conn = state.get_workspace_connection().await?;
    let now = chrono::Utc::now().to_rfc3339();

    if let Some(ref v) = data.title {
        conn.execute("UPDATE notes SET title=?1, updated_at=?2 WHERE id=?3", rusqlite::params![v, &now, &id]).map_err(|e| e.to_string())?;
    }
    if let Some(ref v) = data.content {
        // Guard: never write plaintext into an encrypted note's content field.
        let is_encrypted: i32 = conn.query_row(
            "SELECT is_encrypted FROM notes WHERE id=?1",
            rusqlite::params![&id],
            |row| row.get(0),
        ).unwrap_or(0);
        if is_encrypted == 0 {
            conn.execute("UPDATE notes SET content=?1, updated_at=?2 WHERE id=?3",
                rusqlite::params![v, &now, &id]).map_err(|e| e.to_string())?;
            conn.execute("UPDATE search_index SET content=?1 WHERE entity_type='note' AND entity_id=?2",
                rusqlite::params![v, &id]).ok();
        }
    }
    if let Some(group_id) = data.group_id {
        conn.execute("UPDATE notes SET group_id=?1, updated_at=?2 WHERE id=?3", rusqlite::params![group_id, &now, &id]).map_err(|e| e.to_string())?;
    }
    if let Some(v) = data.sort_order {
        conn.execute("UPDATE notes SET sort_order=?1, updated_at=?2 WHERE id=?3", rusqlite::params![v, &now, &id]).map_err(|e| e.to_string())?;
    }

    get_note(state, id).await
}

#[tauri::command]
pub async fn delete_note(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let conn = state.get_workspace_connection().await?;
    conn.execute("DELETE FROM search_index WHERE entity_type='note' AND entity_id=?1", rusqlite::params![&id]).ok();
    conn.execute("DELETE FROM notes WHERE id=?1", rusqlite::params![&id])
        .map_err(|e| format!("Failed to delete note: {}", e))?;
    Ok(())
}

/// Encrypt a note with a secondary password (distinct from master password).
#[tauri::command]
pub async fn encrypt_note(
    state: State<'_, AppState>,
    id: String,
    secondary_password: String,
) -> Result<(), String> {
    let note = get_note(state.clone(), id.clone()).await?;
    if note.is_encrypted != 0 {
        return Err("Note is already encrypted".to_string());
    }

    // Derive secondary key
    let secondary_salt = kdf::generate_salt();
    let secondary_key = kdf::derive_key(secondary_password.as_bytes(), &secondary_salt)
        .map_err(|e| format!("Key derivation failed: {}", e))?;

    // Encrypt content
    let (encrypted_bytes, nonce_bytes) = crate::crypto::aes::encrypt(note.content.as_bytes(), &secondary_key)
        .map_err(|e| format!("Encryption failed: {}", e))?;

    let conn = state.get_workspace_connection().await?;
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "UPDATE notes SET is_encrypted=1, encrypted_content=?1, encryption_nonce=?2, secondary_salt=?3, content='', updated_at=?4 WHERE id=?5",
        rusqlite::params![
            hex::encode(&encrypted_bytes),
            hex::encode(&nonce_bytes),
            hex::encode(&secondary_salt),
            &now,
            &id,
        ],
    ).map_err(|e| format!("Failed to encrypt note: {}", e))?;

    // Remove from search index (content is now encrypted)
    conn.execute("DELETE FROM search_index WHERE entity_type='note' AND entity_id=?1", rusqlite::params![&id]).ok();

    Ok(())
}

/// Decrypt a note temporarily using the secondary password.
/// Returns the note with decrypted content without persisting.
#[tauri::command]
pub async fn decrypt_note(
    state: State<'_, AppState>,
    id: String,
    secondary_password: String,
) -> Result<Note, String> {
    let note = get_note(state.clone(), id.clone()).await?;
    if note.is_encrypted == 0 {
        return Ok(note);
    }

    let secondary_salt_hex = note.secondary_salt.as_deref().ok_or("Missing secondary salt")?;
    let enc_content_hex = note.encrypted_content.as_deref().ok_or("Missing encrypted content")?;
    let nonce_hex = note.encryption_nonce.as_deref().ok_or("Missing encryption nonce")?;

    let secondary_salt = hex::decode(secondary_salt_hex).map_err(|e| format!("Invalid salt: {}", e))?;
    let secondary_key = kdf::derive_key(secondary_password.as_bytes(), &secondary_salt)
        .map_err(|e| format!("Key derivation failed: {}", e))?;

    let encrypted_bytes = hex::decode(enc_content_hex).map_err(|e| format!("Invalid ciphertext: {}", e))?;
    let nonce_bytes = hex::decode(nonce_hex).map_err(|e| format!("Invalid nonce: {}", e))?;

    let decrypted = crate::crypto::aes::decrypt(&encrypted_bytes, &nonce_bytes, &secondary_key)
        .map_err(|_| "Incorrect secondary password".to_string())?;

    let content = String::from_utf8(decrypted).map_err(|e| format!("Invalid content encoding: {}", e))?;

    Ok(Note {
        content,
        is_encrypted: 0, // present as decrypted
        encrypted_content: None,
        encryption_nonce: None,
        secondary_salt: None,
        ..note
    })
}

/// Permanently decrypt a note (requires secondary password).
#[tauri::command]
pub async fn permanently_decrypt_note(
    state: State<'_, AppState>,
    id: String,
    secondary_password: String,
) -> Result<Note, String> {
    let decrypted = decrypt_note(state.clone(), id.clone(), secondary_password).await?;
    let conn = state.get_workspace_connection().await?;
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "UPDATE notes SET is_encrypted=0, content=?1, encrypted_content=NULL, encryption_nonce=NULL, secondary_salt=NULL, updated_at=?2 WHERE id=?3",
        rusqlite::params![&decrypted.content, &now, &id],
    ).map_err(|e| format!("Failed to decrypt note: {}", e))?;

    // Re-add to search index
    conn.execute(
        "INSERT OR REPLACE INTO search_index (entity_type, entity_id, title, content, tags) VALUES ('note', ?1, ?2, ?3, '')",
        rusqlite::params![&id, &decrypted.title, &decrypted.content],
    ).ok();

    get_note(state, id).await
}

#[tauri::command]
pub async fn list_note_groups(
    state: State<'_, AppState>,
) -> Result<Vec<NoteGroup>, String> {
    let conn = state.get_workspace_connection().await?;
    let mut stmt = conn.prepare(
        "SELECT id, name, icon, color, parent_id, sort_order, created_at, updated_at
         FROM note_groups ORDER BY sort_order, name"
    ).map_err(|e| e.to_string())?;
    let groups = stmt.query_map([], row_to_group)
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok()).collect();
    Ok(groups)
}

#[tauri::command]
pub async fn create_note_group(
    state: State<'_, AppState>,
    data: CreateGroupPayload,
) -> Result<NoteGroup, String> {
    let conn = state.get_workspace_connection().await?;
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO note_groups (id, name, icon, color, parent_id, sort_order, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6, ?7)",
        rusqlite::params![&id, &data.name, &data.icon, &data.color, &data.parent_id, &now, &now],
    ).map_err(|e| format!("Failed to create note group: {}", e))?;

    Ok(NoteGroup {
        id, name: data.name, icon: data.icon, color: data.color,
        parent_id: data.parent_id, sort_order: 0,
        created_at: now.clone(), updated_at: now,
    })
}

#[tauri::command]
pub async fn update_note_group(
    state: State<'_, AppState>,
    id: String,
    data: UpdateGroupPayload,
) -> Result<NoteGroup, String> {
    let conn = state.get_workspace_connection().await?;
    let now = chrono::Utc::now().to_rfc3339();

    if let Some(ref v) = data.name {
        conn.execute("UPDATE note_groups SET name=?1, updated_at=?2 WHERE id=?3",
            rusqlite::params![v, &now, &id]).map_err(|e| e.to_string())?;
    }
    if data.icon.is_some() {
        conn.execute("UPDATE note_groups SET icon=?1, updated_at=?2 WHERE id=?3",
            rusqlite::params![&data.icon, &now, &id]).map_err(|e| e.to_string())?;
    }

    conn.query_row(
        "SELECT id, name, icon, color, parent_id, sort_order, created_at, updated_at FROM note_groups WHERE id=?1",
        rusqlite::params![&id],
        |row| Ok(NoteGroup {
            id: row.get(0)?, name: row.get(1)?, icon: row.get(2)?,
            color: row.get(3)?, parent_id: row.get(4)?,
            sort_order: row.get(5)?, created_at: row.get(6)?, updated_at: row.get(7)?,
        }),
    ).map_err(|e| format!("Group not found: {}", e))
}

#[tauri::command]
pub async fn delete_note_group(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let conn = state.get_workspace_connection().await?;
    conn.execute("DELETE FROM note_groups WHERE id=?1", rusqlite::params![&id])
        .map_err(|e| format!("Failed to delete note group: {}", e))?;
    Ok(())
}

/// Save changes to an encrypted note: updates title/group and re-encrypts the
/// new content with a fresh salt.  Verifies the secondary password before writing.
#[tauri::command]
pub async fn save_and_reencrypt_note(
    state: State<'_, AppState>,
    id: String,
    title: String,
    content: String,
    group_id: Option<String>,
    secondary_password: String,
) -> Result<Note, String> {
    let note = get_note(state.clone(), id.clone()).await?;
    if note.is_encrypted == 0 {
        return Err("Note is not encrypted".to_string());
    }

    // Verify the secondary password is correct before re-encrypting.
    decrypt_note(state.clone(), id.clone(), secondary_password.clone()).await
        .map_err(|_| "Incorrect secondary password".to_string())?;

    // Derive a fresh salt so each save produces independent ciphertext.
    let secondary_salt = kdf::generate_salt();
    let secondary_key = kdf::derive_key(secondary_password.as_bytes(), &secondary_salt)
        .map_err(|e| format!("Key derivation failed: {}", e))?;
    let (encrypted_bytes, nonce_bytes) = crate::crypto::aes::encrypt(content.as_bytes(), &secondary_key)
        .map_err(|e| format!("Encryption failed: {}", e))?;

    let conn = state.get_workspace_connection().await?;
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "UPDATE notes SET title=?1, encrypted_content=?2, encryption_nonce=?3, \
         secondary_salt=?4, content='', group_id=?5, updated_at=?6 WHERE id=?7",
        rusqlite::params![
            &title,
            hex::encode(&encrypted_bytes),
            hex::encode(&nonce_bytes),
            hex::encode(&secondary_salt),
            &group_id,
            &now,
            &id,
        ],
    ).map_err(|e| format!("Failed to save encrypted note: {}", e))?;

    get_note(state, id).await
}
