pub mod events;
pub mod types;

use crate::context;
use crate::perspectives;
use crate::providers::types::ProviderName;
use events::{Event, EventLogger};
use std::fs;
use std::path::Path;
use types::{InvocationMetadata, JobResult, JobSpec, JobState, RunConfig, RunSummary};

/// Build the job matrix from a RunConfig: providers × perspectives.
pub fn build_job_matrix(config: &RunConfig, context_content: Option<&str>) -> Vec<JobSpec> {
    let mut jobs = Vec::new();

    for provider in &config.providers {
        for perspective_id in &config.perspectives {
            let perspective = perspectives::get_perspective(perspective_id);
            let instructions = perspective
                .as_ref()
                .map(|p| p.instructions.clone())
                .unwrap_or_default();

            let assembled_prompt = perspectives::assemble_prompt(
                &config.prompt,
                &perspective.unwrap_or(perspectives::Perspective {
                    id: perspective_id.clone(),
                    label: perspective_id.clone(),
                    instructions: instructions.clone(),
                }),
                context_content,
            );

            jobs.push(JobSpec::new(
                provider.clone(),
                perspective_id.clone(),
                assembled_prompt,
                instructions,
                config.working_directory.clone(),
                context_content.map(|s| s.to_string()),
                config.timeout_secs,
            ));
        }
    }

    jobs
}

/// Execute all jobs with concurrency control, timeout handling, and artifact persistence.
pub async fn run_jobs(
    config: &RunConfig,
    session_dir: &Path,
    probe_results: &[crate::providers::types::ProviderProbeResult],
) -> RunSummary {
    let logger = EventLogger::new(session_dir.join("logs").join("events.jsonl"));

    // Build context pack
    let context_pack =
        context::build_context_pack(&config.context_paths, config.working_directory.as_deref())
            .ok();

    // Persist context manifest and content
    if let Some(ref pack) = context_pack {
        let prompts_dir = session_dir.join("prompts");
        let _ = fs::write(prompts_dir.join("context-pack.md"), &pack.content);
        let _ = fs::write(
            prompts_dir.join("context-manifest.json"),
            serde_json::to_string_pretty(&pack.manifest).unwrap_or_default(),
        );
    }

    // Persist base prompt
    let _ = fs::write(session_dir.join("prompts").join("base.md"), &config.prompt);

    let context_content = context_pack.as_ref().map(|p| p.content.as_str());
    let jobs = build_job_matrix(config, context_content);

    logger.log_best_effort(&Event::RunStarted {
        session_id: config.session_id.clone(),
        total_jobs: jobs.len(),
    });

    // Log all queued jobs and persist perspective files
    let mut jobs = jobs;
    for job in &mut jobs {
        logger.log_best_effort(&Event::JobQueued {
            job_id: job.id.clone(),
            provider: job.provider.clone(),
            perspective_id: job.perspective_id.clone(),
        });

        // Persist perspective text to session dir
        let persp_dir = session_dir.join("prompts").join("perspectives");
        let persp_file = persp_dir.join(format!("{}.md", job.perspective_id));
        let _ = fs::write(&persp_file, &job.perspective_instructions);

        // Set perspective_file path so Gemini can use GEMINI_SYSTEM_MD
        job.perspective_file = Some(persp_file.to_string_lossy().to_string());
    }

    let total_jobs = jobs.len();

    // Execute jobs with concurrency control
    let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(config.max_concurrent));
    let mut handles = Vec::new();

    for job in jobs {
        let sem = semaphore.clone();
        let session_dir = session_dir.to_path_buf();
        let logger_path = session_dir.join("logs").join("events.jsonl");
        let probe_results = probe_results.to_vec();

        let handle = tokio::spawn(async move {
            let logger = EventLogger::new(logger_path);
            let _permit = sem.acquire().await.expect("semaphore closed");

            execute_single_job(&job, &session_dir, &logger, &probe_results).await
        });

        handles.push(handle);
    }

    // Collect all results
    let mut results = Vec::new();
    for handle in handles {
        match handle.await {
            Ok(result) => results.push(result),
            Err(e) => {
                log::error!("Job task panicked: {e}");
            }
        }
    }

    // Build summary
    let mut summary = RunSummary {
        session_id: config.session_id.clone(),
        total_jobs,
        completed: 0,
        failed: 0,
        timed_out: 0,
        blocked: 0,
        cancelled: 0,
        jobs: Vec::new(),
    };

    for result in &results {
        match result.state {
            JobState::Completed => summary.completed += 1,
            JobState::Failed => summary.failed += 1,
            JobState::TimedOut => summary.timed_out += 1,
            JobState::Blocked => summary.blocked += 1,
            JobState::Cancelled => summary.cancelled += 1,
            _ => {}
        }
    }

    logger.log_best_effort(&Event::RunCompleted {
        session_id: config.session_id.clone(),
        total_jobs: summary.total_jobs,
        completed: summary.completed,
        failed: summary.failed,
        timed_out: summary.timed_out,
        blocked: summary.blocked,
        cancelled: summary.cancelled,
    });

    // Persist run summary
    let _ = fs::write(
        session_dir.join("run-summary.json"),
        serde_json::to_string_pretty(&results).unwrap_or_default(),
    );

    summary.jobs = results;
    summary
}

