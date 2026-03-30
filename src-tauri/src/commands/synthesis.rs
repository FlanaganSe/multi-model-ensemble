use crate::session_store;
use crate::synthesis;

/// Get the rendered brief.md for a session.
#[tauri::command]
pub async fn get_brief(session_id: String) -> Result<String, String> {
    let root = session_store::safe_paths::SessionRoot::resolve().map_err(|e| e.to_string())?;
    let session_dir = root.session_path(&session_id);
    root.assert_within_root(&session_dir)
        .map_err(|e| e.to_string())?;

    let brief_path = session_dir.join("synthesis").join("brief.md");
    if !brief_path.exists() {
        return Err("No brief found for this session. Run synthesis first.".to_string());
    }

    std::fs::read_to_string(&brief_path).map_err(|e| e.to_string())
}

/// Get the evidence matrix JSON for a session.
#[tauri::command]
pub async fn get_evidence_matrix(session_id: String) -> Result<serde_json::Value, String> {
    let root = session_store::safe_paths::SessionRoot::resolve().map_err(|e| e.to_string())?;
    let session_dir = root.session_path(&session_id);
    root.assert_within_root(&session_dir)
        .map_err(|e| e.to_string())?;

    let matrix_path = session_dir.join("synthesis").join("evidence-matrix.json");
    if !matrix_path.exists() {
        return Err("No evidence matrix found for this session.".to_string());
    }

    let content = std::fs::read_to_string(&matrix_path).map_err(|e| e.to_string())?;
    serde_json::from_str(&content).map_err(|e| e.to_string())
}

/// Get all normalized run artifacts for a session.
#[tauri::command]
pub async fn get_normalized_runs(session_id: String) -> Result<Vec<serde_json::Value>, String> {
    let root = session_store::safe_paths::SessionRoot::resolve().map_err(|e| e.to_string())?;
    let session_dir = root.session_path(&session_id);
    root.assert_within_root(&session_dir)
        .map_err(|e| e.to_string())?;

    let runs_dir = session_dir.join("runs");
    if !runs_dir.exists() {
        return Ok(vec![]);
    }

    let mut normalized = Vec::new();
    for provider_dir in std::fs::read_dir(&runs_dir).map_err(|e| e.to_string())? {
        let provider_dir = provider_dir.map_err(|e| e.to_string())?;
        if !provider_dir.path().is_dir() {
            continue;
        }
        for perspective_dir in std::fs::read_dir(provider_dir.path()).map_err(|e| e.to_string())? {
            let perspective_dir = perspective_dir.map_err(|e| e.to_string())?;
            if !perspective_dir.path().is_dir() {
                continue;
            }
            let norm_path = perspective_dir.path().join("normalized.json");
            if norm_path.exists() {
                let content = std::fs::read_to_string(&norm_path).map_err(|e| e.to_string())?;
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                    normalized.push(val);
                }
            }
        }
    }

    Ok(normalized)
}

