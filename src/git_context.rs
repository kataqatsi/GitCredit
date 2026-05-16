use std::path::{Path, PathBuf};

/// Parse leading git global options (`-C`, `-c`, etc.). Returns resolved working tree and index of
/// the first argument that is not a handled global (usually the git subcommand).
pub fn parse_leading_git_globals(process_cwd: &Path, argv: &[String]) -> (PathBuf, usize) {
    let mut base = process_cwd.to_path_buf();
    let mut i = 0;

    while i < argv.len() {
        let arg = argv[i].as_str();

        if arg == "-C" && i + 1 < argv.len() {
            base = join_git_c_path(&base, Path::new(&argv[i + 1]));
            i += 2;
            continue;
        }

        if let Some(rest) = arg.strip_prefix("-C=") {
            base = join_git_c_path(&base, Path::new(rest));
            i += 1;
            continue;
        }

        if arg == "-c" && i + 1 < argv.len() {
            i += 2;
            continue;
        }

        if arg.starts_with("-c") && arg.contains('=') {
            i += 1;
            continue;
        }

        if arg == "--namespace" && i + 1 < argv.len() {
            i += 2;
            continue;
        }

        if let Some(rest) = arg.strip_prefix("--namespace=") {
            if rest.is_empty() && i + 1 < argv.len() {
                i += 2;
            } else {
                i += 1;
            }
            continue;
        }

        break;
    }

    let base = std::fs::canonicalize(&base).unwrap_or(base);
    (base, i)
}

/// Working directory Git uses after leading global options (`-C`, `-c`, etc.).
pub fn resolve_git_working_directory(process_cwd: &Path, git_argv: &[String]) -> PathBuf {
    parse_leading_git_globals(process_cwd, git_argv).0
}

/// First git subcommand word, e.g. `push` in `git -C x push origin main`.
pub fn first_git_subcommand(argv: &[String]) -> Option<&str> {
    let (_, i) = parse_leading_git_globals(Path::new("."), argv);
    argv.get(i).map(|s| s.as_str())
}

fn join_git_c_path(base: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(path)
    }
}
