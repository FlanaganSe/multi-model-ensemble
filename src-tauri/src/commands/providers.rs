use crate::perspectives;
use crate::providers::{claude, codex, gemini, types::ProviderProbeResult};

#[tauri::command]
pub async fn probe_providers() -> Result<Vec<ProviderProbeResult>, String> {
    let (claude_result, codex_result, gemini_result) =
        tokio::join!(claude::probe(), codex::probe(), gemini::probe());

    Ok(vec![claude_result, codex_result, gemini_result])
}

#[tauri::command]
pub async fn list_perspectives() -> Result<Vec<perspectives::Perspective>, String> {
    Ok(perspectives::load_builtin_perspectives())
}
