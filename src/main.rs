mod config;
mod git_context;
mod git_exec;
mod reporter;

use std::env;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::process::{self, Command, Stdio};

use crate::config::Config;

fn main() -> process::ExitCode {
    let cfg = Config::load();

    let raw: Vec<String> = env::args().skip(1).collect();

    if raw.len() == 1 && matches!(raw[0].as_str(), "--version" | "-V") {
        print_version();
        return process::ExitCode::SUCCESS;
    }

    if raw.len() == 1 && matches!(raw[0].as_str(), "help" | "--help" | "-h") {
        print_help();
        return process::ExitCode::SUCCESS;
    }

    if matches!(
        raw.first().map(|s| s.as_str()),
        Some("gitcredit-login-path" | "config-path")
    ) {
        if let Some(p) = config::config_file_path() {
            println!("{}", p.display());
        } else {
            eprintln!("gitcredit: could not resolve config directory");
            return process::ExitCode::from(1);
        }
        return process::ExitCode::SUCCESS;
    }

    if matches!(raw.first().map(|s| s.as_str()), Some("configure")) {
        return cmd_configure(&cfg, &raw[1..]);
    }

    if matches!(raw.first().map(|s| s.as_str()), Some("record")) {
        return cmd_record(&cfg, &raw[1..]);
    }

    let passthrough = strip_optional_git_prefix(raw);

    let process_cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let git_workdir = git_context::resolve_git_working_directory(&process_cwd, &passthrough);

    let status = run_git_forward(&passthrough);

    let code = status
        .code()
        .unwrap_or(if status.success() { 0 } else { 1 });

    if let Err(e) = reporter::maybe_record_after_git(&cfg, &git_workdir, &passthrough, code) {
        eprintln!("gitcredit: contribution report failed: {e}");
    }

    u8::try_from(code)
        .map(process::ExitCode::from)
        .unwrap_or(process::ExitCode::from(1))
}

fn cmd_configure(cfg: &Config, args: &[String]) -> process::ExitCode {
    match args.first().map(|s| s.as_str()) {
        None | Some("help") | Some("--help") | Some("-h") => {
            print_configure_help();
            process::ExitCode::SUCCESS
        }
        Some("api-key") => {
            let key = args
                .get(1)
                .map(|s| s.trim().to_owned())
                .filter(|s| !s.is_empty())
                .or_else(read_api_key_from_stdin);
            match key {
                Some(k) => match config::save_api_key(&k) {
                    Ok(()) => {
                        println!(
                            "Saved API key to {}",
                            config::config_file_path()
                                .map(|p| p.display().to_string())
                                .unwrap_or_else(|| "config".to_owned())
                        );
                        let url = cfg
                            .api_url
                            .as_deref()
                            .unwrap_or(config::DEFAULT_API_URL);
                        println!("API URL: {url}");
                        process::ExitCode::SUCCESS
                    }
                    Err(e) => {
                        eprintln!("gitcredit: {e}");
                        process::ExitCode::from(1)
                    }
                },
                None => {
                    eprintln!("gitcredit: no API key provided");
                    process::ExitCode::from(1)
                }
            }
        }
        Some("show") => {
            let url = cfg
                .api_url
                .as_deref()
                .unwrap_or(config::DEFAULT_API_URL);
            println!("API URL: {url}");
            match cfg.api_key.as_deref() {
                Some(k) => println!("API key: {}", config::mask_api_key(k)),
                None => println!("API key: (not set)"),
            }
            if let Some(p) = config::config_file_path() {
                println!("Config: {}", p.display());
            }
            process::ExitCode::SUCCESS
        }
        Some(other) => {
            eprintln!("gitcredit: unknown configure subcommand: {other}");
            print_configure_help();
            process::ExitCode::from(1)
        }
    }
}

fn read_api_key_from_stdin() -> Option<String> {
    let stderr = io::stderr();
    let mut err = stderr.lock();
    let _ = writeln!(err, "Paste your API key from GitCredit settings, then press Enter:");
    let _ = err.flush();
    let mut line = String::new();
    io::stdin().lock().read_line(&mut line).ok()?;
    let key = line.trim().to_owned();
    if key.is_empty() {
        None
    } else {
        Some(key)
    }
}

fn cmd_record(cfg: &Config, args: &[String]) -> process::ExitCode {
    if !cfg.reporting_enabled() {
        eprintln!(
            "gitcredit: reporting is disabled. Run:\n  \
             gitcredit configure api-key <paste-from-web-app>\n  \
             or set GITCREDIT_API_KEY."
        );
        return process::ExitCode::from(1);
    }

    let process_cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let git_workdir = git_context::resolve_git_working_directory(process_cwd.as_path(), args);

    match reporter::record_contribution(cfg, git_workdir.as_path()) {
        Ok(()) => process::ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("gitcredit: {e}");
            process::ExitCode::from(1)
        }
    }
}

fn print_version() {
    println!(
        "gitcredit {} ({})",
        env!("CARGO_PKG_VERSION"),
        option_env!("GIT_COMMIT_SHORT").unwrap_or("dev")
    );
    let _ = Command::new(git_exec::git_program())
        .arg("--version")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();
}

fn print_configure_help() {
    println!(
        "\
Configure GitCredit (API key from web app Settings → API):

  gitcredit configure api-key <key>   Save the pasted API key
  gitcredit configure api-key         Prompt for paste on stdin
  gitcredit configure show              Show saved URL and masked key
"
    );
}

fn print_help() {
    println!(
        "\
gitcredit — forward to git, then record a heatmap event after a successful push.

Usage:
  gitcredit <same as git>          Runs real git; on successful push, reports to the API
                                   when an API key is configured.

  gitcredit configure api-key …    Save API key from the web app (see: gitcredit configure help)

  gitcredit configure show         Show API URL and masked key

  gitcredit record [GIT_GLOBALS]   Record one event without running git (hooks / manual).

  gitcredit config-path            Print path to config.toml.

  gitcredit help                   This text (for `git help`, run: gitcredit help <topic>).

Environment:
  GITCREDIT_API_URL     API base (default: production Lambda URL)
  GITCREDIT_API_KEY     API key from web settings (overrides config file)

  GITCREDIT_GIT         Path to the real `git` binary (avoids wrapper loops).

Tip: alias git=gitcredit  (ensure GITCREDIT_GIT points at system git if needed)
"
    );
}

/// `gitcredit git push` → same as `gitcredit push`
fn strip_optional_git_prefix(mut argv: Vec<String>) -> Vec<String> {
    if argv.len() >= 2 && argv[0] == "git" {
        argv.remove(0);
    }
    argv
}

fn run_git_forward(git_argv: &[String]) -> process::ExitStatus {
    Command::new(git_exec::git_program())
        .args(git_argv)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .unwrap_or_else(|e| {
            eprintln!("gitcredit: failed to execute git: {e}");
            process::exit(127);
        })
}
