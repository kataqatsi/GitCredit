use std::path::{Path, PathBuf};

/// Path to the real `git` executable for read-only introspection (`config`, `remote`, `rev-parse`).
/// Set `GITCREDIT_GIT` if `git` on `PATH` is a shim that must be bypassed.
pub fn git_program() -> PathBuf {
    if let Ok(p) = std::env::var("GITCREDIT_GIT") {
        let t = p.trim();
        if !t.is_empty() {
            return PathBuf::from(t);
        }
    }
    find_distinct_git_on_path().unwrap_or_else(|| PathBuf::from("git"))
}

fn find_distinct_git_on_path() -> Option<PathBuf> {
    let me = std::env::current_exe()
        .ok()
        .and_then(|p| std::fs::canonicalize(p).ok());
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let candidate = dir.join("git");
        if !is_runnable_file(&candidate) {
            continue;
        }
        let Ok(canonical) = std::fs::canonicalize(&candidate) else {
            continue;
        };
        if let Some(ref self_exe) = me {
            if canonical == *self_exe {
                continue;
            }
        }
        return Some(candidate);
    }
    None
}

fn is_runnable_file(path: &Path) -> bool {
    match std::fs::metadata(path) {
        Ok(m) if m.is_file() => {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                return m.permissions().mode() & 0o111 != 0;
            }
            #[cfg(not(unix))]
            {
                return true;
            }
        }
        Ok(m) if m.file_type().is_symlink() => true,
        _ => false,
    }
}
