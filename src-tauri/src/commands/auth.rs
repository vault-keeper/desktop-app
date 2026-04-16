use serde::Serialize;
use tauri::State;

use crate::crypto::{kdf, aes};
use crate::db::connection::AppState;

#[derive(Serialize)]
pub struct SetupResult {
    pub mnemonic: Vec<String>,
}

/// Check if the vault has been initialized (master password has been set).
#[tauri::command]
pub async fn is_setup_complete(state: State<'_, AppState>) -> Result<bool, String> {
    let conn = state.get_meta_connection().await?;
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM master_key_verify", [], |row| row.get(0))
        .unwrap_or(0);
    Ok(count > 0)
}

/// Set up the master password for the first time.
#[tauri::command]
pub async fn setup_master_password(
    state: State<'_, AppState>,
    password: String,
) -> Result<(), String> {
    let salt = kdf::generate_salt();
    let master_key = kdf::derive_key(password.as_bytes(), &salt)
        .map_err(|e| format!("Key derivation failed: {}", e))?;

    let mut verify_input = master_key.clone();
    verify_input.extend_from_slice(b"verify");
    let verify_hash = kdf::derive_key(&verify_input, &salt)
        .map_err(|e| format!("Failed to generate verify hash: {}", e))?;

    let conn = state.get_meta_connection().await?;
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT OR REPLACE INTO master_key_verify
         (id, verify_hash, salt, backup_salt, encrypted_master, master_nonce, created_at)
         VALUES (1, ?1, ?2, '', '', '', ?3)",
        rusqlite::params![
            hex::encode(&verify_hash),
            hex::encode(&salt),
            &now,
        ],
    ).map_err(|e| format!("Failed to store master key data: {}", e))?;

    state.set_master_key(master_key);
    Ok(())
}

/// Verify the master password and unlock the vault.
#[tauri::command]
pub async fn verify_master_password(
    state: State<'_, AppState>,
    password: String,
) -> Result<bool, String> {
    let conn = state.get_meta_connection().await?;

    let (verify_hash_hex, salt_hex): (String, String) = conn
        .query_row(
            "SELECT verify_hash, salt FROM master_key_verify WHERE id = 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|_| "Vault not initialized. Please set up your master password.".to_string())?;

    let salt = hex::decode(&salt_hex).map_err(|e| format!("Invalid salt: {}", e))?;
    let derived_key = kdf::derive_key(password.as_bytes(), &salt)
        .map_err(|e| format!("Key derivation failed: {}", e))?;

    let mut verify_input = derived_key.clone();
    verify_input.extend_from_slice(b"verify");
    let expected_hash = kdf::derive_key(&verify_input, &salt)
        .map_err(|e| format!("Verify hash generation failed: {}", e))?;

    let is_valid = hex::encode(&expected_hash) == verify_hash_hex;

    if is_valid {
        state.set_master_key(derived_key);
    }

    Ok(is_valid)
}

/// Lock the vault (clear master key from memory).
#[tauri::command]
pub async fn lock_vault(state: State<'_, AppState>) -> Result<(), String> {
    state.clear_master_key();
    Ok(())
}

/// Check if the vault is currently unlocked.
#[tauri::command]
pub async fn is_vault_unlocked(state: State<'_, AppState>) -> Result<bool, String> {
    Ok(state.is_unlocked())
}

/// Change the master password (requires current password to be verified).
#[tauri::command]
pub async fn change_master_password(
    state: State<'_, AppState>,
    current_password: String,
    new_password: String,
) -> Result<(), String> {
    // 1. Verify current password (also refreshes master key in memory)
    let valid = verify_master_password(state.clone(), current_password).await?;
    if !valid {
        return Err("Current password is incorrect".to_string());
    }

    let old_master_key = state.get_master_key().ok_or("Vault not unlocked")?;
    let old_key_hex = hex::encode(&old_master_key);

    // 2. Derive new master key
    let new_salt = kdf::generate_salt();
    let new_master_key = kdf::derive_key(new_password.as_bytes(), &new_salt)
        .map_err(|e| format!("Key derivation failed: {}", e))?;
    let new_key_hex = hex::encode(&new_master_key);

    // 3. Re-encrypt all workspace databases BEFORE updating meta db.
    state.rekey_all_workspace_dbs(&old_key_hex, &new_key_hex).await?;

    // 4. Update meta db
    let mut verify_input = new_master_key.clone();
    verify_input.extend_from_slice(b"verify");
    let verify_hash = kdf::derive_key(&verify_input, &new_salt)
        .map_err(|e| format!("Verify hash generation failed: {}", e))?;

    let conn = state.get_meta_connection().await?;
    conn.execute(
        "UPDATE master_key_verify SET verify_hash=?1, salt=?2, backup_salt='', encrypted_master='', master_nonce='' WHERE id=1",
        rusqlite::params![
            hex::encode(&verify_hash),
            hex::encode(&new_salt),
        ],
    ).map_err(|e| format!("Failed to update master key: {}", e))?;

    // 5. Update in-memory key
    state.set_master_key(new_master_key);
    Ok(())
}
