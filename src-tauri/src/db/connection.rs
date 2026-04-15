use rusqlite::Connection;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};
use crate::commands::workspace::Workspace;
use crate::db::migrations;

/// Application state holding database connections and master key
pub struct AppState {
    /// Master key, kept in memory only during unlocked session
    master_key: Mutex<Option<Vec<u8>>>,
    /// Currently active workspace
    current_workspace: Mutex<Option<Workspace>>,
    /// Derived key for current workspace DB (cached to avoid repeated KDF)
    current_workspace_key: Mutex<Option<Vec<u8>>>,
    /// Current workspace DB file name
    current_db_file: Mutex<Option<String>>,
    /// App handle for resolving paths
    app_handle: AppHandle,
}

impl AppState {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            master_key: Mutex::new(None),
            current_workspace: Mutex::new(None),
            current_workspace_key: Mutex::new(None),
            current_db_file: Mutex::new(None),
            app_handle,
        }
    }

    pub fn set_master_key(&self, key: Vec<u8>) {
        *self.master_key.lock().unwrap() = Some(key);
    }

    pub fn get_master_key(&self) -> Option<Vec<u8>> {
        self.master_key.lock().unwrap().clone()
    }

    pub fn clear_master_key(&self) {
        *self.master_key.lock().unwrap() = None;
        *self.current_workspace.lock().unwrap() = None;
        *self.current_workspace_key.lock().unwrap() = None;
        *self.current_db_file.lock().unwrap() = None;
    }

    pub fn is_unlocked(&self) -> bool {
        self.master_key.lock().unwrap().is_some()
    }

    pub fn set_current_workspace(&self, workspace: Workspace) {
        *self.current_workspace.lock().unwrap() = Some(workspace);
    }

    pub fn get_current_workspace(&self) -> Option<Workspace> {
        self.current_workspace.lock().unwrap().clone()
    }

    pub fn set_workspace_key(&self, key: Vec<u8>, db_file: String) {
        *self.current_workspace_key.lock().unwrap() = Some(key);
        *self.current_db_file.lock().unwrap() = Some(db_file);
    }

    // ── SQLCipher helpers ─────────────────────────────────────────────────────

    /// Derive the SQLCipher hex key from the master key.
    /// The 32-byte master key is used directly as the AES-256 database key.
    fn workspace_db_key(master_key: &[u8]) -> String {
        hex::encode(master_key)
    }

    /// Try opening a database with a specific SQLCipher key.
    /// Returns `Ok(conn)` only if the key is valid and the DB is readable.
    fn try_open_encrypted(path: &std::path::Path, key_hex: &str) -> Result<Connection, ()> {
        let conn = Connection::open(path).map_err(|_| ())?;
        conn.execute_batch(&format!("PRAGMA key = \"x'{}'\";\nPRAGMA cipher_page_size = 4096;", key_hex))
            .map_err(|_| ())?;
        conn.execute_batch("SELECT count(*) FROM sqlite_master;")
            .map_err(|_| ())?;
        Ok(conn)
    }

    /// Migrate a legacy plaintext SQLite database to SQLCipher encryption.
    ///
    /// Uses SQLCipher's built-in `sqlcipher_export` function to copy all data
    /// into an encrypted file, then replaces the original atomically.
    fn migrate_plaintext_to_sqlcipher(path: &std::path::Path, key_hex: &str) -> Result<(), String> {
        let tmp_path = path.with_extension("sqlcipher_tmp");

        // Clean up any leftover temp file from a previous failed migration.
        if tmp_path.exists() {
            std::fs::remove_file(&tmp_path).ok();
        }

        {
            // Open source as plain SQLite (no PRAGMA key → plaintext mode).
            let source = Connection::open(path)
                .map_err(|e| format!("Cannot open legacy database: {}", e))?;

            // Escape single-quotes in the path (Windows backslashes are safe).
            let tmp_str = tmp_path.to_str().ok_or("Non-UTF8 path")?
                .replace('\'', "''");

            // ATTACH creates the encrypted target; sqlcipher_export copies everything.
            source.execute_batch(&format!(
                "ATTACH DATABASE '{tmp}' AS sqlcipher_export KEY \"x'{key}'\";\
                 SELECT sqlcipher_export('sqlcipher_export');\
                 DETACH DATABASE sqlcipher_export;",
                tmp = tmp_str,
                key = key_hex,
            )).map_err(|e| format!("sqlcipher_export failed: {e}"))?;
        }

        // Replace original file with the encrypted copy.
        std::fs::rename(&tmp_path, path)
            .map_err(|e| format!("Failed to replace database file: {}", e))?;

        Ok(())
    }

    /// Open an encrypted workspace database.
    ///
    /// Strategy:
    ///   1. Try to open with the encryption key (normal / already-encrypted case).
    ///   2. If that fails, try as a legacy plaintext database and migrate it.
    ///   3. Re-open as encrypted after migration.
    fn open_workspace_db_conn(path: &std::path::Path, key_hex: &str) -> Result<Connection, String> {
        // Fast path: already encrypted.
        if let Ok(conn) = Self::try_open_encrypted(path, key_hex) {
            return Ok(conn);
        }

        // Check whether the file is a readable plaintext SQLite database
        // (legacy databases created before SQLCipher was introduced).
        let is_legacy_plaintext = Connection::open(path)
            .and_then(|c| {
                c.execute_batch("SELECT count(*) FROM sqlite_master;")?;
                Ok(())
            })
            .is_ok();

        if is_legacy_plaintext {
            Self::migrate_plaintext_to_sqlcipher(path, key_hex)
                .map_err(|e| format!("Database encryption migration failed: {e}"))?;

            return Self::try_open_encrypted(path, key_hex)
                .map_err(|_| "Database inaccessible after migration – key mismatch?".to_string());
        }

        Err("Database is inaccessible: incorrect key or corrupted file".to_string())
    }

    // ── Public connection methods ─────────────────────────────────────────────

    /// Open a fresh connection to the meta database and ensure migrations have run.
    ///
    /// The meta database is intentionally NOT encrypted with SQLCipher because it
    /// must be readable before the master key is known (needed to verify the
    /// master password).  Its sensitive rows (verify_hash, encrypted_master) are
    /// already cryptographically protected at the application level.
    pub async fn get_meta_connection(&self) -> Result<Connection, String> {
        let app_data_dir = self.app_handle.path().app_config_dir()
            .map_err(|e| format!("Failed to get app config dir: {}", e))?;

        std::fs::create_dir_all(&app_data_dir)
            .map_err(|e| format!("Failed to create app config dir: {}", e))?;

        let db_path = app_data_dir.join("vault_meta.db");
        let conn = Connection::open(&db_path)
            .map_err(|e| format!("Failed to open meta database: {}", e))?;

        migrations::run_meta_migrations(&conn)
            .map_err(|e| format!("Meta migration failed: {}", e))?;

        Ok(conn)
    }

    /// Open a fresh SQLCipher-encrypted connection to the current workspace database.
    /// Automatically migrates plaintext databases from older app versions.
    pub async fn get_workspace_connection(&self) -> Result<Connection, String> {
        let master_key = self.get_master_key()
            .ok_or_else(|| "Vault is locked".to_string())?;

        let db_file = self.current_db_file.lock().unwrap().clone()
            .ok_or_else(|| "No workspace selected".to_string())?;

        let app_data_dir = self.app_handle.path().app_config_dir()
            .map_err(|e| format!("Failed to get app config dir: {}", e))?;

        let db_path = app_data_dir.join(&db_file);
        let key_hex = Self::workspace_db_key(&master_key);

        Self::open_workspace_db_conn(&db_path, &key_hex)
    }

    /// Create a new SQLCipher-encrypted workspace database and run migrations.
    pub async fn create_workspace_db(&self, db_file: &str) -> Result<(), String> {
        let master_key = self.get_master_key()
            .ok_or_else(|| "Vault is locked".to_string())?;

        let app_data_dir = self.app_handle.path().app_config_dir()
            .map_err(|e| format!("Failed to get app config dir: {}", e))?;

        let db_path = app_data_dir.join(db_file);
        let key_hex = Self::workspace_db_key(&master_key);

        let conn = Self::open_workspace_db_conn(&db_path, &key_hex)?;

        migrations::run_workspace_migrations(&conn)
            .map_err(|e| format!("Failed to run migrations: {}", e))?;

        Ok(())
    }

    /// Switch to a workspace database, migrating to SQLCipher if needed.
    pub async fn switch_workspace_db(&self, db_file: &str) -> Result<(), String> {
        let master_key = self.get_master_key()
            .ok_or_else(|| "Vault is locked".to_string())?;

        let app_data_dir = self.app_handle.path().app_config_dir()
            .map_err(|e| format!("Failed to get app config dir: {}", e))?;

        let db_path = app_data_dir.join(db_file);
        let key_hex = Self::workspace_db_key(&master_key);

        // Verify accessibility (migrates automatically if legacy plaintext format).
        Self::open_workspace_db_conn(&db_path, &key_hex)?;

        *self.current_db_file.lock().unwrap() = Some(db_file.to_string());
        Ok(())
    }

    /// Delete a workspace database file.
    pub async fn delete_workspace_db(&self, db_file: &str) -> Result<(), String> {
        let app_data_dir = self.app_handle.path().app_config_dir()
            .map_err(|e| format!("Failed to get app config dir: {}", e))?;

        let db_path = app_data_dir.join(db_file);

        // If this is the current DB, clear it
        {
            let mut current = self.current_db_file.lock().unwrap();
            if current.as_deref() == Some(db_file) {
                *current = None;
                *self.current_workspace_key.lock().unwrap() = None;
                *self.current_workspace.lock().unwrap() = None;
            }
        }

        if db_path.exists() {
            std::fs::remove_file(&db_path)
                .map_err(|e| format!("Failed to delete workspace database: {}", e))?;
        }

        Ok(())
    }

    /// Get the app data directory path.
    pub fn get_app_data_dir(&self) -> Result<std::path::PathBuf, String> {
        self.app_handle.path().app_config_dir()
            .map_err(|e| format!("Failed to get app config dir: {}", e))
    }
}
