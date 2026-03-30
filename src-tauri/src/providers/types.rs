use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProviderName {
    Claude,
    Codex,
    Gemini,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
        let install_hint = match binary_name {
            "claude" => "Install Claude Code: https://code.claude.com/docs/en/getting-started",
            "codex" => "Install Codex CLI: npm install -g @openai/codex",
            "gemini" => "Install Gemini CLI: see https://google-gemini.github.io/gemini-cli/ for install instructions",
            _ => "Install the CLI tool and ensure it is on your PATH",
        };
        Self {
            provider,
            status: ProviderStatus::NotInstalled,
            executable_path: None,
            version: None,
            auth_ready: false,
            blocked_reason: Some(format!(
                "{binary_name} was not found on your PATH. This provider cannot be used until it is installed."
            )),
            remediation: Some(format!(
                "{install_hint}\nAfter installation, relaunch the app or probe again to detect it."
            )),
        }
    }
}
