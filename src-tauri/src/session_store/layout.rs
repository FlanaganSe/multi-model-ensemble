use serde::Serialize;
use std::fs;
use std::path::Path;

/// Canonical subdirectories within a session directory.
const SESSION_SUBDIRS: &[&str] = &[
    "prompts",
    "prompts/perspectives",
    "runs",
    "synthesis",
    "logs",
];

/// Create the canonical session directory layout.
pub fn create_canonical_layout(session_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    for subdir in SESSION_SUBDIRS {
        fs::create_dir_all(session_dir.join(subdir))?;
    }
    Ok(())
}

/// Entry in the session list returned to the frontend.
#[derive(Debug, Clone, Serialize)]
pub struct SessionListEntry {
    pub id: String,
    pub created_at: String,
    pub status: crate::session_store::metadata::SessionStatus,
    pub label: Option<String>,
    pub path: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canonical_layout_creates_all_subdirs() {
        let tmp = tempfile::tempdir().unwrap();
        let session_dir = tmp.path().join("test-session");
        fs::create_dir_all(&session_dir).unwrap();

        create_canonical_layout(&session_dir).unwrap();

        assert!(session_dir.join("prompts").is_dir());
        assert!(session_dir.join("prompts/perspectives").is_dir());
        assert!(session_dir.join("runs").is_dir());
        assert!(session_dir.join("synthesis").is_dir());
        assert!(session_dir.join("logs").is_dir());
    }

    #[test]
    fn test_canonical_layout_is_idempotent() {
        let tmp = tempfile::tempdir().unwrap();
        let session_dir = tmp.path().join("test-session");
        fs::create_dir_all(&session_dir).unwrap();

        create_canonical_layout(&session_dir).unwrap();
        // Second call should not fail
        create_canonical_layout(&session_dir).unwrap();

        assert!(session_dir.join("runs").is_dir());
    }
}
