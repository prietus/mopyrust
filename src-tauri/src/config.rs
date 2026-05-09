use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub host: String,
    #[serde(default = "default_mpd_port")]
    pub mpd_port: u16,
    #[serde(default = "default_http_port")]
    pub http_port: u16,
    #[serde(default)]
    pub theme: Option<String>,
    #[serde(default)]
    pub lastfm_api_key: Option<String>,
    #[serde(default)]
    pub fanart_api_key: Option<String>,
    #[serde(default)]
    pub discogs_token: Option<String>,
}

fn default_mpd_port() -> u16 { 6600 }
fn default_http_port() -> u16 { 6680 }

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".into(),
            mpd_port: 6600,
            http_port: 6680,
            theme: Some("midnight".into()),
            lastfm_api_key: None,
            fanart_api_key: None,
            discogs_token: None,
        }
    }
}

const TEMPLATE: &str = r#"# mopyrust config
host = "127.0.0.1"
mpd_port = 6600
http_port = 6680

# midnight | soft-dark | daylight | solar
theme = "midnight"

# Optional: API keys for richer metadata (artist bios, similar artists, HD photos).
# Get them from:
#   last.fm:   https://www.last.fm/api/account/create
#   fanart.tv: https://fanart.tv/get-an-api-key/   (login required)
#   discogs:   https://www.discogs.com/settings/developers
lastfm_api_key = ""
fanart_api_key = ""
discogs_token = ""
"#;

pub fn config_path() -> Option<PathBuf> {
    let dirs = directories::ProjectDirs::from("", "", "mopyrust")?;
    Some(dirs.config_dir().join("config.toml"))
}

pub fn load_or_template() -> AppConfig {
    let Some(path) = config_path() else {
        return AppConfig::default();
    };
    if !path.exists() {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&path, TEMPLATE);
        eprintln!("wrote template config at {}", path.display());
        return AppConfig::default();
    }
    match read_config(&path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("config error: {e} — using defaults");
            AppConfig::default()
        }
    }
}

fn read_config(path: &Path) -> Result<AppConfig, String> {
    let raw = std::fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    toml::from_str::<AppConfig>(&raw).map_err(|e| format!("parse: {e}"))
}

pub fn save(cfg: &AppConfig) -> Result<(), String> {
    let path = config_path().ok_or("no config dir")?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("create dir: {e}"))?;
    }
    let body = serialize(cfg);
    std::fs::write(&path, body).map_err(|e| format!("write {}: {e}", path.display()))
}

fn serialize(cfg: &AppConfig) -> String {
    // Hand-formatted to keep the file readable + the comment header intact.
    let theme = cfg.theme.as_deref().unwrap_or("midnight");
    let lastfm = cfg.lastfm_api_key.as_deref().unwrap_or("");
    let fanart = cfg.fanart_api_key.as_deref().unwrap_or("");
    let discogs = cfg.discogs_token.as_deref().unwrap_or("");
    format!(
        "# mopyrust config\n\
host = {host:?}\n\
mpd_port = {mpd}\n\
http_port = {http}\n\n\
# midnight | soft-dark | daylight | solar\n\
theme = {theme:?}\n\n\
# Optional: API keys for richer metadata.\n\
lastfm_api_key = {lastfm:?}\n\
fanart_api_key = {fanart:?}\n\
discogs_token = {discogs:?}\n",
        host = cfg.host,
        mpd = cfg.mpd_port,
        http = cfg.http_port,
        theme = theme,
        lastfm = lastfm,
        fanart = fanart,
        discogs = discogs,
    )
}
