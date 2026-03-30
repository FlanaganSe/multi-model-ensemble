pub mod layout;
pub mod metadata;
pub mod safe_paths;

use layout::SessionListEntry;
use metadata::SessionMetadata;
use safe_paths::SessionRoot;
use std::fs;

/// Create a new session with an optional label.
pub fn create(label: Option<String>) -> Result<SessionListEntry, Box<dyn std::error::Error>> {
    let root = SessionRoot::resolve()?;
    let meta = SessionMetadata::new(label);
    // Ensure sessions directory exists before safety check (assert_within_root
    // requires the parent to exist for canonicalization)
    let sessions_dir = root.sessions_dir();
    fs::create_dir_all(&sessions_dir)?;

    let session_dir = root.session_path(&meta.id);

    // Safety: verify the session path is within the app-owned root before creating
    root.assert_within_root(&session_dir)?;

    fs::create_dir_all(&session_dir)?;
    layout::create_canonical_layout(&session_dir)?;

    let meta_path = session_dir.join("session.json");
    let json = serde_json::to_string_pretty(&meta)?;
    fs::write(&meta_path, json)?;

    log::info!("Created session {} at {}", meta.id, session_dir.display());

    Ok(SessionListEntry {
        id: meta.id,
        created_at: meta.created_at,
        status: meta.status,
        label: meta.label,
        path: session_dir.to_string_lossy().to_string(),
    })
}

/// List all sessions (active and archived).
pub fn list() -> Result<Vec<SessionListEntry>, Box<dyn std::error::Error>> {
    let root = SessionRoot::resolve()?;
    let sessions_dir = root.sessions_dir();

    if !sessions_dir.exists() {
        return Ok(vec![]);
    }

    let mut entries = Vec::new();
    for entry in fs::read_dir(&sessions_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        // Skip entries that resolve outside the session root (e.g., symlinks)
        if root.assert_within_root(&path).is_err() {
            log::warn!(
                "Skipping session at {}: resolves outside session root",
                path.display()
            );
            continue;
        }
        let meta_path = path.join("session.json");
        if !meta_path.exists() {
            continue;
        }
        match fs::read_to_string(&meta_path) {
            Ok(contents) => match serde_json::from_str::<SessionMetadata>(&contents) {
                Ok(meta) => {
                    entries.push(SessionListEntry {
                        id: meta.id,
                        created_at: meta.created_at,
                        status: meta.status,
                        label: meta.label,
                        path: path.to_string_lossy().to_string(),
                    });
                }
                Err(e) => {
                    log::warn!(
                        "Skipping session at {}: invalid metadata: {e}",
                        path.display()
                    );
                }
            },
            Err(e) => {
                log::warn!(
                    "Skipping session at {}: cannot read metadata: {e}",
                    path.display()
                );
            }
        }
    }

    entries.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(entries)
}

/// Archive a session by updating its metadata status.
pub fn archive(session_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let root = SessionRoot::resolve()?;
    let session_dir = root.session_path(session_id);

    root.assert_within_root(&session_dir)?;

    let meta_path = session_dir.join("session.json");
    if !meta_path.exists() {
        return Err(format!("Session {session_id} not found").into());
    }

    let contents = fs::read_to_string(&meta_path)?;
    let mut meta: SessionMetadata = serde_json::from_str(&contents)?;
    meta.status = metadata::SessionStatus::Archived;

    let json = serde_json::to_string_pretty(&meta)?;
    fs::write(&meta_path, json)?;

    log::info!("Archived session {session_id}");
    Ok(())
}

/// Delete a session directory. Only works within the app-owned session root.
pub fn delete(session_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let root = SessionRoot::resolve()?;
    let session_dir = root.session_path(session_id);

    // Critical safety check: canonicalize and verify within root BEFORE deletion
    root.assert_within_root(&session_dir)?;

    if !session_dir.exists() {
        return Err(format!("Session {session_id} not found").into());
    }

    fs::remove_dir_all(&session_dir)?;

    log::info!("Deleted session {session_id}");
    Ok(())
}
