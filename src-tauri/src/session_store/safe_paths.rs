use std::path::{Path, PathBuf};

/// The app-owned session root directory. All session operations
/// must target paths under this root. This is the safety boundary
/// that prevents the app from modifying arbitrary filesystem locations.
pub struct SessionRoot {
    root: PathBuf,
}

impl SessionRoot {
    /// Resolve the session root under the platform app-data directory.
    /// On macOS: ~/Library/Application Support/com.multimodel.synthesizer/
    pub fn resolve() -> Result<Self, Box<dyn std::error::Error>> {
        let base =
            dirs_next::data_dir().ok_or("Could not determine platform app-data directory")?;
        let root = base.join("com.multimodel.synthesizer");
        std::fs::create_dir_all(&root)?;
        Ok(Self { root })
    }

    /// Create a SessionRoot from an explicit path (for testing).
    #[cfg(test)]
    pub fn from_path(path: PathBuf) -> Self {
        Self { root: path }
    }

    /// Get the sessions subdirectory path.
    pub fn sessions_dir(&self) -> PathBuf {
        self.root.join("sessions")
    }

    /// Get the path for a specific session by ID.
    pub fn session_path(&self, session_id: &str) -> PathBuf {
        self.sessions_dir().join(session_id)
    }

    /// The root path.
    pub fn root_path(&self) -> &Path {
        &self.root
    }

    /// Assert that a target path is within the session root after canonicalization.
    /// This is a read-only safety check — it does not create directories.
    ///
    /// For paths that don't exist yet (e.g., new session dirs), we canonicalize
    /// the parent directory and verify the result is within root.
    pub fn assert_within_root(&self, target: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let canonical_root = self.root.canonicalize().map_err(|e| {
            format!(
                "Cannot canonicalize session root {}: {e}",
                self.root.display()
            )
        })?;

        // Try to canonicalize the target directly. If it doesn't exist,
        // canonicalize its parent and append the filename.
        let canonical_target = if target.exists() {
            target.canonicalize()?
        } else {
            let parent = target
                .parent()
                .ok_or_else(|| format!("No parent directory for {}", target.display()))?;

            if !parent.exists() {
                return Err(format!("Parent directory {} does not exist", parent.display()).into());
            }

            let parent_canonical = parent.canonicalize()?;
            let file_name = target
                .file_name()
                .ok_or_else(|| format!("No file name component in {}", target.display()))?;
            parent_canonical.join(file_name)
        };

        if !canonical_target.starts_with(&canonical_root) {
            return Err(format!(
                "Path {} is outside session root {}",
                canonical_target.display(),
                canonical_root.display()
            )
            .into());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_session_path_is_within_root() {
        let tmp = tempfile::tempdir().unwrap();
        let root = SessionRoot::from_path(tmp.path().to_path_buf());
        fs::create_dir_all(root.sessions_dir()).unwrap();

        let session_path = root.session_path("test-session-123");
        assert!(root.assert_within_root(&session_path).is_ok());
    }

    #[test]
    fn test_path_traversal_rejected() {
        let tmp = tempfile::tempdir().unwrap();
        let root = SessionRoot::from_path(tmp.path().to_path_buf());
        fs::create_dir_all(root.sessions_dir()).unwrap();

        // Attempt path traversal
        let evil_path = root.sessions_dir().join("..").join("..").join("etc");
        assert!(root.assert_within_root(&evil_path).is_err());
    }

    #[test]
    fn test_absolute_path_outside_root_rejected() {
        let tmp = tempfile::tempdir().unwrap();
        let root = SessionRoot::from_path(tmp.path().to_path_buf());

        let outside_path = PathBuf::from("/tmp/evil-directory");
        assert!(root.assert_within_root(&outside_path).is_err());
    }

    #[test]
    fn test_symlink_outside_root_rejected() {
        let tmp = tempfile::tempdir().unwrap();
        let root = SessionRoot::from_path(tmp.path().to_path_buf());
        let sessions = root.sessions_dir();
        fs::create_dir_all(&sessions).unwrap();

        // Create a symlink pointing outside root
        let outside_dir = tempfile::tempdir().unwrap();
        let link_path = sessions.join("sneaky-link");
        std::os::unix::fs::symlink(outside_dir.path(), &link_path).unwrap();

        assert!(root.assert_within_root(&link_path).is_err());
    }

    #[test]
    fn test_valid_nested_path_accepted() {
        let tmp = tempfile::tempdir().unwrap();
        let root = SessionRoot::from_path(tmp.path().to_path_buf());
        let sessions = root.sessions_dir();
        fs::create_dir_all(&sessions).unwrap();

        // Parent exists after we create sessions dir; nested path is within root
        assert!(root
            .assert_within_root(&sessions.join("session-abc"))
            .is_ok());
    }
}
