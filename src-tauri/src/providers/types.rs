use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProviderName {
    Claude,
    Codex,
    Gemini,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProviderStatus {
    Ready,
    NotInstalled,
    NotAuthenticated,
    Error,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProviderProbeResult {
    pub provider: ProviderName,
    pub status: ProviderStatus,
    pub executable_path: Option<String>,
    pub version: Option<String>,
    pub auth_ready: bool,
    pub blocked_reason: Option<String>,
    pub remediation: Option<String>,
}

impl ProviderProbeResult {
    pub fn not_installed(provider: ProviderName, binary_name: &str) -> Self {
        Self {
            provider,
            status: ProviderStatus::NotInstalled,
            executable_path: None,
            version: None,
            auth_ready: false,
            blocked_reason: Some(format!("{binary_name} binary not found in PATH")),
            remediation: Some(format!(
                "Install {binary_name} and ensure it is on your PATH"
            )),
        }
    }
}
