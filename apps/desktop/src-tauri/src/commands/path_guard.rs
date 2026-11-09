use std::path::{Path, PathBuf};

const SECRET_DENYLIST: &[&str] = &[
    ".ssh",
    ".aws",
    ".gnupg",
    ".config",
    ".env",
    "id_rsa",
    "id_dsa",
    "id_ecdsa",
    "id_ed25519",
    "credentials",
    ".netrc",
    ".npmrc",
    ".gitconfig",
    ".docker",
];

/// Validate a frontend-supplied READ path (import/preflight/preview) and return the
/// lexically-normalized absolute path, or an error if the path is empty, escapes its
/// root via `..`, or names a sensitive (secret/credential) location.
///
/// This is a defense-in-depth mitigation against a compromised webview exfiltrating
/// secrets or overwriting system files via the import/export/backup commands. It does
/// not restrict which user directories may be used (legitimate imports from USB drives,
/// external volumes, and arbitrary user folders remain supported) but blocks known
/// secret/credential locations anywhere on the path. Existing paths are canonicalized
/// so symlink targets are re-checked against the denylist. The primary trust boundary
/// remains the OS file dialog.
pub fn validate_import_path(raw: &str) -> Result<PathBuf, String> {
    validate(raw)
}

/// Validate a frontend-supplied WRITE path (export/backup) using the same escape and
/// secret-denylist rules as reads. Existing-file overwrite is intentionally permitted to
/// preserve current export behaviour.
pub fn validate_export_path(raw: &str) -> Result<PathBuf, String> {
    validate(raw)
}

fn validate(raw: &str) -> Result<PathBuf, String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err("Path must not be empty".to_string());
    }

    let input = Path::new(raw);
    let absolute = if input.is_absolute() {
        input.to_path_buf()
    } else {
        let home =
            home_dir().ok_or_else(|| "Unable to determine the user home directory".to_string())?;
        home.join(input)
    };

    let normalized = lexical_normalize(&absolute)
        .ok_or_else(|| "Path escapes its root directory and is not allowed".to_string())?;

    reject_if_secret(&normalized)?;
    resolve_existing_path(normalized)
}

fn resolve_existing_path(normalized: PathBuf) -> Result<PathBuf, String> {
    if !normalized.exists() {
        if let Some(parent) = normalized.parent() {
            if parent.exists() {
                let canonical_parent = std::fs::canonicalize(parent).map_err(|err| {
                    format!("Unable to resolve path '{}': {}", parent.display(), err)
                })?;
                reject_if_secret(&canonical_parent)?;
                if let Some(name) = normalized.file_name() {
                    return Ok(canonical_parent.join(name));
                }
            }
        }
        return Ok(normalized);
    }

    let metadata = std::fs::symlink_metadata(&normalized)
        .map_err(|err| format!("Unable to inspect path '{}': {}", normalized.display(), err))?;
    if metadata.file_type().is_symlink() {
        return Err(format!(
            "Symbolic links are not allowed: {}",
            normalized.display()
        ));
    }

    let canonical = std::fs::canonicalize(&normalized)
        .map_err(|err| format!("Unable to resolve path '{}': {}", normalized.display(), err))?;
    reject_if_secret(&canonical)?;
    Ok(canonical)
}

fn lexical_normalize(absolute: &Path) -> Option<PathBuf> {
    use std::path::Component;

    let mut out = PathBuf::new();
    for component in absolute.components() {
        match component {
            Component::ParentDir => {
                if !out.pop() {
                    return None;
                }
            }
            Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    Some(out)
}

fn reject_if_secret(path: &Path) -> Result<(), String> {
    let lower = path.to_string_lossy().to_lowercase();
    for forbidden in SECRET_DENYLIST {
        if lower.contains(forbidden) {
            return Err(format!(
                "Path refers to a sensitive location and is not allowed: {}",
                path.display()
            ));
        }
    }

    if let Some(name) = path.file_name() {
        let name_lower = name.to_string_lossy().to_lowercase();
        if name_lower.starts_with('.')
            && (name_lower.contains("key")
                || name_lower.contains("cred")
                || name_lower.contains("token")
                || name_lower.contains("secret"))
        {
            return Err(format!(
                "Path refers to a sensitive location and is not allowed: {}",
                path.display()
            ));
        }
    }

    Ok(())
}

fn home_dir() -> Option<PathBuf> {
    #[cfg(unix)]
    {
        std::env::var_os("HOME").map(PathBuf::from)
    }
    #[cfg(windows)]
    {
        std::env::var_os("USERPROFILE").map(PathBuf::from)
    }
    #[cfg(not(any(unix, windows)))]
    {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::{reject_if_secret, resolve_existing_path, validate};
    use std::fs;
    use std::path::PathBuf;

    fn temp_dir(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("900crm-path-guard-{}-{}", name, std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("temp dir");
        dir
    }

    #[test]
    fn rejects_secret_path_fragments() {
        assert!(reject_if_secret(&PathBuf::from("/tmp/home/.ssh/id_rsa")).is_err());
        assert!(reject_if_secret(&PathBuf::from("/tmp/safe-import.csv")).is_ok());
    }

    #[test]
    fn validate_rejects_empty_and_parent_escape() {
        assert!(validate("").is_err());
        assert!(validate("/tmp/../..").is_err());
    }

    #[test]
    fn resolve_existing_path_rejects_symlink_to_secret() {
        let dir = temp_dir("symlink");
        let target = dir.join("id_rsa");
        fs::write(&target, "secret").expect("write target");
        let link = dir.join("import.csv");
        #[cfg(unix)]
        {
            let _ = fs::remove_file(&link);
            std::os::unix::fs::symlink(&target, &link).expect("symlink");
            let err = resolve_existing_path(link).expect_err("symlink should fail");
            assert!(err.contains("Symbolic links") || err.contains("sensitive"));
        }
        let _ = fs::remove_dir_all(&dir);
    }
}
