pub mod claude;
pub mod codex;
pub mod gemini;
pub mod types;

use crate::orchestrator::types::JobSpec;
use std::time::Duration;
use tokio::process::Command;
use types::{ProviderName, ProviderProbeResult};

/// Default timeout for probe subcommands (version checks, auth checks).
const PROBE_TIMEOUT: Duration = Duration::from_secs(10);

/// API key environment variables to strip from spawned process environments.
/// Prevents accidental API billing when using subscription-backed CLI tools.
const STRIPPED_ENV_VARS: &[&str] = &[
    "ANTHROPIC_API_KEY",
    "CODEX_API_KEY",
    "GEMINI_API_KEY",
    "OPENAI_API_KEY",
];

/// Discover a CLI binary by asking a login shell for its path.
/// Tauri apps launched from Finder/Dock do not inherit shell PATH,
/// so we must use `/bin/sh -lc "which <name>"` to find binaries.
pub(super) async fn discover_binary(name: &str) -> Option<String> {
    let result = tokio::time::timeout(
        PROBE_TIMEOUT,
        Command::new("/bin/sh")
            .args(["-lc", &format!("which {name}")])
            .output(),
    )
    .await;

    match result {
        Ok(Ok(output)) if output.status.success() => {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if path.is_empty() {
                None
            } else {
                Some(path)
            }
        }
        _ => None,
    }
}

/// Run a command with timeout and return (stdout, stderr, exit_code).
pub(super) async fn run_probe_command(
    program: &str,
    args: &[&str],
) -> Result<(String, String, i32), String> {
    let result =
        tokio::time::timeout(PROBE_TIMEOUT, Command::new(program).args(args).output()).await;

    match result {
        Ok(Ok(output)) => {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let code = output.status.code().unwrap_or(-1);
            Ok((stdout, stderr, code))
        }
        Ok(Err(e)) => Err(format!("failed to execute {program}: {e}")),
        Err(_) => Err(format!("{program} probe timed out after {PROBE_TIMEOUT:?}")),
    }
}

/// Build a sanitized environment for spawned CLI processes.
/// Inherits current env but strips API key variables.
pub(crate) fn sanitized_env() -> Vec<(String, String)> {
    std::env::vars()
        .filter(|(key, _)| !STRIPPED_ENV_VARS.contains(&key.as_str()))
        .collect()
}

/// Strip ANSI escape codes from a string.
pub(crate) fn strip_ansi(s: &str) -> String {
    // Matches ESC [ ... m (SGR), ESC [ ... (CSI), and OSC sequences
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // Skip the escape sequence
            if let Some(&next) = chars.peek() {
                if next == '[' {
                    chars.next(); // consume '['
                                  // Skip until we find a letter (the terminator)
                    while let Some(&ch) = chars.peek() {
                        chars.next();
                        if ch.is_ascii_alphabetic() || ch == '~' {
                            break;
                        }
                    }
                } else if next == ']' {
                    chars.next(); // consume ']'
                                  // OSC: skip until BEL or ST
                    while let Some(&ch) = chars.peek() {
                        chars.next();
                        if ch == '\x07' {
                            break;
                        }
                        if ch == '\x1b' {
                            if chars.peek() == Some(&'\\') {
                                chars.next();
                            }
                            break;
                        }
                    }
                } else {
                    // Skip single char after ESC
                    chars.next();
                }
            }
        } else {
            result.push(c);
        }
    }

    result
}

/// Execute a provider job. Dispatches to the appropriate provider adapter.
/// Returns (stdout, stderr, exit_code).
pub async fn execute(
    provider: &ProviderName,
    probe: &ProviderProbeResult,
    spec: &JobSpec,
) -> Result<(String, String, i32), String> {
    let executable = probe
        .executable_path
        .as_deref()
        .ok_or_else(|| format!("No executable path for {:?}", provider))?;

    match provider {
        ProviderName::Claude => claude::execute(executable, spec).await,
        ProviderName::Codex => codex::execute(executable, spec).await,
        ProviderName::Gemini => gemini::execute(executable, spec).await,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitized_env_strips_api_keys() {
        // Can't easily set env vars in a test without side effects,
        // but we can verify the filter logic works
        let env: Vec<(String, String)> = vec![
            ("PATH".to_string(), "/usr/bin".to_string()),
            ("ANTHROPIC_API_KEY".to_string(), "secret".to_string()),
            ("OPENAI_API_KEY".to_string(), "secret".to_string()),
            ("HOME".to_string(), "/home/user".to_string()),
        ];

        let filtered: Vec<(String, String)> = env
            .into_iter()
            .filter(|(key, _)| !STRIPPED_ENV_VARS.contains(&key.as_str()))
            .collect();

        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|(k, _)| k != "ANTHROPIC_API_KEY"));
        assert!(filtered.iter().all(|(k, _)| k != "OPENAI_API_KEY"));
    }

    #[test]
    fn test_strip_ansi_basic() {
        assert_eq!(strip_ansi("\x1b[32mHello\x1b[0m"), "Hello");
        assert_eq!(strip_ansi("No escapes here"), "No escapes here");
        assert_eq!(
            strip_ansi("\x1b[1;31mRed\x1b[0m and plain"),
            "Red and plain"
        );
    }

    #[test]
    fn test_strip_ansi_empty() {
        assert_eq!(strip_ansi(""), "");
    }
}
