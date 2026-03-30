use super::{discover_binary, run_probe_command};
use crate::providers::types::{ProviderName, ProviderProbeResult, ProviderStatus};

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
            blocked_reason: Some("Claude auth not active".to_string()),
            remediation: Some("Run `claude auth login` in your terminal".to_string()),
        }
    }
}
