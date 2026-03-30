pub mod claude;
pub mod codex;
pub mod gemini;
pub mod types;

use std::time::Duration;
use tokio::process::Command;

/// Default timeout for probe subcommands (version checks, auth checks).
const PROBE_TIMEOUT: Duration = Duration::from_secs(10);

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
