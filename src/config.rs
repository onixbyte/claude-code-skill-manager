use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum InstallMode {
    Copy,
    Link,
}

impl std::fmt::Display for InstallMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstallMode::Copy => write!(f, "copy"),
            InstallMode::Link => write!(f, "link"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MarketplaceEntry {
    pub name: String,
    pub url: String,
    #[serde(default)]
    pub branch: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub marketplaces: Vec<MarketplaceEntry>,
    #[serde(default = "default_mode")]
    pub default_mode: InstallMode,
}

fn default_mode() -> InstallMode {
    InstallMode::Copy
}

impl Config {
    pub fn load() -> io::Result<Self> {
        let path = config_path();
        if path.exists() {
            let data = fs::read_to_string(&path)?;
            Ok(serde_json::from_str(&data).unwrap_or_default())
        } else {
            Ok(Config::default())
        }
    }

    pub fn save(&self) -> io::Result<()> {
        let path = config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let data = serde_json::to_string_pretty(self)?;
        fs::write(path, data)
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            marketplaces: Vec::new(),
            default_mode: default_mode(),
        }
    }
}

pub fn ccsm_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("ccsm")
}

fn config_path() -> PathBuf {
    ccsm_dir().join("config.json")
}

pub fn marketplaces_cache_dir() -> PathBuf {
    ccsm_dir().join("marketplaces")
}

pub fn skills_store_dir() -> PathBuf {
    ccsm_dir().join("skills")
}

pub fn claude_skills_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".claude")
        .join("skills")
}