/// List all artifacts for a session with their paths and types.
#[tauri::command]
pub async fn get_session_artifacts(
    session_id: String,
) -> Result<Vec<synthesis::types::SessionArtifact>, String> {
    let root = session_store::safe_paths::SessionRoot::resolve().map_err(|e| e.to_string())?;
    let session_dir = root.session_path(&session_id);
    root.assert_within_root(&session_dir)
        .map_err(|e| e.to_string())?;

    if !session_dir.exists() {
        return Err(format!("Session {session_id} not found"));
    }

    let mut artifacts = Vec::new();

    // Session metadata
    add_artifact_if_exists(&session_dir, "session.json", "metadata", &mut artifacts);
    add_artifact_if_exists(&session_dir, "run-config.json", "config", &mut artifacts);
    add_artifact_if_exists(&session_dir, "run-summary.json", "summary", &mut artifacts);

    // Prompts
    add_artifact_if_exists(&session_dir, "prompts/base.md", "prompt", &mut artifacts);
    add_artifact_if_exists(
        &session_dir,
        "prompts/context-pack.md",
        "context",
        &mut artifacts,
    );
    add_artifact_if_exists(
        &session_dir,
        "prompts/context-manifest.json",
        "context_manifest",
        &mut artifacts,
    );

    // Synthesis artifacts
    add_artifact_if_exists(&session_dir, "synthesis/brief.md", "brief", &mut artifacts);
    add_artifact_if_exists(
        &session_dir,
        "synthesis/evidence-matrix.json",
        "evidence_matrix",
        &mut artifacts,
    );
    add_artifact_if_exists(
        &session_dir,
        "synthesis/synthesis.json",
        "synthesis",
        &mut artifacts,
    );

    // Run artifacts (per provider/perspective)
    let runs_dir = session_dir.join("runs");
    if runs_dir.exists() {
        if let Ok(provider_entries) = std::fs::read_dir(&runs_dir) {
            for provider_dir in provider_entries.flatten() {
                if !provider_dir.path().is_dir() {
                    continue;
                }
                let provider_name = provider_dir.file_name().to_string_lossy().to_string();
                if let Ok(persp_entries) = std::fs::read_dir(provider_dir.path()) {
                    for persp_dir in persp_entries.flatten() {
                        if !persp_dir.path().is_dir() {
                            continue;
                        }
                        let persp_name = persp_dir.file_name().to_string_lossy().to_string();
                        let prefix = format!("runs/{provider_name}/{persp_name}");

                        add_artifact_if_exists(
                            &session_dir,
                            &format!("{prefix}/stdout.txt"),
                            "raw_output",
                            &mut artifacts,
                        );
                        add_artifact_if_exists(
                            &session_dir,
                            &format!("{prefix}/stderr.txt"),
                            "raw_stderr",
                            &mut artifacts,
                        );
                        add_artifact_if_exists(
                            &session_dir,
                            &format!("{prefix}/invocation.json"),
                            "invocation",
                            &mut artifacts,
                        );
                        add_artifact_if_exists(
                            &session_dir,
                            &format!("{prefix}/result.json"),
                            "job_result",
                            &mut artifacts,
                        );
                        add_artifact_if_exists(
                            &session_dir,
                            &format!("{prefix}/normalized.json"),
                            "normalized",
                            &mut artifacts,
                        );
                    }
                }
            }
        }
    }

    // Event log
    add_artifact_if_exists(
        &session_dir,
        "logs/events.jsonl",
        "event_log",
        &mut artifacts,
    );

    Ok(artifacts)
}

/// Read the content of a specific artifact file.
#[tauri::command]
pub async fn read_artifact(session_id: String, relative_path: String) -> Result<String, String> {
    // Reject obviously malicious paths before any filesystem access
    if relative_path.contains("..") {
        return Err("Path traversal not allowed".to_string());
    }

    let root = session_store::safe_paths::SessionRoot::resolve().map_err(|e| e.to_string())?;
    let session_dir = root.session_path(&session_id);
    root.assert_within_root(&session_dir)
        .map_err(|e| e.to_string())?;

    let artifact_path = session_dir.join(&relative_path);

    if !artifact_path.exists() {
        return Err(format!("Artifact not found: {relative_path}"));
    }

    // Safety: canonicalize and verify the resolved path is within THIS session dir,
    // not just the app root. Prevents cross-session reads via path traversal.
    let canonical_session = session_dir
        .canonicalize()
        .map_err(|e| format!("Cannot resolve session dir: {e}"))?;
    let canonical_artifact = artifact_path
        .canonicalize()
        .map_err(|e| format!("Cannot resolve artifact path: {e}"))?;

    if !canonical_artifact.starts_with(&canonical_session) {
        return Err("Artifact path is outside session directory".to_string());
    }

    std::fs::read_to_string(&artifact_path).map_err(|e| e.to_string())
}

fn add_artifact_if_exists(
    session_dir: &std::path::Path,
    relative_path: &str,
    artifact_type: &str,
    artifacts: &mut Vec<synthesis::types::SessionArtifact>,
) {
    let full_path = session_dir.join(relative_path);
    if full_path.exists() {
        let size_bytes = std::fs::metadata(&full_path).map(|m| m.len()).unwrap_or(0);
        artifacts.push(synthesis::types::SessionArtifact {
            relative_path: relative_path.to_string(),
            artifact_type: artifact_type.to_string(),
            size_bytes,
        });
    }
}
