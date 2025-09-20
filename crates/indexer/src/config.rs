use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub roots: Vec<PathBuf>,
    pub global_ignores: Vec<String>,
    pub size_mode: SizeMode,
    pub concurrency: usize,
    pub git: GitConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitConfig {
    pub use_cli_fallback: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SizeMode {
    ExactCached,
    None,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            roots: vec![shellexpand::tilde("~/Code").to_string().into()],
            global_ignores: vec![
                ".git".into(),
                "node_modules".into(),
                "target".into(),
                "build".into(),
                "dist".into(),
                ".venv".into(),
                "Pods".into(),
                "DerivedData".into(),
                ".cache".into(),
            ],
            size_mode: SizeMode::ExactCached,
            concurrency: 8,
            git: GitConfig {
                use_cli_fallback: false,
            },
        }
    }
}

pub struct ConfigStore;

impl ConfigStore {
    pub fn config_dir() -> Result<PathBuf> {
        let dirs = ProjectDirs::from("com.projectbrowser", "Local", "ProjectBrowser")
            .ok_or_else(|| anyhow::anyhow!("could not resolve project dirs"))?;
        Ok(dirs.config_dir().to_path_buf())
    }

    pub fn data_dir() -> Result<PathBuf> {
        let dirs = ProjectDirs::from("com.projectbrowser", "Local", "ProjectBrowser")
            .ok_or_else(|| anyhow::anyhow!("could not resolve project dirs"))?;
        Ok(dirs.data_dir().to_path_buf())
    }

    pub fn config_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("config.json"))
    }

    /// Primary app-level ignore file next to config.json
    pub fn app_ignore_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("ignore"))
    }

    /// Legacy/convenience ignore file: ~/.config/project-browser/ignore
    pub fn user_ignore_path_legacy() -> PathBuf {
        let home = dirs_next::home_dir().unwrap_or_else(|| PathBuf::from("~"));
        home.join(".config").join("project-browser").join("ignore")
    }

    pub fn load() -> Result<AppConfig> {
        let path = Self::config_path()?;
        if path.exists() {
            let s = fs::read_to_string(&path)?;
            let cfg: AppConfig = serde_json::from_str(&s)?;
            Ok(cfg)
        } else {
            Ok(AppConfig::default())
        }
    }

    pub fn save(cfg: &AppConfig) -> Result<()> {
        let dir = Self::config_dir()?;
        fs::create_dir_all(&dir)?;
        let path = dir.join("config.json");
        let s = serde_json::to_string_pretty(cfg)?;
        fs::write(path, s)?;
        Ok(())
    }
}
