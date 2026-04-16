mod commands;
mod crypto;
mod db;
mod models;

use db::connection::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // 已有实例运行时，聚焦到已有窗口
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            let state = AppState::new(app.handle().clone());
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Auth
            commands::auth::is_setup_complete,
            commands::auth::setup_master_password,
            commands::auth::verify_master_password,
            commands::auth::recover_from_mnemonic,
            commands::auth::lock_vault,
            commands::auth::is_vault_unlocked,
            commands::auth::change_master_password,
            // Workspace
            commands::workspace::list_workspaces,
            commands::workspace::create_workspace,
            commands::workspace::update_workspace,
            commands::workspace::delete_workspace,
            commands::workspace::switch_workspace,
            commands::workspace::get_current_workspace,
            // Bookmarks
            commands::bookmarks::list_bookmarks,
            commands::bookmarks::get_bookmark,
            commands::bookmarks::create_bookmark,
            commands::bookmarks::update_bookmark,
            commands::bookmarks::delete_bookmark,
            commands::bookmarks::list_bookmark_groups,
            commands::bookmarks::create_bookmark_group,
            commands::bookmarks::update_bookmark_group,
            commands::bookmarks::delete_bookmark_group,
            // Accounts
            commands::accounts::list_accounts,
            commands::accounts::get_account,
            commands::accounts::create_account,
            commands::accounts::update_account,
            commands::accounts::delete_account,
            commands::accounts::generate_password,
            commands::accounts::list_account_groups,
            commands::accounts::create_account_group,
            commands::accounts::update_account_group,
            commands::accounts::delete_account_group,
            // Notes
            commands::notes::list_notes,
            commands::notes::get_note,
            commands::notes::create_note,
            commands::notes::update_note,
            commands::notes::delete_note,
            commands::notes::encrypt_note,
            commands::notes::decrypt_note,
            commands::notes::permanently_decrypt_note,
            commands::notes::save_and_reencrypt_note,
            commands::notes::list_note_groups,
            commands::notes::create_note_group,
            commands::notes::update_note_group,
            commands::notes::delete_note_group,
            // Reports
            commands::reports::list_reports,
            commands::reports::get_report,
            commands::reports::create_report,
            commands::reports::update_report,
            commands::reports::delete_report,
            // Media
            commands::media::list_media_assets,
            commands::media::get_media_asset,
            commands::media::upload_media,
            commands::media::delete_media_asset,
            // Tags
            commands::tags::list_tags,
            commands::tags::create_tag,
            commands::tags::update_tag,
            commands::tags::delete_tag,
            commands::tags::tag_entity,
            commands::tags::untag_entity,
            commands::tags::get_entity_tags,
            commands::tags::get_entities_by_tag,
            commands::tags::list_all_entity_tags,
            // Search
            commands::search::search,
            commands::search::simple_search,
            // Settings
            commands::settings::get_settings,
            commands::settings::update_setting,
            commands::settings::get_auto_lock_timeout,
            commands::settings::set_auto_lock_timeout,
            // Fetch
            commands::fetch::fetch_url_metadata,
            // Updater
            commands::updater::check_for_updates,
        ])
        .run(tauri::generate_context!())
        .expect("error while running VaultKeeper");
}
