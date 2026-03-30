use super::{discover_binary, run_probe_command};
use crate::providers::types::{ProviderName, ProviderProbeResult, ProviderStatus};

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
            blocked_reason: Some("Gemini auth not active (exit code 41)".to_string()),
            remediation: Some("Run `gemini` interactively and complete the auth flow".to_string()),
        },
        Ok((_, stderr, code)) => ProviderProbeResult {
            provider: ProviderName::Gemini,
            status: ProviderStatus::Error,
            executable_path: Some(path),
            version,
            auth_ready: false,
            blocked_reason: Some(format!("auth probe failed (exit {code}): {stderr}")),
            remediation: Some("Run `gemini` interactively to verify it works".to_string()),
        },
        Err(e) => ProviderProbeResult {
            provider: ProviderName::Gemini,
            status: ProviderStatus::Error,
            executable_path: Some(path),
            version,
            auth_ready: false,
            blocked_reason: Some(e),
            remediation: Some("Gemini probe timed out — verify the CLI is responsive".to_string()),
        },
    }
}
