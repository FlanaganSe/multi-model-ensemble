use crate::orchestrator;
use crate::orchestrator::types::{JobResult, RunConfig, RunSummary};
use crate::providers::{claude, codex, gemini};
use crate::session_store;
use crate::synthesis;

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn run_session(
    prompt: String,
    providers: Vec<String>,
    perspectives: Vec<String>,
    working_directory: Option<String>,
    context_paths: Option<Vec<String>>,
    timeout_secs: Option<u64>,
    label: Option<String>,
    strategy: Option<String>,
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

    // Run synthesis as post-processing (failure here does not invalidate the session)
    let synthesis_strategy = match strategy.as_deref() {
        Some("comprehensive") => synthesis::types::SynthesisStrategy::Comprehensive,
        Some("executive") => synthesis::types::SynthesisStrategy::Executive,
        _ => synthesis::types::SynthesisStrategy::Consensus,
    };

    if let Err(e) = run_synthesis(session_dir, &session.id, &summary, synthesis_strategy) {
        log::error!("Synthesis failed for session {}: {e}", session.id);
        // Synthesis failure is non-fatal — session and raw artifacts remain valid
    }

    Ok(summary)
}

/// Get job-level results for a session (for retry UI and detailed status).
#[tauri::command]
pub async fn get_run_results(session_id: String) -> Result<Vec<JobResult>, String> {
    let root =
        crate::session_store::safe_paths::SessionRoot::resolve().map_err(|e| e.to_string())?;
    let session_dir = root.session_path(&session_id);
    root.assert_within_root(&session_dir)
        .map_err(|e| e.to_string())?;

    let summary_path = session_dir.join("run-summary.json");
    if !summary_path.exists() {
        return Ok(vec![]);
    }

    let content = std::fs::read_to_string(&summary_path).map_err(|e| e.to_string())?;
    let results: Vec<JobResult> = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    Ok(results)
}

/// Run the synthesis pipeline: normalize → evidence matrix → strategy → brief.md
fn run_synthesis(
    session_dir: &std::path::Path,
    session_id: &str,
    summary: &RunSummary,
    strategy: synthesis::types::SynthesisStrategy,
) -> Result<(), Box<dyn std::error::Error>> {
    let synthesis_dir = session_dir.join("synthesis");
    std::fs::create_dir_all(&synthesis_dir)?;

    // Build evidence matrix (normalizes all job results internally)
    let (matrix, normalized_runs) =
        synthesis::evidence::build_evidence_matrix(session_id, &summary.jobs);

    // Persist normalized runs alongside raw artifacts
    for nr in &normalized_runs {
        let provider_name = match nr.provider {
            crate::providers::types::ProviderName::Claude => "claude",
            crate::providers::types::ProviderName::Codex => "codex",
            crate::providers::types::ProviderName::Gemini => "gemini",
        };
        let norm_path = session_dir
            .join("runs")
            .join(provider_name)
            .join(&nr.perspective_id)
            .join("normalized.json");
        if let Some(parent) = norm_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&norm_path, serde_json::to_string_pretty(nr)?)?;
    }

    // Persist evidence matrix
    std::fs::write(
        synthesis_dir.join("evidence-matrix.json"),
        serde_json::to_string_pretty(&matrix)?,
    )?;

    // Apply synthesis strategy
    let output = synthesis::strategies::synthesize(&matrix, strategy);

    // Persist synthesis output
    std::fs::write(
        synthesis_dir.join("synthesis.json"),
        serde_json::to_string_pretty(&output)?,
    )?;

    // Render and persist brief.md
    let coverage_cells: Vec<(String, String, String)> = matrix
        .coverage
        .cells
        .iter()
        .map(|c| {
            let provider = match c.provider {
                crate::providers::types::ProviderName::Claude => "Claude",
                crate::providers::types::ProviderName::Codex => "Codex",
                crate::providers::types::ProviderName::Gemini => "Gemini",
            };
            let status = serde_json::to_value(&c.status)
                .ok()
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| format!("{:?}", c.status));
            (provider.to_string(), c.perspective.clone(), status)
        })
        .collect();

    let brief = synthesis::brief::render_brief_with_coverage(&output, &coverage_cells);
    std::fs::write(synthesis_dir.join("brief.md"), &brief)?;

    log::info!(
        "Synthesis complete for session {session_id}: {} themes, {} recommendations",
        output.themes.len(),
        output.recommendations.len()
    );

    Ok(())
}
