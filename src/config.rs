use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use colored::Colorize;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct RunConfig {
    /// Primary accent color: "electric-blue" (default), "violet", "emerald", "amber", "rose", "cyan", or custom hex "#RRGGBB"
    pub primary_color: Option<String>,
    /// Auto clear terminal screen before commands and interactive menus
    pub auto_clear: Option<bool>,
    /// Compact mode: sleek minimalist layout without large ASCII banners
    pub compact_mode: Option<bool>,
    /// Preferred default editor: "antigravity", "cursor", "vscode", "xcode", "terminal", or "ask"
    pub default_ide: Option<String>,
    /// Additional custom directory paths to scan for projects
    pub custom_hubs: Option<Vec<PathBuf>>,
    /// Custom command shortcuts / aliases
    pub aliases: Option<BTreeMap<String, String>>,
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

# Automatically clear terminal screen before running commands and interactive menus
auto_clear = false

# Compact mode (sleek minimalist layout without large ASCII banners)
compact_mode = false

# Default editor to open projects directly without prompting:
# Options: "antigravity", "cursor", "vscode", "xcode", "terminal", "ask" (default)
default_ide = "ask"

# Additional custom directory hubs to scan for projects in `run project`:
custom_hubs = [
    # "~/Developer/Summit",
    # "~/Work"
]

# Custom developer aliases and shortcuts:
[aliases]
# c = "cargo check"
# gs = "git status"
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

/// Retrieves all registered custom aliases
pub fn get_aliases() -> BTreeMap<String, String> {
    load_config().aliases.unwrap_or_default()
}

/// Registers or updates a custom alias
pub fn set_alias(name: &str, target: &str) -> Result<()> {
    let mut cfg = load_config();
    let mut aliases = cfg.aliases.unwrap_or_default();
    aliases.insert(name.trim().to_string(), target.trim().to_string());
    cfg.aliases = Some(aliases);
    save_config(&cfg)
}

/// Removes a custom alias
pub fn remove_alias(name: &str) -> Result<bool> {
    let mut cfg = load_config();
    let mut aliases = cfg.aliases.unwrap_or_default();
    if aliases.remove(name.trim()).is_some() {
        cfg.aliases = Some(aliases);
        save_config(&cfg)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CommandStats {
    pub total_runs: u64,
    pub first_used: Option<String>,
    pub last_used: Option<String>,
    pub command_counts: BTreeMap<String, u64>,
}

pub fn stats_path() -> Result<PathBuf> {
    Ok(config_dir()?.join("stats.json"))
}

pub fn load_command_stats() -> CommandStats {
    let Ok(path) = stats_path() else {
        return CommandStats::default();
    };
    if !path.exists() {
        return CommandStats::default();
    }
    match fs::read_to_string(path) {
        Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
        Err(_) => CommandStats::default(),
    }
}

pub fn record_command_stat(cmd_name: &str) {
    let Ok(path) = stats_path() else {
        return;
    };
    let mut stats = load_command_stats();
    stats.total_runs = stats.total_runs.saturating_add(1);
    let now = chrono_now_str();
    if stats.first_used.is_none() {
        stats.first_used = Some(now.clone());
    }
    stats.last_used = Some(now);

    let count = stats.command_counts.entry(cmd_name.to_string()).or_insert(0);
    *count = count.saturating_add(1);

    if let Ok(dir) = config_dir() {
        let _ = fs::create_dir_all(dir);
        if let Ok(serialized) = serde_json::to_string_pretty(&stats) {
            let _ = fs::write(path, serialized);
        }
    }
}

fn chrono_now_str() -> String {
    // Human readable local timestamp without extra dependencies
    let output = std::process::Command::new("date")
        .arg("+%Y-%m-%d %H:%M:%S")
        .output();
    if let Ok(out) = output {
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    } else {
        "Unknown".to_string()
    }
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
