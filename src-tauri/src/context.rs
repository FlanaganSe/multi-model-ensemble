use serde::Serialize;
use std::fs;
use std::path::Path;

/// Maximum total context size in bytes before truncation (50KB default).
const MAX_CONTEXT_BYTES: u64 = 50 * 1024;

/// A file entry in the context manifest.
#[derive(Debug, Clone, Serialize)]
pub struct ContextFileEntry {
    pub path: String,
    pub bytes: u64,
    pub included: bool,
    pub truncated: bool,
}

/// Manifest recording what was included/excluded in the context pack.
#[derive(Debug, Clone, Serialize)]
pub struct ContextManifest {
    pub working_directory: Option<String>,
    pub included_files: Vec<ContextFileEntry>,
    pub total_bytes: u64,
    pub truncated: bool,
}

/// A context pack: the assembled content string plus its manifest.
#[derive(Debug, Clone, Serialize)]
pub struct ContextPack {
    pub content: String,
    pub manifest: ContextManifest,
}

/// Build a context pack from a list of file paths.
///
/// Files are read, concatenated with path headers, and a manifest is produced.
/// If total size exceeds MAX_CONTEXT_BYTES, later files are excluded.
pub fn build_context_pack(
    file_paths: &[String],
    working_directory: Option<&str>,
) -> Result<ContextPack, Box<dyn std::error::Error>> {
    let mut content = String::new();
    let mut entries = Vec::new();
    let mut total_bytes: u64 = 0;
    let mut pack_truncated = false;

    for path_str in file_paths {
        let path = Path::new(path_str);

        if !path.exists() {
            entries.push(ContextFileEntry {
                path: path_str.clone(),
                bytes: 0,
                included: false,
                truncated: false,
            });
            continue;
        }

        if path.is_dir() {
            // For directories, include a file listing instead of contents
            let listing = build_directory_listing(path)?;
            let listing_bytes = listing.len() as u64;

            if total_bytes + listing_bytes > MAX_CONTEXT_BYTES {
                entries.push(ContextFileEntry {
                    path: path_str.clone(),
                    bytes: listing_bytes,
                    included: false,
                    truncated: false,
                });
                pack_truncated = true;
                continue;
            }

            content.push_str(&format!("--- {path_str} (directory listing) ---\n"));
            content.push_str(&listing);
            content.push('\n');
            total_bytes += listing_bytes;

            entries.push(ContextFileEntry {
                path: path_str.clone(),
                bytes: listing_bytes,
                included: true,
                truncated: false,
            });
        } else {
            match fs::read_to_string(path) {
                Ok(file_content) => {
                    let file_bytes = file_content.len() as u64;

                    if total_bytes + file_bytes > MAX_CONTEXT_BYTES {
                        entries.push(ContextFileEntry {
                            path: path_str.clone(),
                            bytes: file_bytes,
                            included: false,
                            truncated: false,
                        });
                        pack_truncated = true;
                        continue;
                    }

                    content.push_str(&format!("--- {path_str} ---\n"));
                    content.push_str(&file_content);
                    if !file_content.ends_with('\n') {
                        content.push('\n');
                    }
                    total_bytes += file_bytes;

                    entries.push(ContextFileEntry {
                        path: path_str.clone(),
                        bytes: file_bytes,
                        included: true,
                        truncated: false,
                    });
                }
                Err(_) => {
                    // Binary file or read error — skip
                    entries.push(ContextFileEntry {
                        path: path_str.clone(),
                        bytes: 0,
                        included: false,
                        truncated: false,
                    });
                }
            }
        }
    }

    Ok(ContextPack {
        content,
        manifest: ContextManifest {
            working_directory: working_directory.map(|s| s.to_string()),
            included_files: entries,
            total_bytes,
            truncated: pack_truncated,
        },
    })
}

/// Build a simple file listing for a directory (non-recursive, top-level only).
fn build_directory_listing(dir: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let mut lines = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        let ft = entry.file_type()?;
        if ft.is_dir() {
            lines.push(format!("  {name}/"));
        } else {
            let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
            lines.push(format!("  {name} ({size} bytes)"));
        }
    }
    lines.sort();
    Ok(lines.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_build_context_pack_with_files() {
        let tmp = tempfile::tempdir().unwrap();
        let file1 = tmp.path().join("hello.txt");
        let file2 = tmp.path().join("world.txt");
        fs::write(&file1, "Hello").unwrap();
        fs::write(&file2, "World").unwrap();

        let paths = vec![
            file1.to_string_lossy().to_string(),
            file2.to_string_lossy().to_string(),
        ];

        let pack = build_context_pack(&paths, None).unwrap();
        assert!(pack.content.contains("Hello"));
        assert!(pack.content.contains("World"));
        assert_eq!(pack.manifest.included_files.len(), 2);
        assert!(pack.manifest.included_files[0].included);
        assert!(pack.manifest.included_files[1].included);
        assert!(!pack.manifest.truncated);
    }

    #[test]
    fn test_build_context_pack_missing_file() {
        let paths = vec!["/nonexistent/file.txt".to_string()];
        let pack = build_context_pack(&paths, None).unwrap();
        assert!(pack.content.is_empty());
        assert_eq!(pack.manifest.included_files.len(), 1);
        assert!(!pack.manifest.included_files[0].included);
    }

    #[test]
    fn test_build_context_pack_with_directory() {
        let tmp = tempfile::tempdir().unwrap();
        let sub = tmp.path().join("subdir");
        fs::create_dir(&sub).unwrap();
        fs::write(sub.join("a.txt"), "aaa").unwrap();
        fs::write(sub.join("b.txt"), "bbb").unwrap();

        let paths = vec![sub.to_string_lossy().to_string()];
        let pack = build_context_pack(&paths, None).unwrap();
        assert!(pack.content.contains("directory listing"));
        assert!(pack.content.contains("a.txt"));
        assert!(pack.content.contains("b.txt"));
    }

    #[test]
    fn test_build_context_pack_working_directory() {
        let paths: Vec<String> = vec![];
        let pack = build_context_pack(&paths, Some("/some/dir")).unwrap();
        assert_eq!(
            pack.manifest.working_directory,
            Some("/some/dir".to_string())
        );
    }
}
