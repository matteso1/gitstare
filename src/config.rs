use anyhow::Result;
use directories::ProjectDirs;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize, Clone)]
#[serde(default)]
pub struct Config {
    pub scan_paths: Vec<PathBuf>,
    pub max_depth: usize,
    pub ignore: Vec<String>,
    pub stale_threshold: u64,
}

impl Default for Config {
    fn default() -> Self {
        let home = dirs_home();
        Self {
            scan_paths: vec![home],
            max_depth: 4,
            ignore: vec![
                "node_modules".into(),
                "target".into(),
                ".cache".into(),
                "vendor".into(),
                ".cargo".into(),
                ".rustup".into(),
                "AppData".into(),
                ".npm".into(),
                ".nuget".into(),
                "Library".into(),
            ],
            stale_threshold: 30,
        }
    }
}

fn dirs_home() -> PathBuf {
    directories::BaseDirs::new()
        .map(|d| d.home_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."))
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = config_path();
        if path.exists() {
            let contents = std::fs::read_to_string(&path)?;
            let mut cfg: Config = toml::from_str(&contents)?;
            // Expand ~ in scan_paths
            cfg.scan_paths = cfg
                .scan_paths
                .into_iter()
                .map(|p| {
                    let s = p.to_string_lossy();
                    if s.starts_with("~/") || s == "~" {
                        dirs_home().join(s.trim_start_matches("~/").trim_start_matches('~'))
                    } else {
                        p
                    }
                })
                .collect();
            Ok(cfg)
        } else {
            Ok(Config::default())
        }
    }
}

fn config_path() -> PathBuf {
    ProjectDirs::from("", "", "gitstare")
        .map(|d| d.config_dir().join("config.toml"))
        .unwrap_or_else(|| PathBuf::from("gitstare.toml"))
}
