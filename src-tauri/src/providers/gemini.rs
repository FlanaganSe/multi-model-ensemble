use super::{discover_binary, run_probe_command, sanitized_env, strip_ansi};
use crate::orchestrator::types::JobSpec;
use crate::providers::types::{ProviderName, ProviderProbeResult, ProviderStatus};
use tokio::process::Command;

const BINARY_NAME: &str = "gemini";

pub async fn probe() -> ProviderProbeResult {
    let path = match discover_binary(BINARY_NAME).await {
        Some(p) => p,
        None => return ProviderProbeResult::not_installed(ProviderName::Gemini, BINARY_NAME),
    };

    // Gemini `--version` can hang; use `-v` with a tight timeout.
    // The version string is the first line of output.
    let version = match run_probe_command(&path, &["-v"]).await {
        Ok((stdout, _, 0)) => {
            let v = stdout.lines().next().unwrap_or("").trim().to_string();
            if v.is_empty() {
                None
            } else {
                Some(v)
            }
        }
        _ => None,
    };

    // Gemini has no `auth status` command. A live `gemini -p "ok"` probe is
    // unsuitable because it loads MCP servers, makes a real API call, can hit
    // rate limits, and routinely exceeds any reasonable probe timeout.
    //
    // Instead: if `-v` succeeded we mark the provider as Ready and defer auth
    // checking to runtime. If a run fails with exit code 41 the orchestrator
    // will record a blocked state with remediation text at that point.
    if version.is_some() {
        ProviderProbeResult {
            provider: ProviderName::Gemini,
            status: ProviderStatus::Ready,
            executable_path: Some(path),
            version,
            auth_ready: true,
            blocked_reason: None,
            remediation: None,
        }
    } else {
        ProviderProbeResult {
            provider: ProviderName::Gemini,
            status: ProviderStatus::Error,
            executable_path: Some(path),
            version: None,
            auth_ready: false,
            blocked_reason: Some(
                "Gemini is installed but did not respond to version check. The CLI may be misconfigured."
                    .to_string(),
            ),
            remediation: Some(
                "Try running `gemini -v` in a terminal. If it hangs, check your Gemini CLI installation."
                    .to_string(),
            ),
        }
    }
}

/// Execute Gemini in non-interactive mode.
///
/// Perspective injection: uses GEMINI_SYSTEM_MD pointing to the perspective file
/// already persisted by the orchestrator in the session's prompts/ directory.
/// No temp files are created — cleanup is handled by session lifecycle.
///
/// Command: GEMINI_SYSTEM_MD=/path/to/perspective.md gemini -p "<prompt>" --output-format json
pub async fn execute(executable: &str, spec: &JobSpec) -> Result<(String, String, i32), String> {
    let mut cmd = Command::new(executable);

    cmd.arg("-p").arg(&spec.prompt);
    cmd.arg("--output-format").arg("json");
    cmd.arg("--approval-mode").arg("plan");

    // Set working directory if provided
    if let Some(ref cwd) = spec.working_directory {
        cmd.current_dir(cwd);
    }

    // Sanitize environment: strip API keys
    cmd.env_clear();
    for (key, value) in sanitized_env() {
        cmd.env(&key, &value);
    }

    // Set GEMINI_SYSTEM_MD to the perspective file persisted by the orchestrator.
    // This avoids creating temp files that could leak on timeout.
    if let Some(ref persp_file) = spec.perspective_file {
        if !spec.perspective_instructions.is_empty() {
            cmd.env("GEMINI_SYSTEM_MD", persp_file);
        }
    }

    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    let output = cmd
        .output()
        .await
        .map_err(|e| format!("Failed to spawn gemini: {e}"))?;

    let stdout = strip_ansi(&String::from_utf8_lossy(&output.stdout));
    let stderr = strip_ansi(&String::from_utf8_lossy(&output.stderr));
    let exit_code = output.status.code().unwrap_or(-1);

    Ok((stdout, stderr, exit_code))
}
