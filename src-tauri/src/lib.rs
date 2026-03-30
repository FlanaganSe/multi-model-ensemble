pub mod commands;
pub mod providers;
pub mod session_store;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::providers::probe_providers,
            commands::sessions::create_session,
            commands::sessions::list_sessions,
            commands::sessions::archive_session,
            commands::sessions::delete_session,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