/// Execute a single job: check provider readiness, spawn process, handle timeout, persist artifacts.
async fn execute_single_job(
    spec: &JobSpec,
    session_dir: &Path,
    logger: &EventLogger,
    probe_results: &[crate::providers::types::ProviderProbeResult],
) -> JobResult {
    let mut result = JobResult::queued(spec);

    // Check if provider is ready
    let probe = probe_results.iter().find(|p| p.provider == spec.provider);

    let probe = match probe {
        Some(p) if p.auth_ready => p,
        Some(p) => {
            let reason = p
                .blocked_reason
                .clone()
                .unwrap_or_else(|| "provider not ready".to_string());
            result.mark_blocked(reason.clone());
            logger.log_best_effort(&Event::JobBlocked {
                job_id: spec.id.clone(),
                provider: spec.provider.clone(),
                reason,
            });
            persist_job_artifacts(session_dir, spec, &result, p).await;
            return result;
        }
        None => {
            result.mark_blocked("provider not found in probe results".to_string());
            let fallback_probe = crate::providers::types::ProviderProbeResult {
                provider: spec.provider.clone(),
                status: crate::providers::types::ProviderStatus::Error,
                executable_path: None,
                version: None,
                auth_ready: false,
                blocked_reason: Some("not probed".to_string()),
                remediation: None,
            };
            persist_job_artifacts(session_dir, spec, &result, &fallback_probe).await;
            return result;
        }
    };

    // Mark as running
    result.mark_running();
    logger.log_best_effort(&Event::JobStarted {
        job_id: spec.id.clone(),
        provider: spec.provider.clone(),
    });

    // Execute via provider adapter
    let exec_result = tokio::time::timeout(
        std::time::Duration::from_secs(spec.timeout_secs),
        crate::providers::execute(&spec.provider, probe, spec),
    )
    .await;

    match exec_result {
        Ok(Ok((stdout, stderr, exit_code))) => {
            if exit_code == 0 {
                result.mark_completed(exit_code, stdout, stderr);
            } else {
                result.mark_failed(
                    format!("process exited with code {exit_code}"),
                    stdout,
                    stderr,
                    Some(exit_code),
                );
            }
        }
        Ok(Err(e)) => {
            result.mark_failed(e.clone(), String::new(), String::new(), None);
        }
        Err(_) => {
            // Timeout
            result.mark_timed_out(String::new(), String::new());
        }
    }

    // Emit terminal event
    let terminal_event = events::terminal_event(&result, spec.timeout_secs);
    logger.log_best_effort(&terminal_event);

    // Persist artifacts
    persist_job_artifacts(session_dir, spec, &result, probe).await;

    result
}

