use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct RunConfig {
    /// Preferred default editor: "antigravity", "cursor", "vscode", "xcode", "terminal", or "ask"
    pub default_ide: Option<String>,
    /// Additional custom directory paths to scan for projects
    pub custom_hubs: Option<Vec<PathBuf>>,
}

/// Returns the path to the run configuration directory (~/.config/run)
pub fn config_dir() -> Result<PathBuf> {
    let home = std::env::var_os("HOME").context("unable to determine HOME directory")?;
    Ok(PathBuf::from(home).join(".config").join("run"))
}

/// Returns the path to ~/.config/run/config.toml
pub fn config_path() -> Result<PathBuf> {
    Ok(config_dir()?.join("config.toml"))
}

/// Loads existing configuration or creates a default template file if missing
pub fn load_config() -> RunConfig {
    let path = match config_path() {
        Ok(p) => p,
        Err(_) => return RunConfig::default(),
    };

    if !path.exists() {
        if let Ok(dir) = config_dir() {
            let _ = fs::create_dir_all(&dir);
            let default_template = r#"# run-cli Configuration (~/.config/run/config.toml)

# Default editor to open projects directly without prompting:
# Options: "antigravity", "cursor", "vscode", "xcode", "terminal", "ask" (default)
# default_ide = "antigravity"

# Additional custom directory hubs to scan for projects in `run project`:
# custom_hubs = [
#     "~/Developer/Summit",
#     "~/Work"
# ]
"#;
            let _ = fs::write(&path, default_template);
        }
        return RunConfig::default();
    }

    match fs::read_to_string(&path) {
        Ok(content) => toml::from_str(&content).unwrap_or_default(),
        Err(_) => RunConfig::default(),
    }
}
