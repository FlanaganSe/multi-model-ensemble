use super::{discover_binary, run_probe_command, sanitized_env, strip_ansi};
use crate::orchestrator::types::JobSpec;
use crate::providers::types::{ProviderName, ProviderProbeResult, ProviderStatus};
use tokio::process::Command;

const BINARY_NAME: &str = "gemini";

/// Gemini exit code 41 = FatalAuthenticationError.
const AUTH_FAILURE_EXIT_CODE: i32 = 41;

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

    // No `gemini auth status` command exists. Probe auth by running a minimal
    // headless call and checking exit code. Exit 0 = auth OK, exit 41 = auth failure.
    let auth_result = run_probe_command(&path, &["-p", "ok", "--output-format", "json"]).await;

    match auth_result {
        Ok((_, _, 0)) => ProviderProbeResult {
            provider: ProviderName::Gemini,
            status: ProviderStatus::Ready,
            executable_path: Some(path),
            version,
            auth_ready: true,
            blocked_reason: None,
            remediation: None,
        },
        Ok((_, _, code)) if code == AUTH_FAILURE_EXIT_CODE => ProviderProbeResult {
            provider: ProviderName::Gemini,
            status: ProviderStatus::NotAuthenticated,
            executable_path: Some(path),
            version,
            auth_ready: false,
            blocked_reason: Some(
                "Gemini is installed but not authenticated (exit code 41). Runs using Gemini will be skipped."
                    .to_string(),
            ),
            remediation: Some(
                "Open a terminal and run: gemini\nComplete the interactive auth flow, then return here."
                    .to_string(),
            ),
        },
        Ok((_, stderr, code)) => ProviderProbeResult {
            provider: ProviderName::Gemini,
            status: ProviderStatus::Error,
            executable_path: Some(path),
            version,
            auth_ready: false,
            blocked_reason: Some(format!(
                "Gemini auth probe failed (exit {code}). This may be a transient issue or a configuration problem. stderr: {stderr}"
            )),
            remediation: Some(
                "Try running `gemini` interactively to verify it works. If the issue persists, check your Google account auth."
                    .to_string(),
            ),
        },
        Err(e) => ProviderProbeResult {
            provider: ProviderName::Gemini,
            status: ProviderStatus::Error,
            executable_path: Some(path),
            version,
            auth_ready: false,
            blocked_reason: Some(format!(
                "Gemini probe timed out or failed: {e}. The CLI may be unresponsive."
            )),
            remediation: Some(
                "Verify the Gemini CLI is responsive by running `gemini -v` in a terminal. If it hangs, try reinstalling."
                    .to_string(),
            ),
        },
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
