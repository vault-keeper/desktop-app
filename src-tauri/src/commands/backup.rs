use std::io::{Read, Write};
use serde::{Deserialize, Serialize};
use tauri::State;
use crate::db::connection::AppState;
use crate::crypto::kdf;

#[derive(Serialize, Deserialize)]
struct WorkspaceEntry {
    name: String,
    icon: Option<String>,
    color: Option<String>,
    db_file: String,
    sort_order: i32,
}

#[derive(Serialize, Deserialize)]
struct BackupManifest {
    version: u32,
    app_version: String,
    exported_at: String,
    export_key_salt: String,
    workspaces: Vec<WorkspaceEntry>,
}

fn open_sqlcipher(path: &std::path::Path, key_hex: &str) -> Result<rusqlite::Connection, String> {
    let conn = rusqlite::Connection::open(path).map_err(|e| e.to_string())?;
    conn.execute_batch(&format!(
        "PRAGMA key = \"x'{key_hex}'\";\nPRAGMA cipher_page_size = 4096;"
    )).map_err(|e| e.to_string())?;
    conn.execute_batch("SELECT count(*) FROM sqlite_master;")
        .map_err(|_| "Key verification failed".to_string())?;
    Ok(conn)
}

#[tauri::command]
pub async fn export_vault(
    state: State<'_, AppState>,
    export_password: String,
    dest_path: String,
) -> Result<(), String> {
    let master_key = state.get_master_key().ok_or("Vault is locked")?;
    let master_key_hex = hex::encode(&master_key);

    // Derive export key with a fresh random salt
    let salt = kdf::generate_salt();
    let export_key = kdf::derive_key(export_password.as_bytes(), &salt)
        .map_err(|e| format!("Key derivation failed: {}", e))?;
    let export_key_hex = hex::encode(&export_key);

    let app_data_dir = state.get_app_data_dir()?;
    let meta_conn = state.get_meta_connection().await?;

    // Collect workspace list
    let entries: Vec<WorkspaceEntry> = {
        let mut stmt = meta_conn.prepare(
            "SELECT name, icon, color, db_file, sort_order FROM workspaces ORDER BY sort_order, created_at"
        ).map_err(|e| e.to_string())?;
        stmt.query_map([], |row| Ok(WorkspaceEntry {
            name: row.get(0)?,
            icon: row.get(1)?,
            color: row.get(2)?,
            db_file: row.get(3)?,
            sort_order: row.get(4)?,
        })).map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect()
    };

    // Create the zip archive
    let dest_file = std::fs::File::create(&dest_path)
        .map_err(|e| format!("Cannot create backup file: {}", e))?;
    let mut zip = zip::ZipWriter::new(dest_file);
    let opts = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    let tmp_dir = std::env::temp_dir();

    // Add each workspace database (rekeyed from master → export key)
    for ws in &entries {
        let src = app_data_dir.join(&ws.db_file);
        if !src.exists() {
            continue;
        }
        let tmp = tmp_dir.join(format!("vkexp_{}", ws.db_file));
        std::fs::copy(&src, &tmp)
            .map_err(|e| format!("Failed to copy '{}': {}", ws.db_file, e))?;

        {
            let conn = open_sqlcipher(&tmp, &master_key_hex)?;
            conn.execute_batch(&format!("PRAGMA rekey = \"x'{export_key_hex}'\";"))
                .map_err(|e| format!("Rekey failed for '{}': {}", ws.db_file, e))?;
        }

        let mut db_file = std::fs::File::open(&tmp)
            .map_err(|e| e.to_string())?;
        std::fs::remove_file(&tmp).ok();

        zip.start_file(format!("workspaces/{}", ws.db_file), opts)
            .map_err(|e| e.to_string())?;
        std::io::copy(&mut db_file, &mut zip).map_err(|e| e.to_string())?;
    }

    // Add media files as-is (streaming, no full-file buffering)
    let media_dir = app_data_dir.join("media");
    if media_dir.exists() {
        for entry in std::fs::read_dir(&media_dir).map_err(|e| e.to_string())?.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let fname = path.file_name().unwrap().to_string_lossy().into_owned();
            if let Ok(mut f) = std::fs::File::open(&path) {
                let _ = zip.start_file(format!("media/{}", fname), opts);
                let _ = std::io::copy(&mut f, &mut zip);
            }
        }
    }

    // Write manifest (last, so partial writes are detectable)
    let manifest = BackupManifest {
        version: 1,
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        exported_at: chrono::Utc::now().to_rfc3339(),
        export_key_salt: hex::encode(&salt),
        workspaces: entries,
    };
    let json = serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?;
    zip.start_file("manifest.json", opts).map_err(|e| e.to_string())?;
    zip.write_all(&json).map_err(|e| e.to_string())?;
    zip.finish().map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn import_vault(
    state: State<'_, AppState>,
    src_path: String,
    export_password: String,
) -> Result<u32, String> {
    let master_key = state.get_master_key().ok_or("Vault is locked")?;
    let master_key_hex = hex::encode(&master_key);
    let app_data_dir = state.get_app_data_dir()?;

    // Open zip and read all data in one pass
    let file = std::fs::File::open(&src_path)
        .map_err(|e| format!("Cannot open backup file: {}", e))?;
    let mut zip = zip::ZipArchive::new(file)
        .map_err(|e| format!("Invalid backup file: {}", e))?;

    // Read manifest first
    let manifest: BackupManifest = {
        let mut mf = zip.by_name("manifest.json")
            .map_err(|_| "Backup file is missing manifest — may not be a valid .vkbak file".to_string())?;
        let mut s = String::new();
        mf.read_to_string(&mut s).map_err(|e| e.to_string())?;
        serde_json::from_str(&s).map_err(|e| format!("Invalid manifest: {}", e))?
    };

    // Derive export key from manifest salt + provided password
    let export_salt = hex::decode(&manifest.export_key_salt)
        .map_err(|_| "Invalid salt in backup manifest".to_string())?;
    let export_key = kdf::derive_key(export_password.as_bytes(), &export_salt)
        .map_err(|e| format!("Key derivation failed: {}", e))?;
    let export_key_hex = hex::encode(&export_key);

    // Single-pass extraction:
    //   - workspace DB files → buffer in memory (small, need SQLCipher rekey)
    //   - media files        → stream directly to disk (potentially very large)
    let mut workspace_data: Vec<(String, Vec<u8>)> = Vec::new();

    let media_dir = app_data_dir.join("media");
    std::fs::create_dir_all(&media_dir).ok();

    for i in 0..zip.len() {
        let mut zf = zip.by_index(i).map_err(|e| e.to_string())?;
        let name = zf.name().to_string();

        if let Some(fname) = name.strip_prefix("workspaces/") {
            if !fname.is_empty() {
                let mut buf = Vec::new();
                zf.read_to_end(&mut buf).map_err(|e| e.to_string())?;
                workspace_data.push((fname.to_string(), buf));
            }
        } else if let Some(fname) = name.strip_prefix("media/") {
            if !fname.is_empty() {
                let dest = media_dir.join(fname);
                if !dest.exists() {
                    if let Ok(mut out) = std::fs::File::create(&dest) {
                        std::io::copy(&mut zf, &mut out).ok();
                    }
                }
            }
        }
    }

    let tmp_dir = std::env::temp_dir();
    let mut imported = 0u32;

    // Pre-fetch existing workspace names for conflict detection
    let meta_conn = state.get_meta_connection().await?;
    let mut existing_names: std::collections::HashSet<String> = {
        let mut stmt = meta_conn.prepare("SELECT name FROM workspaces")
            .map_err(|e| e.to_string())?;
        stmt.query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect()
    };

    // Process each workspace
    for ws in &manifest.workspaces {
        let data = match workspace_data.iter().find(|(f, _)| f == &ws.db_file) {
            Some((_, d)) => d,
            None => continue,
        };

        let tmp = tmp_dir.join(format!("vkimp_{}", ws.db_file));
        std::fs::write(&tmp, data)
            .map_err(|e| format!("Failed to write temp db: {}", e))?;

        // Verify export key and rekey to local master key
        {
            let conn = open_sqlcipher(&tmp, &export_key_hex)
                .map_err(|_| {
                    std::fs::remove_file(&tmp).ok();
                    "Incorrect export password".to_string()
                })?;
            conn.execute_batch(&format!("PRAGMA rekey = \"x'{master_key_hex}'\";"))
                .map_err(|e| {
                    std::fs::remove_file(&tmp).ok();
                    format!("Rekey failed: {}", e)
                })?;
        }

        // Resolve name conflict: append (YYMMDD-xxxx) suffix if needed
        let final_name = if existing_names.contains(&ws.name) {
            let date = chrono::Utc::now().format("%y%m%d").to_string();
            let suffix: String = {
                use rand::Rng;
                rand::thread_rng()
                    .sample_iter(rand::distributions::Alphanumeric)
                    .take(4)
                    .map(|c| (c as char).to_lowercase().next().unwrap())
                    .collect()
            };
            format!("{}({}-{})", ws.name, date, suffix)
        } else {
            ws.name.clone()
        };

        // Move to app data dir with a fresh unique filename
        let new_id = uuid::Uuid::new_v4().to_string();
        let new_db_file = format!("workspace_{}.db", &new_id[..8]);
        std::fs::rename(&tmp, app_data_dir.join(&new_db_file))
            .map_err(|e| format!("Failed to place db: {}", e))?;

        // Register workspace in meta db
        let now = chrono::Utc::now().to_rfc3339();
        meta_conn.execute(
            "INSERT INTO workspaces (id, name, icon, color, db_file, sort_order, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                &new_id, &final_name, &ws.icon, &ws.color,
                &new_db_file, ws.sort_order, &now, &now,
            ],
        ).map_err(|e| format!("Failed to register workspace '{}': {}", final_name, e))?;

        // Track the new name so subsequent workspaces in this batch also avoid it
        existing_names.insert(final_name);
        imported += 1;
    }

    Ok(imported)
}
