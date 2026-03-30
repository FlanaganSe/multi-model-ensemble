use crate::session_store::{self, layout::SessionListEntry};

#[tauri::command]
pub async fn create_session(label: Option<String>) -> Result<SessionListEntry, String> {
    session_store::create(label).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_sessions() -> Result<Vec<SessionListEntry>, String> {
    session_store::list().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn archive_session(session_id: String) -> Result<(), String> {
    session_store::archive(&session_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_session(session_id: String) -> Result<(), String> {
    session_store::delete(&session_id).map_err(|e| e.to_string())
}
