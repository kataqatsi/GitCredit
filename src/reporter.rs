use std::path::Path;
use std::process::Command;

use sha2::{Digest, Sha256};

use crate::config::{self, Config, DEFAULT_API_URL};
use crate::git_context;
use crate::git_exec;

const REPORT_PATH: &str = "/v1/contribution/create_contribution";

#[derive(Debug, thiserror::Error)]
pub enum ReportError {
    #[error("invalid API URL")]
    InvalidUrl,
    #[error("HTTP error: {0}")]
    Http(String),
}

/// After a successful `git push`, record one heatmap event when an API key is configured.
pub fn maybe_record_after_git(
    cfg: &Config,
    git_workdir: &Path,
    git_argv: &[String],
    exit_code: i32,
) -> Result<(), ReportError> {
    if exit_code != 0 {
        return Ok(());
    }
    if git_context::first_git_subcommand(git_argv) != Some("push") {
        return Ok(());
    }
    record_contribution(cfg, git_workdir)
}

/// `GET {api_url}/v1/contribution/create_contribution?api_key=...&repository_id=...`
pub fn record_contribution(cfg: &Config, git_workdir: &Path) -> Result<(), ReportError> {
    if !cfg.reporting_enabled() {
        if !cfg.has_api_key() {
            config::print_missing_api_key_hint();
        }
        return Ok(());
    }

    let base = cfg.api_url.as_deref().unwrap_or(DEFAULT_API_URL);
    let api_key = cfg.api_key.as_deref().unwrap_or_default();
    let url = join_url(base, REPORT_PATH)?;

    let repository_id = resolve_repository_id(git_workdir);

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| ReportError::Http(e.to_string()))?;

    let mut req = client.get(&url).query(&[("api_key", api_key)]);
    if let Some(ref repo) = repository_id {
        req = req.query(&[("repository_id", repo.as_str())]);
    }

    let resp = req.send().map_err(|e| ReportError::Http(e.to_string()))?;

    if resp.status().is_success() {
        println!("gitcredit activity +1");
        Ok(())
    } else {
        Err(ReportError::Http(format!(
            "{} {}",
            resp.status(),
            resp.text().unwrap_or_default()
        )))
    }
}

/// Per-repo bucket: optional `gitcredit.repositoryId` in that repo's git config, else `origin`
/// URL, else a stable fingerprint of `git-common-dir` (shared across worktrees of one repo).
fn resolve_repository_id(git_workdir: &Path) -> Option<String> {
    if let Ok(cfg) = git_one_line(
        git_workdir,
        &["config", "--get", "gitcredit.repositoryId"],
    ) {
        let t = cfg.trim();
        if !t.is_empty() {
            return Some(t.to_string());
        }
    }

    if let Ok(origin) = git_one_line(git_workdir, &["remote", "get-url", "origin"]) {
        let u = origin.trim();
        if !u.is_empty() {
            return Some(fingerprint_opaque(u));
        }
    }

    let common = git_one_line(
        git_workdir,
        &["rev-parse", "--path-format=absolute", "--git-common-dir"],
    )
    .ok()?;
    let u = common.trim();
    if u.is_empty() {
        return None;
    }
    Some(fingerprint_opaque(u))
}

fn fingerprint_opaque(s: &str) -> String {
    let digest = Sha256::digest(s.as_bytes());
    digest[..8].iter().map(|b| format!("{b:02x}")).collect()
}

fn join_url(base: &str, path: &str) -> Result<String, ReportError> {
    let base = base.trim_end_matches('/');
    if base.is_empty() {
        return Err(ReportError::InvalidUrl);
    }
    Ok(format!("{base}{path}"))
}

fn git_one_line(cwd: &Path, args: &[&str]) -> Result<String, ()> {
    let out = Command::new(git_exec::git_program())
        .current_dir(cwd)
        .args(args)
        .output()
        .map_err(|_| ())?;
    if !out.status.success() {
        return Err(());
    }
    let s = String::from_utf8_lossy(&out.stdout).trim().to_owned();
    if s.is_empty() {
        Err(())
    } else {
        Ok(s)
    }
}
