use std::path::{Path, PathBuf};

/// Reject paths that escape the `base` directory, including via symlinks.
///
/// Ported from `@hyperframes/core/src/safePath.ts`.
pub fn is_safe_path(base: impl AsRef<Path>, target: impl AsRef<Path>) -> bool {
    let base_path = base.as_ref();
    let target_path = target.as_ref();

    let Ok(base_canonical) = std::fs::canonicalize(base_path) else {
        return false;
    };

    let resolved_target = if target_path.is_absolute() {
        target_path.to_path_buf()
    } else {
        base_canonical.join(target_path)
    };

    // If target exists, canonicalize directly
    if let Ok(target_canonical) = std::fs::canonicalize(&resolved_target) {
        return target_canonical.starts_with(&base_canonical);
    }

    // Target does not exist yet (e.g. pending write target).
    // Walk ancestors to find the deepest existing ancestor and verify containment.
    let mut probe = resolved_target.clone();
    let mut trailing = Vec::new();

    loop {
        if let Ok(ancestor_canonical) = std::fs::canonicalize(&probe) {
            // Reconstruct projected target
            let mut final_path = ancestor_canonical;
            for seg in trailing.iter().rev() {
                final_path.push(seg);
            }
            return final_path.starts_with(&base_canonical);
        }

        match probe.file_name() {
            Some(name) => {
                trailing.push(name.to_os_string());
                if !probe.pop() {
                    return false;
                }
            }
            None => return false,
        }
    }
}

/// Resolve `relative_path` against `base` and return absolute PathBuf only if safe.
pub fn resolve_within_project(
    base: impl AsRef<Path>,
    relative_path: impl AsRef<Path>,
) -> Option<PathBuf> {
    let base_path = base.as_ref();
    let target = base_path.join(relative_path);
    if is_safe_path(base_path, &target) {
        Some(target)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_allows_base_directory() {
        let dir = tempdir().unwrap();
        assert!(is_safe_path(dir.path(), dir.path()));
    }

    #[test]
    fn test_allows_nested_existing_path() {
        let dir = tempdir().unwrap();
        let sub = dir.path().join("assets");
        std::fs::create_dir(&sub).unwrap();
        let file = sub.join("logo.png");
        std::fs::write(&file, b"test").unwrap();
        assert!(is_safe_path(dir.path(), &file));
    }

    #[test]
    fn test_allows_not_yet_existing_target() {
        let dir = tempdir().unwrap();
        let target = dir.path().join("new").join("deep").join("file.txt");
        assert!(is_safe_path(dir.path(), &target));
    }

    #[test]
    fn test_rejects_parent_traversal() {
        let dir = tempdir().unwrap();
        let target = dir.path().join("..").join("..").join("etc").join("passwd");
        assert!(!is_safe_path(dir.path(), &target));
    }

    #[cfg(unix)]
    #[test]
    fn test_rejects_symlink_escape() {
        use std::os::unix::fs::symlink;

        let base = tempdir().unwrap();
        let external = tempdir().unwrap();
        let secret = external.path().join("secret.txt");
        std::fs::write(&secret, b"secret").unwrap();

        let link = base.path().join("link");
        if symlink(external.path(), &link).is_ok() {
            assert!(!is_safe_path(base.path(), link.join("secret.txt")));
        }
    }
}
