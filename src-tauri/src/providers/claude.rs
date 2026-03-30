use super::{discover_binary, run_probe_command, sanitized_env, strip_ansi};
use crate::orchestrator::types::JobSpec;
use crate::providers::types::{ProviderName, ProviderProbeResult, ProviderStatus};
use tokio::process::Command;

const BINARY_NAME: &str = "claude";

pub async fn probe() -> ProviderProbeResult {
    let path = match discover_binary(BINARY_NAME).await {
        Some(p) => p,
        None => return ProviderProbeResult::not_installed(ProviderName::Claude, BINARY_NAME),
    };

    let version = match run_probe_command(&path, &["--version"]).await {
        Ok((stdout, _, 0)) => Some(stdout),
        Ok((_, stderr, _)) => {
            return ProviderProbeResult {
                provider: ProviderName::Claude,
                status: ProviderStatus::Error,
                executable_path: Some(path),
                version: None,
                auth_ready: false,
                blocked_reason: Some(format!("version check failed: {stderr}")),
                remediation: Some("Verify claude installation is working".to_string()),
            };
        }
        Err(e) => {
            return ProviderProbeResult {
                provider: ProviderName::Claude,
                status: ProviderStatus::Error,
                executable_path: Some(path),
                version: None,
                auth_ready: false,
                blocked_reason: Some(e),
                remediation: None,
            };
        }
    };

    // Check auth: `claude auth status` exits 0 if logged in
    let auth_ready = match run_probe_command(&path, &["auth", "status"]).await {
        Ok((_, _, 0)) => true,
        Ok((_, _, _)) => false,
        Err(_) => false,
    };

    if auth_ready {
        ProviderProbeResult {
            provider: ProviderName::Claude,
            status: ProviderStatus::Ready,
            executable_path: Some(path),
            version,
            auth_ready: true,
            blocked_reason: None,
            remediation: None,
        }
    } else {
        ProviderProbeResult {
            provider: ProviderName::Claude,
            status: ProviderStatus::NotAuthenticated,
            executable_path: Some(path),
            version,
            auth_ready: false,
            blocked_reason: Some(
                "Claude is installed but not authenticated. Runs using Claude will be skipped."
                    .to_string(),
            ),
            remediation: Some(
                "Open a terminal and run: claude auth login\nThen return here and the status will update on next probe."
                    .to_string(),
            ),
        }
    }
}

/// Execute Claude in non-interactive mode.
///
/// Command: claude -p "<prompt>" --output-format json --permission-mode dontAsk
///          --max-turns 1 --system-prompt "<perspective>" --allowedTools Read Grep Glob
pub async fn execute(executable: &str, spec: &JobSpec) -> Result<(String, String, i32), String> {
    let mut cmd = Command::new(executable);

    cmd.arg("-p")
        .arg(&spec.prompt)
        .arg("--output-format")
        .arg("json")
        .arg("--permission-mode")
        .arg("dontAsk")
        .arg("--max-turns")
        .arg("1")
        .arg("--no-session-persistence");

    // Inject perspective via system prompt
    if !spec.perspective_instructions.is_empty() {
        cmd.arg("--system-prompt")
            .arg(&spec.perspective_instructions);
    }

    // Narrow tool scope to read-only
    cmd.arg("--allowedTools")
        .arg("Read")
        .arg("Grep")
        .arg("Glob");

    // Set working directory if provided
    if let Some(ref cwd) = spec.working_directory {
        cmd.current_dir(cwd);
    }

    // Sanitize environment: strip API keys
    cmd.env_clear();
    for (key, value) in sanitized_env() {
        cmd.env(&key, &value);
    }

    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    let output = cmd
        .output()
        .await
        .map_err(|e| format!("Failed to spawn claude: {e}"))?;

    let stdout = strip_ansi(&String::from_utf8_lossy(&output.stdout));
    let stderr = strip_ansi(&String::from_utf8_lossy(&output.stderr));
    let exit_code = output.status.code().unwrap_or(-1);

    Ok((stdout, stderr, exit_code))
}