/// Persist raw artifacts for a job to the session directory.
async fn persist_job_artifacts(
    session_dir: &Path,
    spec: &JobSpec,
    result: &JobResult,
    probe: &crate::providers::types::ProviderProbeResult,
) {
    let provider_name = match spec.provider {
        ProviderName::Claude => "claude",
        ProviderName::Codex => "codex",
        ProviderName::Gemini => "gemini",
    };

    let job_dir = session_dir
        .join("runs")
        .join(provider_name)
        .join(&spec.perspective_id);

    if let Err(e) = fs::create_dir_all(&job_dir) {
        log::error!("Failed to create job dir {}: {e}", job_dir.display());
        return;
    }

    // Write invocation metadata
    let metadata = InvocationMetadata {
        job_id: spec.id.clone(),
        provider: spec.provider.clone(),
        provider_executable: probe.executable_path.clone(),
        provider_version: probe.version.clone(),
        perspective_id: spec.perspective_id.clone(),
        prompt: spec.prompt.clone(),
        perspective_instructions: spec.perspective_instructions.clone(),
        working_directory: spec.working_directory.clone(),
        timeout_secs: spec.timeout_secs,
        started_at: result.started_at.clone(),
        ended_at: result.ended_at.clone(),
        duration_ms: result.duration_ms,
        exit_code: result.exit_code,
        terminal_state: result.state.clone(),
    };

    let _ = fs::write(
        job_dir.join("invocation.json"),
        serde_json::to_string_pretty(&metadata).unwrap_or_default(),
    );

    // Write stdout (raw evidence)
    let _ = fs::write(job_dir.join("stdout.txt"), &result.stdout);

    // Write stderr
    let _ = fs::write(job_dir.join("stderr.txt"), &result.stderr);

    // Write a combined result file
    let _ = fs::write(
        job_dir.join("result.json"),
        serde_json::to_string_pretty(result).unwrap_or_default(),
    );
}

/// Build a job matrix without executing (for testing and inspection).
pub fn expand_matrix(config: &RunConfig) -> Vec<(ProviderName, String)> {
    let mut matrix = Vec::new();
    for provider in &config.providers {
        for perspective_id in &config.perspectives {
            matrix.push((provider.clone(), perspective_id.clone()));
        }
    }
    matrix
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_job_matrix() {
        let config = RunConfig {
            session_id: "test".to_string(),
            prompt: "What is Rust?".to_string(),
            providers: vec![ProviderName::Claude, ProviderName::Codex],
            perspectives: vec!["default".to_string(), "adversarial".to_string()],
            working_directory: None,
            context_paths: vec![],
            timeout_secs: 60,
            max_concurrent: 4,
        };

        let jobs = build_job_matrix(&config, None);
        assert_eq!(jobs.len(), 4);

        // Verify all combinations
        let combos: Vec<(String, String)> = jobs
            .iter()
            .map(|j| {
                let pname = match j.provider {
                    ProviderName::Claude => "claude",
                    ProviderName::Codex => "codex",
                    ProviderName::Gemini => "gemini",
                };
                (pname.to_string(), j.perspective_id.clone())
            })
            .collect();

        assert!(combos.contains(&("claude".to_string(), "default".to_string())));
        assert!(combos.contains(&("claude".to_string(), "adversarial".to_string())));
        assert!(combos.contains(&("codex".to_string(), "default".to_string())));
        assert!(combos.contains(&("codex".to_string(), "adversarial".to_string())));
    }

    #[test]
    fn test_expand_matrix() {
        let config = RunConfig {
            session_id: "test".to_string(),
            prompt: "test".to_string(),
            providers: vec![
                ProviderName::Claude,
                ProviderName::Codex,
                ProviderName::Gemini,
            ],
            perspectives: vec!["default".to_string(), "creative".to_string()],
            working_directory: None,
            context_paths: vec![],
            timeout_secs: 60,
            max_concurrent: 4,
        };

        let matrix = expand_matrix(&config);
        assert_eq!(matrix.len(), 6);
    }

    #[test]
    fn test_build_job_matrix_with_context() {
        let config = RunConfig {
            session_id: "test".to_string(),
            prompt: "Analyze this code".to_string(),
            providers: vec![ProviderName::Claude],
            perspectives: vec!["default".to_string()],
            working_directory: Some("/some/dir".to_string()),
            context_paths: vec![],
            timeout_secs: 60,
            max_concurrent: 4,
        };

        let jobs = build_job_matrix(&config, Some("file contents here"));
        assert_eq!(jobs.len(), 1);
        assert!(jobs[0].prompt.contains("<context>"));
        assert!(jobs[0].prompt.contains("file contents here"));
    }
}
