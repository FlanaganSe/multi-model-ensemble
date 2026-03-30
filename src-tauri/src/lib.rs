pub mod commands;
pub mod context;
pub mod orchestrator;
pub mod perspectives;
pub mod providers;
pub mod session_store;
pub mod synthesis;

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
            commands::runs::run_session,
            commands::synthesis::get_brief,
            commands::synthesis::get_evidence_matrix,
            commands::synthesis::get_normalized_runs,
            commands::synthesis::get_session_artifacts,
            commands::synthesis::read_artifact,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
