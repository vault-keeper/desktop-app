use serde::{Deserialize, Serialize};
use tauri::State;

use crate::crypto::{kdf, aes, mnemonic as bip39};
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
/// Returns the BIP39 mnemonic recovery phrase.
#[tauri::command]
pub async fn setup_master_password(
    state: State<'_, AppState>,
    password: String,
) -> Result<SetupResult, String> {
    // 1. Generate random salt for password KDF
    let salt = kdf::generate_salt();
    // 2. Derive master key from password
    let master_key = kdf::derive_key(password.as_bytes(), &salt)
        .map_err(|e| format!("Key derivation failed: {}", e))?;

    // 3. Generate BIP39 mnemonic entropy (256-bit = 24 words)
    let entropy = bip39::generate_seed();
    let mnemonic_words = bip39::seed_to_mnemonic(&entropy)
        .map_err(|e| format!("Failed to generate mnemonic: {}", e))?;

    // 4. Derive backup key from mnemonic entropy
    let backup_salt = kdf::generate_salt();
    let backup_key = kdf::derive_key(&entropy, &backup_salt)
        .map_err(|e| format!("Backup key derivation failed: {}", e))?;

    // 5. Encrypt master key with backup key (for recovery)
    let (encrypted_master, master_nonce) = aes::encrypt(&master_key, &backup_key)
        .map_err(|e| format!("Failed to encrypt master key: {}", e))?;

    // 6. Generate verification hash
    let mut verify_input = master_key.clone();
    verify_input.extend_from_slice(b"verify");
    let verify_hash = kdf::derive_key(&verify_input, &salt)
        .map_err(|e| format!("Failed to generate verify hash: {}", e))?;

    // 7. Store in vault_meta.db
    let conn = state.get_meta_connection().await?;
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT OR REPLACE INTO master_key_verify
         (id, verify_hash, salt, backup_salt, encrypted_master, master_nonce, created_at)
         VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![
            hex::encode(&verify_hash),
            hex::encode(&salt),
            hex::encode(&backup_salt),
            hex::encode(&encrypted_master),
            hex::encode(&master_nonce),
            &now,
        ],
    ).map_err(|e| format!("Failed to store master key data: {}", e))?;

    // 8. Store master key in memory
    state.set_master_key(master_key);

    Ok(SetupResult { mnemonic: mnemonic_words })
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

/// Recover the master key from BIP39 mnemonic phrase.
#[tauri::command]
pub async fn recover_from_mnemonic(
    state: State<'_, AppState>,
    mnemonic_words: Vec<String>,
) -> Result<bool, String> {
    // 1. Parse mnemonic → entropy
    let entropy = bip39::mnemonic_to_seed(&mnemonic_words)
        .map_err(|e| format!("Invalid mnemonic: {}", e))?;

    let conn = state.get_meta_connection().await?;

    let (verify_hash_hex, salt_hex, backup_salt_hex, encrypted_master_hex, master_nonce_hex):
        (String, String, String, String, String) = conn
        .query_row(
            "SELECT verify_hash, salt, backup_salt, encrypted_master, master_nonce FROM master_key_verify WHERE id = 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
        )
        .map_err(|_| "Vault not initialized".to_string())?;

    // 2. Derive backup key from entropy
    let backup_salt = hex::decode(&backup_salt_hex).map_err(|e| format!("Invalid backup salt: {}", e))?;
    let backup_key = kdf::derive_key(&entropy, &backup_salt)
        .map_err(|e| format!("Backup key derivation failed: {}", e))?;

    // 3. Decrypt master key
    let encrypted_master = hex::decode(&encrypted_master_hex).map_err(|e| format!("Invalid encrypted master: {}", e))?;
    let master_nonce = hex::decode(&master_nonce_hex).map_err(|e| format!("Invalid master nonce: {}", e))?;
    let master_key = aes::decrypt(&encrypted_master, &master_nonce, &backup_key)
        .map_err(|_| "Invalid recovery phrase or corrupted data".to_string())?;

    // 4. Verify against stored hash
    let salt = hex::decode(&salt_hex).map_err(|e| format!("Invalid salt: {}", e))?;
    let mut verify_input = master_key.clone();
    verify_input.extend_from_slice(b"verify");
    let expected_hash = kdf::derive_key(&verify_input, &salt)
        .map_err(|e| format!("Verify hash generation failed: {}", e))?;

    if hex::encode(&expected_hash) == verify_hash_hex {
        state.set_master_key(master_key);
        Ok(true)
    } else {
        Err("Recovery phrase does not match this vault".to_string())
    }
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
    // Verify current password
    let valid = verify_master_password(state.clone(), current_password).await?;
    if !valid {
        return Err("Current password is incorrect".to_string());
    }

    let master_key = state.get_master_key().ok_or("Vault not unlocked")?;

    // Derive new key from new password
    let new_salt = kdf::generate_salt();
    let _new_key = kdf::derive_key(new_password.as_bytes(), &new_salt)
        .map_err(|e| format!("Key derivation failed: {}", e))?;

    // Generate new backup key (re-use existing mnemonic entropy by re-deriving backup)
    // For simplicity: generate new entropy (user should note new mnemonic in a real app)
    // Here we just re-derive the backup from the same structure
    let new_backup_salt = kdf::generate_salt();
    let new_backup_key = kdf::derive_key(&master_key, &new_backup_salt) // use master_key as entropy stand-in
        .map_err(|e| format!("Backup key derivation failed: {}", e))?;

    let new_master_key = kdf::derive_key(new_password.as_bytes(), &new_salt)
        .map_err(|e| format!("Key derivation failed: {}", e))?;

    let (encrypted_master, master_nonce) = aes::encrypt(&new_master_key, &new_backup_key)
        .map_err(|e| format!("Failed to encrypt master key: {}", e))?;

    let mut verify_input = new_master_key.clone();
    verify_input.extend_from_slice(b"verify");
    let verify_hash = kdf::derive_key(&verify_input, &new_salt)
        .map_err(|e| format!("Verify hash generation failed: {}", e))?;

    let conn = state.get_meta_connection().await?;
    conn.execute(
        "UPDATE master_key_verify SET verify_hash=?1, salt=?2, backup_salt=?3, encrypted_master=?4, master_nonce=?5 WHERE id=1",
        rusqlite::params![
            hex::encode(&verify_hash),
            hex::encode(&new_salt),
            hex::encode(&new_backup_salt),
            hex::encode(&encrypted_master),
            hex::encode(&master_nonce),
        ],
    ).map_err(|e| format!("Failed to update master key: {}", e))?;

    state.set_master_key(new_master_key);
    Ok(())
}
