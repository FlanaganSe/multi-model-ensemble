#![cfg(test)]

use crate::orchestrator::types::{JobResult, JobState};
use crate::providers::types::ProviderName;

/// Build a Claude JSON output string.
pub fn claude_json(result_text: &str) -> String {
    serde_json::json!({
        "type": "result",
        "subtype": "success",
        "result": result_text,
        "cost_usd": 0.05,
        "duration_ms": 3500,
        "is_error": false,
        "num_turns": 1
    })
    .to_string()
}

/// Build a Gemini JSON output string.
#[allow(dead_code)]
pub fn gemini_json(response_text: &str) -> String {
    serde_json::json!({
        "response": response_text,
        "stats": {
            "models": {
                "gemini-2.0-flash": {
                    "requests": 1,
                    "errors": 0,
                    "latency": 2500,
                    "tokens": {
                        "prompt": 100,
                        "candidates": 500,
                        "cached": 0,
                        "thoughts": 0,
                        "tool": 0
                    }
                }
            },
            "tools": { "totalCalls": 0, "success": 0, "fail": 0, "decisions": {} },
            "files": { "additions": 0, "removals": 0 }
        }
    })
    .to_string()
}

pub fn make_job_result(
    job_id: &str,
    provider: ProviderName,
    perspective: &str,
    state: JobState,
    stdout: &str,
) -> JobResult {
    JobResult {
        job_id: job_id.to_string(),
        provider,
        perspective_id: perspective.to_string(),
        state: state.clone(),
        started_at: Some("2026-03-30T10:00:00Z".to_string()),
        ended_at: Some("2026-03-30T10:00:05Z".to_string()),
        duration_ms: Some(5000),
        exit_code: if state == JobState::Completed {
            Some(0)
        } else {
            None
        },
        stdout: stdout.to_string(),
        stderr: String::new(),
        blocked_reason: None,
        error: None,
    }
}

/// Two providers, same perspective, both completed — standard multi-provider scenario.
pub fn two_provider_results() -> Vec<JobResult> {
    vec![
        make_job_result(
            "j1",
            ProviderName::Claude,
            "default",
            JobState::Completed,
            &claude_json("## Summary\n\nStrong codebase.\n\n## Recommendations\n\n- Add integration tests\n- Improve error handling"),
        ),
        make_job_result(
            "j2",
            ProviderName::Codex,
            "default",
            JobState::Completed,
            "## Summary\n\nWell-structured code.\n\n## Recommendations\n\n- Add integration tests\n- Document API",
        ),
    ]
}

/// Three providers, one failed — tests incomplete coverage handling.
pub fn three_provider_one_failed() -> Vec<JobResult> {
    vec![
        make_job_result(
            "j1",
            ProviderName::Claude,
            "default",
            JobState::Completed,
            &claude_json("## Analysis\n\nLooks good.\n\n## Risks\n\n- Memory leak potential"),
        ),
        make_job_result(
            "j2",
            ProviderName::Codex,
            "default",
            JobState::Completed,
            "## Analysis\n\nAlternative view.\n\n## Caveats\n\n- Limited sample size",
        ),
        make_job_result("j3", ProviderName::Gemini, "default", JobState::Failed, ""),
    ]
}
