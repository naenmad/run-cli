use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use colored::Colorize;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct RunConfig {
    /// Primary accent color: "electric-blue" (default), "violet", "emerald", "amber", "rose", "cyan", or custom hex "#RRGGBB"
    pub primary_color: Option<String>,
    /// Auto clear terminal screen before interactive menus
    pub auto_clear: Option<bool>,
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
            let default_template = r##"# ==============================================================================
# run-cli Configuration (~/.config/run/config.toml)
# ==============================================================================

# Primary accent color for CLI theme and pointers:
# Presets: "electric-blue" (default), "violet", "emerald", "amber", "rose", "cyan"
# Or use any custom HEX color code: "#ff007f", "#8b5cf6", "#10b981", "#00a2ff"
primary_color = "electric-blue"

# Automatically clear terminal screen before interactive menus
auto_clear = false

# Default editor to open projects directly without prompting:
# Options: "antigravity", "cursor", "vscode", "xcode", "terminal", "ask" (default)
default_ide = "ask"

# Additional custom directory hubs to scan for projects in `run project`:
custom_hubs = [
    # "~/Developer/Summit",
    # "~/Work"
]
"##;
            let _ = fs::write(&path, default_template);
        }
        return RunConfig::default();
    }

    match fs::read_to_string(&path) {
        Ok(content) => toml::from_str(&content).unwrap_or_default(),
        Err(_) => RunConfig::default(),
    }
}

/// Saves the given configuration back to ~/.config/run/config.toml
pub fn save_config(config: &RunConfig) -> Result<()> {
    let dir = config_dir()?;
    if !dir.exists() {
        fs::create_dir_all(&dir)?;
    }
    let path = config_path()?;
    let serialized = toml::to_string_pretty(config).context("failed to serialize configuration")?;
    fs::write(path, serialized).context("failed to write configuration file")?;
    Ok(())
}

/// Parses a preset color name or arbitrary hex code ("#RRGGBB") into RGB (u8, u8, u8)
pub fn parse_color(name_or_hex: &str) -> (u8, u8, u8) {
    let clean = name_or_hex.trim().to_lowercase();
    if clean.starts_with('#') {
        let hex = clean.trim_start_matches('#');
        if hex.len() == 6
            && let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(&hex[0..2], 16),
                u8::from_str_radix(&hex[2..4], 16),
                u8::from_str_radix(&hex[4..6], 16),
            )
        {
            return (r, g, b);
        }
    }

    match clean.as_str() {
        "violet" | "purple" => (139, 92, 246),
        "emerald" | "green" => (16, 185, 129),
        "amber" | "orange" => (245, 158, 11),
        "rose" | "pink" | "red" => (244, 63, 94),
        "cyan" | "aqua" => (6, 182, 212),
        "yellow" => (234, 179, 8),
        // Default Electric Blue (#00a2ff)
        _ => (0, 162, 255),
    }
}

/// Maps an RGB value to the nearest ANSI 256 color code
pub fn rgb_to_ansi256(r: u8, g: u8, b: u8) -> u8 {
    let r_idx = (r as f32 / 255.0 * 5.0).round() as u8;
    let g_idx = (g as f32 / 255.0 * 5.0).round() as u8;
    let b_idx = (b as f32 / 255.0 * 5.0).round() as u8;
    16 + 36 * r_idx + 6 * g_idx + b_idx
}

/// Returns the active primary RGB configured by the user
pub fn get_primary_rgb() -> (u8, u8, u8) {
    let cfg = load_config();
    let col = cfg
        .primary_color
        .unwrap_or_else(|| "electric-blue".to_string());
    parse_color(&col)
}

/// Returns the nearest ANSI 256 code for the active primary color
pub fn get_primary_color256() -> u8 {
    let (r, g, b) = get_primary_rgb();
    rgb_to_ansi256(r, g, b)
}

/// Applies the active configured primary color to a text string
pub fn primary_colored(text: &str) -> colored::ColoredString {
    let (r, g, b) = get_primary_rgb();
    text.truecolor(r, g, b)
}
