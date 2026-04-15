use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;
use rand::Rng;
use crate::db::connection::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountGroup {
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
pub struct Account {
    pub id: String,
    pub title: String,
    pub url: Option<String>,
    pub username: Option<String>,
    pub password: String,
    pub notes: Option<String>,
    pub icon_url: Option<String>,
    pub group_id: Option<String>,
    pub favorite: i32,
    pub sort_order: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateAccount {
    pub title: String,
    pub url: Option<String>,
    pub username: Option<String>,
    pub password: String,
    pub notes: Option<String>,
    pub icon_url: Option<String>,
    pub group_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAccount {
    pub title: Option<String>,
    pub url: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub notes: Option<String>,
    pub icon_url: Option<String>,
    /// None = not included (no change); Some(None) = clear group; Some(Some(id)) = set group
    #[serde(default)]
    pub group_id: Option<Option<String>>,
    pub favorite: Option<i32>,
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

/// Encrypt a plaintext field using AES-256-GCM with the master key.
/// Returns format: "hex_nonce:hex_ciphertext"
fn encrypt_field(value: &str, key: &[u8]) -> Result<String, String> {
    let (ciphertext, nonce) = crate::crypto::aes::encrypt(value.as_bytes(), key)?;
    Ok(format!("{}:{}", hex::encode(&nonce), hex::encode(&ciphertext)))
}

/// Decrypt a field previously encrypted with encrypt_field.
fn decrypt_field(encoded: &str, key: &[u8]) -> Result<String, String> {
    if encoded.is_empty() {
        return Ok(String::new());
    }
    let mut parts = encoded.splitn(2, ':');
    let nonce_hex = parts.next().ok_or("Missing nonce")?;
    let ct_hex = parts.next().ok_or("Missing ciphertext")?;
    let nonce = hex::decode(nonce_hex).map_err(|e| format!("Invalid nonce: {}", e))?;
    let ciphertext = hex::decode(ct_hex).map_err(|e| format!("Invalid ciphertext: {}", e))?;
    let plaintext = crate::crypto::aes::decrypt(&ciphertext, &nonce, key)?;
    String::from_utf8(plaintext).map_err(|e| format!("Invalid UTF-8: {}", e))
}

fn encrypt_opt(value: &Option<String>, key: &[u8]) -> Result<Option<String>, String> {
    match value {
        Some(v) if !v.is_empty() => Ok(Some(encrypt_field(v, key)?)),
        _ => Ok(None),
    }
}

fn decrypt_opt(value: &Option<String>, key: &[u8]) -> Result<Option<String>, String> {
    match value {
        Some(v) if !v.is_empty() => Ok(Some(decrypt_field(v, key)?)),
        _ => Ok(None),
    }
}

fn row_to_group(row: &rusqlite::Row) -> rusqlite::Result<AccountGroup> {
    Ok(AccountGroup {
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

fn decrypt_account_row(row: &rusqlite::Row, key: &[u8]) -> Result<Account, String> {
    let username_enc: Option<String> = row.get(3).unwrap_or(None);
    let password_enc: String = row.get(4).unwrap_or_default();
    let notes_enc: Option<String> = row.get(5).unwrap_or(None);

    Ok(Account {
        id: row.get(0).map_err(|e| e.to_string())?,
        title: row.get(1).map_err(|e| e.to_string())?,
        url: row.get(2).unwrap_or(None),
        username: decrypt_opt(&username_enc, key)?,
        password: decrypt_field(&password_enc, key).unwrap_or(password_enc),
        notes: decrypt_opt(&notes_enc, key)?,
        icon_url: row.get(6).unwrap_or(None),
        group_id: row.get(7).unwrap_or(None),
        favorite: row.get(8).unwrap_or(0),
        sort_order: row.get(9).unwrap_or(0),
        created_at: row.get(10).map_err(|e| e.to_string())?,
        updated_at: row.get(11).map_err(|e| e.to_string())?,
    })
}

#[tauri::command]
pub async fn list_accounts(
    state: State<'_, AppState>,
    group_id: Option<String>,
) -> Result<Vec<Account>, String> {
    let master_key = state.get_master_key().ok_or("Vault is locked")?;
    let conn = state.get_workspace_connection().await?;

    let sql = "SELECT id, title, url, username, password, notes, icon_url, group_id, favorite, sort_order, created_at, updated_at FROM accounts";
    let mut rows: Vec<Account> = Vec::new();
    if let Some(ref gid) = group_id {
        let mut stmt = conn.prepare(&format!("{} WHERE group_id=?1 ORDER BY sort_order, title", sql))
            .map_err(|e| e.to_string())?;
        let mapped = stmt.query_map(rusqlite::params![gid], |row| {
            Ok(decrypt_account_row(row, &master_key))
        }).map_err(|e| e.to_string())?;
        for r in mapped { if let Ok(Ok(acc)) = r { rows.push(acc); } }
    } else {
        let mut stmt = conn.prepare(&format!("{} ORDER BY sort_order, title", sql))
            .map_err(|e| e.to_string())?;
        let mapped = stmt.query_map([], |row| {
            Ok(decrypt_account_row(row, &master_key))
        }).map_err(|e| e.to_string())?;
        for r in mapped { if let Ok(Ok(acc)) = r { rows.push(acc); } }
    }

    Ok(rows)
}

#[tauri::command]
pub async fn get_account(
    state: State<'_, AppState>,
    id: String,
) -> Result<Account, String> {
    let master_key = state.get_master_key().ok_or("Vault is locked")?;
    let conn = state.get_workspace_connection().await?;

    conn.query_row(
        "SELECT id, title, url, username, password, notes, icon_url, group_id, favorite, sort_order, created_at, updated_at
         FROM accounts WHERE id=?1",
        rusqlite::params![&id],
        |row| Ok(decrypt_account_row(row, &master_key)),
    ).map_err(|e| format!("Account not found: {}", e))?.map_err(|e| e)
}

#[tauri::command]
pub async fn create_account(
    state: State<'_, AppState>,
    data: CreateAccount,
) -> Result<Account, String> {
    let master_key = state.get_master_key().ok_or("Vault is locked")?;
    let conn = state.get_workspace_connection().await?;
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    // Encrypt sensitive fields
    let enc_password = encrypt_field(&data.password, &master_key)?;
    let enc_username = encrypt_opt(&data.username, &master_key)?;
    let enc_notes = encrypt_opt(&data.notes, &master_key)?;

    conn.execute(
        "INSERT INTO accounts (id, title, url, username, password, notes, icon_url, group_id, favorite, sort_order, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 0, 0, ?9, ?10)",
        rusqlite::params![
            &id, &data.title, &data.url, &enc_username, &enc_password,
            &enc_notes, &data.icon_url, &data.group_id, &now, &now
        ],
    ).map_err(|e| format!("Failed to create account: {}", e))?;

    // Update search index
    conn.execute(
        "INSERT OR REPLACE INTO search_index (entity_type, entity_id, title, content, tags)
         VALUES ('account', ?1, ?2, ?3, '')",
        rusqlite::params![&id, &data.title, data.url.as_deref().unwrap_or("")],
    ).ok();

    Ok(Account {
        id,
        title: data.title,
        url: data.url,
        username: data.username,
        password: data.password,
        notes: data.notes,
        icon_url: data.icon_url,
        group_id: data.group_id,
        favorite: 0,
        sort_order: 0,
        created_at: now.clone(),
        updated_at: now,
    })
}

#[tauri::command]
pub async fn update_account(
    state: State<'_, AppState>,
    id: String,
    data: UpdateAccount,
) -> Result<Account, String> {
    let master_key = state.get_master_key().ok_or("Vault is locked")?;
    let conn = state.get_workspace_connection().await?;
    let now = chrono::Utc::now().to_rfc3339();

    if let Some(ref v) = data.title {
        conn.execute("UPDATE accounts SET title=?1, updated_at=?2 WHERE id=?3", rusqlite::params![v, &now, &id]).map_err(|e| e.to_string())?;
    }
    if data.url.is_some() {
        conn.execute("UPDATE accounts SET url=?1, updated_at=?2 WHERE id=?3", rusqlite::params![&data.url, &now, &id]).map_err(|e| e.to_string())?;
    }
    if let Some(ref v) = data.username {
        let enc = encrypt_field(v, &master_key)?;
        conn.execute("UPDATE accounts SET username=?1, updated_at=?2 WHERE id=?3", rusqlite::params![&enc, &now, &id]).map_err(|e| e.to_string())?;
    }
    if let Some(ref v) = data.password {
        let enc = encrypt_field(v, &master_key)?;
        conn.execute("UPDATE accounts SET password=?1, updated_at=?2 WHERE id=?3", rusqlite::params![&enc, &now, &id]).map_err(|e| e.to_string())?;
    }
    if data.notes.is_some() {
        let enc_notes = encrypt_opt(&data.notes, &master_key)?;
        conn.execute("UPDATE accounts SET notes=?1, updated_at=?2 WHERE id=?3", rusqlite::params![&enc_notes, &now, &id]).map_err(|e| e.to_string())?;
    }
    if let Some(group_id) = data.group_id {
        conn.execute("UPDATE accounts SET group_id=?1, updated_at=?2 WHERE id=?3", rusqlite::params![group_id, &now, &id]).map_err(|e| e.to_string())?;
    }
    if let Some(v) = data.favorite {
        conn.execute("UPDATE accounts SET favorite=?1, updated_at=?2 WHERE id=?3", rusqlite::params![v, &now, &id]).map_err(|e| e.to_string())?;
    }
    if let Some(v) = data.sort_order {
        conn.execute("UPDATE accounts SET sort_order=?1, updated_at=?2 WHERE id=?3", rusqlite::params![v, &now, &id]).map_err(|e| e.to_string())?;
    }

    get_account(state, id).await
}

#[tauri::command]
pub async fn delete_account(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let conn = state.get_workspace_connection().await?;
    conn.execute("DELETE FROM search_index WHERE entity_type='account' AND entity_id=?1", rusqlite::params![&id]).ok();
    conn.execute("DELETE FROM accounts WHERE id=?1", rusqlite::params![&id])
        .map_err(|e| format!("Failed to delete account: {}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn generate_password(
    length: Option<u32>,
    _state: State<'_, AppState>,
) -> Result<String, String> {
    let len = length.unwrap_or(16) as usize;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*-_=+";
    let mut rng = rand::thread_rng();
    let password: String = (0..len)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();
    Ok(password)
}

#[tauri::command]
pub async fn list_account_groups(
    state: State<'_, AppState>,
) -> Result<Vec<AccountGroup>, String> {
    let conn = state.get_workspace_connection().await?;
    let mut stmt = conn.prepare(
        "SELECT id, name, icon, color, parent_id, sort_order, created_at, updated_at
         FROM account_groups ORDER BY sort_order, name"
    ).map_err(|e| e.to_string())?;
    let groups = stmt.query_map([], row_to_group)
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok()).collect();
    Ok(groups)
}

#[tauri::command]
pub async fn create_account_group(
    state: State<'_, AppState>,
    data: CreateGroupPayload,
) -> Result<AccountGroup, String> {
    let conn = state.get_workspace_connection().await?;
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO account_groups (id, name, icon, color, parent_id, sort_order, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6, ?7)",
        rusqlite::params![&id, &data.name, &data.icon, &data.color, &data.parent_id, &now, &now],
    ).map_err(|e| format!("Failed to create account group: {}", e))?;

    Ok(AccountGroup {
        id, name: data.name, icon: data.icon, color: data.color,
        parent_id: data.parent_id, sort_order: 0,
        created_at: now.clone(), updated_at: now,
    })
}

#[tauri::command]
pub async fn update_account_group(
    state: State<'_, AppState>,
    id: String,
    data: UpdateGroupPayload,
) -> Result<AccountGroup, String> {
    let conn = state.get_workspace_connection().await?;
    let now = chrono::Utc::now().to_rfc3339();

    if let Some(ref v) = data.name {
        conn.execute("UPDATE account_groups SET name=?1, updated_at=?2 WHERE id=?3",
            rusqlite::params![v, &now, &id]).map_err(|e| e.to_string())?;
    }
    if data.icon.is_some() {
        conn.execute("UPDATE account_groups SET icon=?1, updated_at=?2 WHERE id=?3",
            rusqlite::params![&data.icon, &now, &id]).map_err(|e| e.to_string())?;
    }

    conn.query_row(
        "SELECT id, name, icon, color, parent_id, sort_order, created_at, updated_at FROM account_groups WHERE id=?1",
        rusqlite::params![&id],
        |row| Ok(AccountGroup {
            id: row.get(0)?, name: row.get(1)?, icon: row.get(2)?,
            color: row.get(3)?, parent_id: row.get(4)?,
            sort_order: row.get(5)?, created_at: row.get(6)?, updated_at: row.get(7)?,
        }),
    ).map_err(|e| format!("Group not found: {}", e))
}

#[tauri::command]
pub async fn delete_account_group(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let conn = state.get_workspace_connection().await?;
    conn.execute("DELETE FROM account_groups WHERE id=?1", rusqlite::params![&id])
        .map_err(|e| format!("Failed to delete account group: {}", e))?;
    Ok(())
}
