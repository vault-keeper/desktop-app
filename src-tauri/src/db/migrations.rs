use rusqlite::Connection;

/// Run migrations for the vault_meta database
pub fn run_meta_migrations(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS master_key_verify (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            verify_hash TEXT NOT NULL,
            salt TEXT NOT NULL,
            backup_salt TEXT NOT NULL,
            encrypted_master TEXT NOT NULL,
            master_nonce TEXT NOT NULL,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS workspaces (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            icon TEXT,
            color TEXT,
            db_file TEXT NOT NULL,
            sort_order INTEGER DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS app_settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        INSERT OR IGNORE INTO app_settings (key, value, updated_at) VALUES ('auto_lock_minutes', '5', datetime('now'));
        INSERT OR IGNORE INTO app_settings (key, value, updated_at) VALUES ('theme', 'system', datetime('now'));
        INSERT OR IGNORE INTO app_settings (key, value, updated_at) VALUES ('language', 'zh-CN', datetime('now'));"
    ).map_err(|e| format!("Meta migration failed: {}", e))?;

    Ok(())
}

/// Run migrations for a workspace database
pub fn run_workspace_migrations(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "-- Cleanup: drop unused report_templates table from old installs
        DROP TABLE IF EXISTS report_templates;

        -- Bookmark groups
        CREATE TABLE IF NOT EXISTS bookmark_groups (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            icon TEXT,
            color TEXT,
            parent_id TEXT,
            sort_order INTEGER DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (parent_id) REFERENCES bookmark_groups(id) ON DELETE CASCADE
        );

        -- Bookmarks
        CREATE TABLE IF NOT EXISTS bookmarks (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            url TEXT NOT NULL,
            description TEXT,
            favicon_url TEXT,
            group_id TEXT,
            sort_order INTEGER DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (group_id) REFERENCES bookmark_groups(id) ON DELETE SET NULL
        );

        -- Account groups
        CREATE TABLE IF NOT EXISTS account_groups (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            icon TEXT,
            color TEXT,
            parent_id TEXT,
            sort_order INTEGER DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (parent_id) REFERENCES account_groups(id) ON DELETE CASCADE
        );

        -- Accounts (credentials) - sensitive fields are AES-256-GCM encrypted
        CREATE TABLE IF NOT EXISTS accounts (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            url TEXT,
            username TEXT,
            password TEXT NOT NULL,
            notes TEXT,
            icon_url TEXT,
            group_id TEXT,
            favorite INTEGER DEFAULT 0,
            sort_order INTEGER DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (group_id) REFERENCES account_groups(id) ON DELETE SET NULL
        );

        -- Note groups
        CREATE TABLE IF NOT EXISTS note_groups (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            icon TEXT,
            color TEXT,
            parent_id TEXT,
            sort_order INTEGER DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (parent_id) REFERENCES note_groups(id) ON DELETE CASCADE
        );

        -- Notes
        CREATE TABLE IF NOT EXISTS notes (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            content TEXT NOT NULL,
            is_encrypted INTEGER DEFAULT 0,
            encrypted_content TEXT,
            encryption_nonce TEXT,
            secondary_salt TEXT,
            group_id TEXT,
            sort_order INTEGER DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (group_id) REFERENCES note_groups(id) ON DELETE SET NULL
        );

        -- Reports
        CREATE TABLE IF NOT EXISTS reports (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            type TEXT NOT NULL,
            content TEXT NOT NULL,
            date TEXT NOT NULL,
            week_start TEXT,
            week_end TEXT,
            month TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        -- Media assets
        CREATE TABLE IF NOT EXISTS media_assets (
            id TEXT PRIMARY KEY,
            filename TEXT NOT NULL,
            mime_type TEXT NOT NULL,
            size INTEGER NOT NULL,
            storage_type TEXT NOT NULL,
            storage_path TEXT NOT NULL,
            thumbnail_path TEXT,
            description TEXT,
            metadata TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        -- Tags
        CREATE TABLE IF NOT EXISTS tags (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            color TEXT,
            icon TEXT,
            created_at TEXT NOT NULL
        );

        -- Taggables (polymorphic tag associations)
        CREATE TABLE IF NOT EXISTS taggables (
            tag_id TEXT NOT NULL,
            taggable_type TEXT NOT NULL,
            taggable_id TEXT NOT NULL,
            created_at TEXT NOT NULL,
            PRIMARY KEY (tag_id, taggable_type, taggable_id),
            FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
        );

        -- Full-text search index (FTS5)
        CREATE VIRTUAL TABLE IF NOT EXISTS search_index USING fts5(
            entity_type,
            entity_id UNINDEXED,
            title,
            content,
            tags,
            tokenize='unicode61'
        );"
    ).map_err(|e| format!("Workspace migration failed: {}", e))?;

    Ok(())
}
