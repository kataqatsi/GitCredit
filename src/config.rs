use std::env;
use std::fs;
use std::path::PathBuf;

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

pub const DEFAULT_API_URL: &str =
    "https://adodsq6tonhgvzolgsaymofinu0kvtwf.lambda-url.us-east-1.on.aws";

const ENV_API_URL: &str = "GITCREDIT_API_URL";
const ENV_API_KEY: &str = "GITCREDIT_API_KEY";
/// Legacy env name (same value as API key from the web app settings).
const ENV_TOKEN: &str = "GITCREDIT_TOKEN";

#[derive(Debug, Clone, Default)]
pub struct Config {
    pub api_url: Option<String>,
    pub api_key: Option<String>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct FileConfig {
    api_url: Option<String>,
    api_key: Option<String>,
    /// Older config files used `token` for the API key.
    #[serde(skip_serializing_if = "Option::is_none")]
    token: Option<String>,
}

impl Config {
    pub fn load() -> Self {
        let mut cfg = Self {
            api_url: Some(DEFAULT_API_URL.to_owned()),
            api_key: None,
        };

        if let Ok(url) = env::var(ENV_API_URL) {
            let url = url.trim().to_owned();
            if !url.is_empty() {
                cfg.api_url = Some(url);
            }
        }

        let env_key = env::var(ENV_API_KEY)
            .ok()
            .filter(|s| !s.trim().is_empty())
            .or_else(|| env::var(ENV_TOKEN).ok().filter(|s| !s.trim().is_empty()));

        if let Some(key) = env_key {
            cfg.api_key = Some(key.trim().to_owned());
        }

        if let Some(path) = config_file_path() {
            if let Ok(raw) = fs::read_to_string(&path) {
                if let Ok(file) = toml::from_str::<FileConfig>(&raw) {
                    if let Some(url) = file.api_url {
                        let url = url.trim().to_owned();
                        if !url.is_empty() {
                            cfg.api_url = Some(url);
                        }
                    }
                    if cfg.api_key.is_none() {
                        let key = file
                            .api_key
                            .or(file.token)
                            .map(|k| k.trim().to_owned())
                            .filter(|k| !k.is_empty());
                        cfg.api_key = key;
                    }
                }
            }
        }

        cfg
    }

    pub fn has_api_key(&self) -> bool {
        self.api_key
            .as_ref()
            .is_some_and(|k| !k.trim().is_empty())
    }

    pub fn reporting_enabled(&self) -> bool {
        self.api_url.as_ref().is_some_and(|u| !u.is_empty()) && self.has_api_key()
    }
}

pub fn print_missing_api_key_hint() {
    eprintln!(
        "gitcredit: no API key configured. Run:\n  \
         gitcredit configure api-key <paste-from-web-app>\n  \
         or set GITCREDIT_API_KEY."
    );
}

pub fn config_file_path() -> Option<PathBuf> {
    ProjectDirs::from("dev", "sinewax", "gitcredit")
        .map(|dirs| dirs.config_dir().join("config.toml"))
}

pub fn save_api_key(api_key: &str) -> Result<(), String> {
    let api_key = api_key.trim();
    if api_key.is_empty() {
        return Err("API key cannot be empty".to_owned());
    }

    let path = config_file_path().ok_or_else(|| {
        "Could not resolve config directory (set HOME or use a standard OS user profile)".to_owned()
    })?;

    let mut file = if path.exists() {
        let raw = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        toml::from_str::<FileConfig>(&raw).unwrap_or_default()
    } else {
        FileConfig::default()
    };

    if file
        .api_url
        .as_ref()
        .is_none_or(|u| u.trim().is_empty())
    {
        file.api_url = Some(DEFAULT_API_URL.to_owned());
    }
    file.api_key = Some(api_key.to_owned());
    file.token = None;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let body = toml::to_string_pretty(&file).map_err(|e| e.to_string())?;
    fs::write(&path, body).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn mask_api_key(key: &str) -> String {
    let key = key.trim();
    if key.len() <= 8 {
        return "****".to_owned();
    }
    format!("{}…{}", &key[..4], &key[key.len() - 4..])
}
