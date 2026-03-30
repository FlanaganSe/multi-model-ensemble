use crate::orchestrator;
use crate::orchestrator::types::{RunConfig, RunSummary};
use crate::providers::{claude, codex, gemini};
use crate::session_store;

#[tauri::command]
pub async fn run_session(
    prompt: String,
    providers: Vec<String>,
    perspectives: Vec<String>,
    working_directory: Option<String>,
    context_paths: Option<Vec<String>>,
    timeout_secs: Option<u64>,
    label: Option<String>,
) -> Result<RunSummary, String> {
    // Parse provider names
    let provider_names: Vec<crate::providers::types::ProviderName> = providers
        .iter()
        .filter_map(|p| match p.as_str() {
            "claude" => Some(crate::providers::types::ProviderName::Claude),
            "codex" => Some(crate::providers::types::ProviderName::Codex),
            "gemini" => Some(crate::providers::types::ProviderName::Gemini),
            _ => {
                log::warn!("Unknown provider: {p}");
                None
            }
        })
        .collect();

    if provider_names.is_empty() {
        return Err("No valid providers selected".to_string());
    }

    if perspectives.is_empty() {
        return Err("No perspectives selected".to_string());
    }

    if prompt.trim().is_empty() {
        return Err("Prompt cannot be empty".to_string());
    }

    // Create a session
    let session = session_store::create(label).map_err(|e| e.to_string())?;

    // Probe all selected providers
    let (claude_probe, codex_probe, gemini_probe) =
        tokio::join!(claude::probe(), codex::probe(), gemini::probe());
    let probe_results = vec![claude_probe, codex_probe, gemini_probe];

    // Build run config
    let config = RunConfig {
        session_id: session.id.clone(),
        prompt,
        providers: provider_names,
        perspectives,
        working_directory,
        context_paths: context_paths.unwrap_or_default(),
        timeout_secs: timeout_secs.unwrap_or(120),
        max_concurrent: 4,
    };

    // Persist run config to session
    let session_dir = std::path::Path::new(&session.path);
    let _ = std::fs::write(
        session_dir.join("run-config.json"),
        serde_json::to_string_pretty(&config).unwrap_or_default(),
    );

    // Execute the run
    let summary = orchestrator::run_jobs(&config, session_dir, &probe_results).await;

    Ok(summary)
}
