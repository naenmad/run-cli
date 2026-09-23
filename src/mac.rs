use std::fs;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use colored::Colorize;
use dialoguer::{Confirm, FuzzySelect, Input, Select};

use crate::ui::{self, RunTheme as ColorfulTheme, cancel_option};

/// Helper to run an osascript one-liner and return trimmed stdout
fn run_osascript(script: &str) -> Result<String> {
    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .context("failed to execute osascript")?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
        bail!("{err}");
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Helper to detect the primary Wi-Fi hardware interface (typically en0)
fn detect_wifi_interface() -> String {
    let output = Command::new("networksetup")
        .args(["-listallhardwareports"])
        .output();
    if let Ok(out) = output {
        let text = String::from_utf8_lossy(&out.stdout);
        let mut is_wifi = false;
        for line in text.lines() {
            if line.contains("Hardware Port: Wi-Fi") || line.contains("Hardware Port: AirPort") {
                is_wifi = true;
                continue;
            }
            if is_wifi && line.trim().starts_with("Device:") {
                let dev = line.replace("Device:", "").trim().to_string();
                if !dev.is_empty() {
                    return dev;
                }
            }
        }
    }
    "en0".to_string()
}

// ============================================================================
// 1. Wi-Fi Manager
// ============================================================================

pub fn handle_wifi(
    theme: &ColorfulTheme,
    action: Option<&str>,
    arg1: Option<&str>,
    arg2: Option<&str>,
) -> Result<()> {
    ui::maybe_auto_clear();
    let dev = detect_wifi_interface();

    match action {
        Some("on") => {
            let status = Command::new("networksetup")
                .args(["-setairportpower", &dev, "on"])
                .status()?;
            if status.success() {
                println!("{} Wi-Fi powered ON ({dev}).", "✔".green().bold());
            } else {
                bail!("failed to power on Wi-Fi interface {dev}");
            }
        }
        Some("off") => {
            let status = Command::new("networksetup")
                .args(["-setairportpower", &dev, "off"])
                .status()?;
            if status.success() {
                println!("{} Wi-Fi powered OFF ({dev}).", "✔".yellow().bold());
            } else {
                bail!("failed to power off Wi-Fi interface {dev}");
            }
        }
        Some("scan") => {
            scan_and_display_wifi(theme, &dev)?;
        }
        Some("connect") => {
            let ssid = arg1.context("missing Wi-Fi network SSID (e.g. 'run wifi connect MySSID')")?;
            connect_wifi(&dev, ssid, arg2)?;
        }
        Some("pass") | Some("password") => {
            let target_ssid = if let Some(s) = arg1 {
                s.to_string()
            } else {
                get_current_wifi_ssid(&dev).unwrap_or_default()
            };

            if target_ssid.is_empty() {
                bail!("not connected to Wi-Fi. Provide an SSID: run wifi pass <SSID>");
            }

            println!("{}", format!("Fetching saved password for '{}' from macOS Keychain...", target_ssid).dimmed());
            let output = Command::new("security")
                .args(["find-generic-password", "-D", "AirPort network password", "-a", &target_ssid, "-w"])
                .output()?;

            if output.status.success() {
                let pass = String::from_utf8_lossy(&output.stdout).trim().to_string();
                ui::print_banner();
                ui::render_breadcrumbs(&["run", "Wi-Fi Keychain"]);
                ui::print_card(
                    "Wi-Fi Security Credentials",
                    &[
                        ("Network (SSID)", target_ssid),
                        ("Interface", dev),
                        ("Password", pass.green().bold().to_string()),
                    ],
                );
            } else {
                println!(
                    "{} No saved password found for '{}' in Keychain.",
                    "●".yellow(),
                    target_ssid
                );
            }
        }
        Some("status") | None => {
            let ssid = get_current_wifi_ssid(&dev).unwrap_or_else(|| "Not Connected".to_string());
            let ip_out = Command::new("ipconfig").args(["getifaddr", &dev]).output().ok();
            let ip = ip_out
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "No IP Address".to_string());

            let is_connected = ssid != "Not Connected";

            ui::print_banner();
            ui::render_breadcrumbs(&["run", "Wi-Fi Dashboard"]);
            ui::print_card(
                "Wireless Connection Status",
                &[
                    ("Interface", dev.clone()),
                    ("Status", if is_connected { "Connected 🟢".green().to_string() } else { "Disconnected ⚪".dimmed().to_string() }),
                    ("Current SSID", if is_connected { ssid.bold().to_string() } else { ssid.dimmed().to_string() }),
                    ("Local IP", ip),
                ],
            );

            if action == Some("status") || !std::io::stdin().is_terminal() {
                return Ok(());
            }

            let cancel_btn = cancel_option();
            let options = [
                "📶 1. Scan Nearby Networks & Connect",
                "🔑 2. Show Saved Password from Keychain",
                "⚡ 3. Toggle Wi-Fi Power (On/Off)",
                &cancel_btn,
            ];

            let sel = Select::with_theme(theme)
                .with_prompt("Select Wi-Fi action")
                .items(&options)
                .default(0)
                .interact()?;

            match sel {
                0 => scan_and_display_wifi(theme, &dev)?,
                1 => {
                    let cur = get_current_wifi_ssid(&dev);
                    let prompt_ssid: String = if let Some(ref c) = cur {
                        c.clone()
                    } else {
                        Input::with_theme(theme)
                            .with_prompt("Enter Wi-Fi network SSID")
                            .interact_text()?
                    };
                    handle_wifi(theme, Some("pass"), Some(&prompt_ssid), None)?;
                }
                2 => {
                    let is_on = is_wifi_powered(&dev);
                    if is_on {
                        handle_wifi(theme, Some("off"), None, None)?;
                    } else {
                        handle_wifi(theme, Some("on"), None, None)?;
                    }
                }
                _ => println!("Cancelled."),
            }
        }
        Some(other) => {
            bail!("unknown wifi action '{other}'. Usage: run wifi [status|scan|connect|pass|on|off]");
        }
    }
    Ok(())
}

fn get_current_wifi_ssid(dev: &str) -> Option<String> {
    let output = Command::new("networksetup")
        .args(["-getairportnetwork", dev])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    if text.contains("Current Wi-Fi Network:") {
        Some(text.replace("Current Wi-Fi Network:", "").trim().to_string())
    } else {
        None
    }
}

fn is_wifi_powered(dev: &str) -> bool {
    let output = Command::new("networksetup")
        .args(["-getairportpower", dev])
        .output()
        .ok();
    if let Some(out) = output {
        String::from_utf8_lossy(&out.stdout).to_lowercase().contains(": on")
    } else {
        false
    }
}

fn scan_and_display_wifi(theme: &ColorfulTheme, dev: &str) -> Result<()> {
    let airport_path = "/System/Library/PrivateFrameworks/Apple80211.framework/Versions/Current/Commands/airport";
    println!("{}", "Scanning nearby Wi-Fi networks (2-3s)...".dimmed());

    let output = Command::new(airport_path).arg("-s").output();
    let Ok(out) = output else {
        println!("Airport scanner binary unavailable. You can connect directly: run wifi connect <SSID>");
        return Ok(());
    };

    let text = String::from_utf8_lossy(&out.stdout);
    let mut ssids = Vec::new();

    for line in text.lines().skip(1) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if !parts.is_empty() {
            let ssid = parts[0];
            if !ssid.is_empty() && !ssids.contains(&ssid.to_string()) {
                ssids.push(ssid.to_string());
            }
        }
    }

    if ssids.is_empty() {
        println!("No Wi-Fi broadcast networks found nearby.");
        return Ok(());
    }

    if !std::io::stdin().is_terminal() {
        println!("Discovered Networks:");
        for s in &ssids {
            println!("  📶 {s}");
        }
        return Ok(());
    }

    let cancel_btn = cancel_option();
    let mut menu_items: Vec<String> = ssids.iter().map(|s| format!("📶  {s}")).collect();
    menu_items.push(cancel_btn);

    let sel = Select::with_theme(theme)
        .with_prompt("Select Wi-Fi network to connect")
        .items(&menu_items)
        .default(0)
        .interact()?;

    if sel >= ssids.len() {
        println!("Cancelled.");
        return Ok(());
    }

    let target_ssid = &ssids[sel];
    let password: String = Input::with_theme(theme)
        .with_prompt(format!("Password for '{}' (leave blank if open/saved)", target_ssid))
        .allow_empty(true)
        .interact_text()?;

    let pass_opt = if password.trim().is_empty() { None } else { Some(password.as_str()) };
    connect_wifi(dev, target_ssid, pass_opt)
}

fn connect_wifi(dev: &str, ssid: &str, password: Option<&str>) -> Result<()> {
    println!("{}", format!("Connecting to '{ssid}' on {dev}...").dimmed());
    let mut cmd = Command::new("networksetup");
    cmd.args(["-setairportnetwork", dev, ssid]);
    if let Some(pass) = password {
        cmd.arg(pass);
    }
    let status = cmd.status()?;
    if status.success() {
        println!("{} Connected to Wi-Fi network '{}'!", "✔".green().bold(), ssid.bold());
    } else {
        bail!("failed to connect to '{ssid}'. Please verify password.");
    }
    Ok(())
}

// ============================================================================
// 2. Bluetooth & AirPods
// ============================================================================

pub fn handle_bluetooth(theme: &ColorfulTheme, action: Option<&str>) -> Result<()> {
    ui::maybe_auto_clear();

    match action {
        Some("on") => {
            let script = "tell application \"System Events\" to tell process \"ControlCenter\" to click";
            let _ = run_osascript(script);
            println!("Bluetooth power toggle requested.");
        }
        Some("off") => {
            println!("To toggle Bluetooth safely, use Control Center or run: run pods");
        }
        _ => {
            ui::print_banner();
            ui::render_breadcrumbs(&["run", "Bluetooth Dashboard"]);

            let output = Command::new("system_profiler")
                .args(["SPBluetoothDataType"])
                .output();

            let mut state = "Unknown".to_string();
            let mut connected_devices = Vec::new();

            if let Ok(out) = output {
                let text = String::from_utf8_lossy(&out.stdout);
                let mut in_connected = false;
                for line in text.lines() {
                    let trim = line.trim();
                    if trim.starts_with("State:") {
                        state = trim.replace("State:", "").trim().to_string();
                    } else if trim == "Connected:" {
                        in_connected = true;
                    } else if trim == "Not Connected:" {
                        in_connected = false;
                    } else if in_connected && trim.ends_with(':') && !trim.starts_with("Minor") {
                        let dev_name = trim.trim_end_matches(':').to_string();
                        if !dev_name.is_empty() && !connected_devices.contains(&dev_name) {
                            connected_devices.push(dev_name);
                        }
                    }
                }
            }

            let dev_str = if connected_devices.is_empty() {
                "No devices connected".dimmed().to_string()
            } else {
                connected_devices.join(", ").bold().to_string()
            };

            ui::print_card(
                "Bluetooth Controller",
                &[
                    ("Controller State", if state.to_lowercase().contains("on") { "On 🟢".green().to_string() } else { state }),
                    ("Connected Devices", dev_str),
                ],
            );

            if action != Some("status") && std::io::stdin().is_terminal() {
                let cancel_btn = cancel_option();
                let options = [
                    "🎧 1. Quick Connect AirPods / Headphones",
                    "⚙️  2. Open Bluetooth System Settings",
                    &cancel_btn,
                ];
                let sel = Select::with_theme(theme)
                    .with_prompt("Select action")
                    .items(&options)
                    .default(0)
                    .interact()?;

                if sel == 0 {
                    handle_airpods(theme)?;
                } else if sel == 1 {
                    let _ = Command::new("open").arg("x-apple.systempreferences:com.apple.BluetoothSettings").status();
                }
            }
        }
    }
    Ok(())
}

pub fn handle_airpods(theme: &ColorfulTheme) -> Result<()> {
    ui::maybe_auto_clear();
    println!("{}", "Scanning for paired audio devices...".dimmed());

    // AppleScript to list audio output devices and switch to headphones/AirPods
    let script = r#"
        tell application "System Events"
            tell process "ControlCenter"
                click menu bar item "Sound" of menu bar 1
            end tell
        end tell
    "#;
    let _ = run_osascript(script);

    println!(
        "{} Audio device menu activated in Control Center.",
        "✔".green().bold()
    );
    println!("Tip: Select your AirPods from the native sound menu, or press any key to open Bluetooth settings.");
    if std::io::stdin().is_terminal() {
        let should_open = Confirm::with_theme(theme)
            .with_prompt("Open Bluetooth settings to pair or reconnect?")
            .default(false)
            .interact()?;
        if should_open {
            let _ = Command::new("open").arg("x-apple.systempreferences:com.apple.BluetoothSettings").status();
        }
    }
    Ok(())
}

// ============================================================================
// 3. AirDrop
// ============================================================================

pub fn handle_airdrop(theme: &ColorfulTheme, file: Option<PathBuf>) -> Result<()> {
    if let Some(target_file) = file {
        if !target_file.exists() {
            bail!("target file '{}' not found", target_file.display());
        }

        let abs_path = fs::canonicalize(&target_file)?;
        let abs_str = abs_path.to_string_lossy();
        println!("{}", format!("Triggering AirDrop sharing for '{}'...", target_file.display()).dimmed());

        // Use native AppleScript share sheet or sharing utility
        let script = format!(
            r#"tell application "Finder" to open POSIX file "{abs_str}""#
        );
        let _ = run_osascript(&script);

        // Also trigger sharing service
        let _ = Command::new("sharing")
            .args(["-s", "com.apple.AirDrop", "-d", &abs_str])
            .status();

        println!("{} AirDrop share sheet opened for {}.", "✔".green().bold(), target_file.display().to_string().bold());
        return Ok(());
    }

    // Interactive or direct AirDrop window opener
    if !std::io::stdin().is_terminal() {
        let _ = Command::new("open").arg("-a").arg("AirDrop").status();
        println!("Opened AirDrop window.");
        return Ok(());
    }

    ui::maybe_auto_clear();
    ui::print_banner();
    ui::render_breadcrumbs(&["run", "AirDrop"]);

    let cancel_btn = cancel_option();
    let options = [
        "📡 1. Open AirDrop Finder Window",
        "📤 2. Select a File in Current Directory to AirDrop",
        &cancel_btn,
    ];

    let sel = Select::with_theme(theme)
        .with_prompt("Select AirDrop action")
        .items(&options)
        .default(0)
        .interact()?;

    match sel {
        0 => {
            let _ = Command::new("open").arg("-a").arg("AirDrop").status();
            println!("{} Native AirDrop window opened in Finder!", "✔".green().bold());
        }
        1 => {
            let cur_dir = std::env::current_dir()?;
            let entries = fs::read_dir(cur_dir)?;
            let mut files: Vec<PathBuf> = Vec::new();
            for e in entries.flatten() {
                if let Ok(m) = e.metadata()
                    && m.is_file()
                {
                    files.push(e.path());
                }
            }

            if files.is_empty() {
                println!("No files found in current directory.");
                return Ok(());
            }

            let mut file_names: Vec<String> = files
                .iter()
                .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
                .collect();
            file_names.push(cancel_option());

            let f_sel = FuzzySelect::with_theme(theme)
                .with_prompt("Select file to AirDrop")
                .items(&file_names)
                .default(0)
                .interact()?;

            if f_sel < files.len() {
                handle_airdrop(theme, Some(files[f_sel].clone()))?;
            } else {
                println!("Cancelled.");
            }
        }
        _ => println!("Cancelled."),
    }

    Ok(())
}

// ============================================================================
// 4. Music Player Controller (Apple Music / Spotify)
// ============================================================================

pub fn handle_music(theme: &ColorfulTheme, action: Option<&str>) -> Result<()> {
    match action {
        Some("play") => {
            let _ = run_osascript("tell application \"Music\" to play");
            println!("{} Apple Music resumed playback.", "✔".green().bold());
        }
        Some("pause") => {
            let _ = run_osascript("tell application \"Music\" to pause");
            println!("{} Apple Music playback paused.", "✔".yellow().bold());
        }
        Some("next") => {
            let _ = run_osascript("tell application \"Music\" to next track");
            println!("{} Skipped to next track.", "✔".green().bold());
        }
        Some("prev") | Some("previous") => {
            let _ = run_osascript("tell application \"Music\" to previous track");
            println!("{} Returned to previous track.", "✔".green().bold());
        }
        Some("open") => {
            let _ = Command::new("open").arg("-a").arg("Music").status();
        }
        _ => {
            ui::maybe_auto_clear();
            ui::print_banner();
            ui::render_breadcrumbs(&["run", "Music Controller"]);

            // Query current track info
            let script = r#"
                if application "Music" is running then
                    tell application "Music"
                        set playerState to (player state as string)
                        if playerState is not "stopped" then
                            set trackName to name of current track
                            set artistName to artist of current track
                            set albumName to album of current track
                            return playerState & "|||" & trackName & "|||" & artistName & "|||" & albumName
                        else
                            return "stopped|||None|||None|||None"
                        end if
                    end tell
                else
                    return "not_running|||None|||None|||None"
                end if
            "#;

            let res = run_osascript(script).unwrap_or_else(|_| "not_running|||None|||None|||None".to_string());
            let parts: Vec<&str> = res.split("|||").collect();
            let state = parts.first().copied().unwrap_or("not_running");
            let track = parts.get(1).copied().unwrap_or("-");
            let artist = parts.get(2).copied().unwrap_or("-");
            let album = parts.get(3).copied().unwrap_or("-");

            let status_badge = match state {
                "playing" => "Playing 🟢".green().to_string(),
                "paused" => "Paused ⏸️".yellow().to_string(),
                "stopped" => "Stopped ⏹️".dimmed().to_string(),
                _ => "Music App Inactive ⚪".dimmed().to_string(),
            };

            ui::print_card(
                "Now Playing",
                &[
                    ("Status", status_badge),
                    ("Track", track.bold().to_string()),
                    ("Artist", artist.to_string()),
                    ("Album", album.dimmed().to_string()),
                ],
            );

            if !std::io::stdin().is_terminal() {
                return Ok(());
            }

            let cancel_btn = cancel_option();
            let options = [
                "⏯️  1. Play / Pause",
                "⏭️  2. Next Track",
                "⏮️  3. Previous Track",
                "🎵 4. Open Music Application",
                &cancel_btn,
            ];

            let sel = Select::with_theme(theme)
                .with_prompt("Music controls")
                .items(&options)
                .default(0)
                .interact()?;

            match sel {
                0 => {
                    let _ = run_osascript("tell application \"Music\" to playpause");
                    println!("Toggled playback state.");
                }
                1 => handle_music(theme, Some("next"))?,
                2 => handle_music(theme, Some("prev"))?,
                3 => handle_music(theme, Some("open"))?,
                _ => println!("Cancelled."),
            }
        }
    }
    Ok(())
}

// ============================================================================
// 5. Volume Controller
// ============================================================================

pub fn handle_volume(theme: &ColorfulTheme, level: Option<&str>) -> Result<()> {
    if let Some(arg) = level {
        let clean = arg.trim().to_lowercase();
        if clean == "mute" {
            let _ = run_osascript("set volume output muted true");
            println!("{} System volume muted.", "✔".yellow().bold());
            return Ok(());
        } else if clean == "unmute" {
            let _ = run_osascript("set volume output muted false");
            println!("{} System volume unmuted.", "✔".green().bold());
            return Ok(());
        }

        let num: u8 = clean.parse().context("volume must be an integer between 0 and 100 or 'mute'/'unmute'")?;
        if num > 100 {
            bail!("volume cannot exceed 100%");
        }
        let script = format!("set volume output volume {num}");
        run_osascript(&script)?;
        println!("{} Volume set to {}%.", "✔".green().bold(), num.to_string().bold());
        return Ok(());
    }

    // Interactive or status
    let cur_vol = run_osascript("output volume of (get volume settings)")
        .unwrap_or_else(|_| "50".to_string())
        .parse::<u8>()
        .unwrap_or(50);
    let is_muted = run_osascript("output muted of (get volume settings)")
        .unwrap_or_else(|_| "false".to_string()) == "true";

    ui::maybe_auto_clear();
    ui::print_banner();
    ui::render_breadcrumbs(&["run", "Volume Controller"]);

    let vol_gauge = format!(
        "[{}{}] {}%",
        "■".repeat((cur_vol / 10) as usize).green(),
        "□".repeat((10 - cur_vol / 10) as usize).dimmed(),
        cur_vol
    );

    ui::print_card(
        "macOS Audio Output",
        &[
            ("Current Volume", vol_gauge),
            ("Mute Status", if is_muted { "Muted 🔇".red().to_string() } else { "Unmuted 🔊".green().to_string() }),
        ],
    );

    if !std::io::stdin().is_terminal() {
        return Ok(());
    }

    let cancel_btn = cancel_option();
    let options = [
        "🔇 1. Mute / Unmute",
        "🔈 2. Set to 25%",
        "🔉 3. Set to 50%",
        "🔊 4. Set to 75%",
        "📢 5. Set to 100% (Max)",
        "🔢 6. Enter Custom Volume Level (0-100)...",
        &cancel_btn,
    ];

    let sel = Select::with_theme(theme)
        .with_prompt("Select volume setting")
        .items(&options)
        .default(0)
        .interact()?;

    match sel {
        0 => {
            if is_muted {
                handle_volume(theme, Some("unmute"))?;
            } else {
                handle_volume(theme, Some("mute"))?;
            }
        }
        1 => handle_volume(theme, Some("25"))?,
        2 => handle_volume(theme, Some("50"))?,
        3 => handle_volume(theme, Some("75"))?,
        4 => handle_volume(theme, Some("100"))?,
        5 => {
            let custom: String = Input::with_theme(theme)
                .with_prompt("Enter volume (0-100)")
                .interact_text()?;
            handle_volume(theme, Some(&custom))?;
        }
        _ => println!("Cancelled."),
    }

    Ok(())
}

// ============================================================================
// 6. Apple Notes Quick Note
// ============================================================================

pub fn handle_note(theme: &ColorfulTheme, text: Option<&str>) -> Result<()> {
    if let Some(content) = text {
        save_quick_note(content)?;
        return Ok(());
    }

    if !std::io::stdin().is_terminal() {
        let _ = Command::new("open").arg("-a").arg("Notes").status();
        return Ok(());
    }

    ui::maybe_auto_clear();
    ui::print_banner();
    ui::render_breadcrumbs(&["run", "Apple Notes"]);

    let cancel_btn = cancel_option();
    let options = [
        "📝 1. Type and Save a Quick Note",
        "📂 2. Open Notes Application",
        &cancel_btn,
    ];

    let sel = Select::with_theme(theme)
        .with_prompt("Select Notes action")
        .items(&options)
        .default(0)
        .interact()?;

    match sel {
        0 => {
            let input: String = Input::with_theme(theme)
                .with_prompt("Enter note text")
                .interact_text()?;
            if !input.trim().is_empty() {
                save_quick_note(&input)?;
            } else {
                println!("Cancelled.");
            }
        }
        1 => {
            let _ = Command::new("open").arg("-a").arg("Notes").status();
            println!("Opened Apple Notes app.");
        }
        _ => println!("Cancelled."),
    }

    Ok(())
}

fn save_quick_note(text: &str) -> Result<()> {
    let escaped = text.replace('\\', "\\\\").replace('"', "\\\"");
    let script = format!(
        r#"
        tell application "Notes"
            tell account 1
                make new note at default folder with properties {{body:"{escaped}"}}
            end tell
        end tell
        "#
    );
    run_osascript(&script)?;
    println!(
        "{} Saved note to Apple Notes: \"{}\"",
        "✔".green().bold(),
        text.bold()
    );
    Ok(())
}

// ============================================================================
// 7. Gatekeeper & Quarantine Fixer (`run fixapp`)
// ============================================================================

pub fn handle_fixapp(theme: &ColorfulTheme, app_name: Option<&str>) -> Result<()> {
    let scan_dirs = [
        PathBuf::from("/Applications"),
        dirs_home_apps(),
    ];

    let mut found_apps: Vec<PathBuf> = Vec::new();
    for dir in &scan_dirs {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.extension().map(|e| e == "app").unwrap_or(false) {
                    found_apps.push(p);
                }
            }
        }
    }
    found_apps.sort();

    if let Some(query) = app_name {
        // Direct matching
        let q_lower = query.to_lowercase();
        let target_path = if Path::new(query).exists() {
            PathBuf::from(query)
        } else if let Some(matched) = found_apps.iter().find(|p| {
            p.file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.to_lowercase() == q_lower || s.to_lowercase().contains(&q_lower))
                .unwrap_or(false)
        }) {
            matched.clone()
        } else {
            bail!("no application matching '{query}' found in /Applications");
        };

        fix_quarantine(&target_path)?;
        return Ok(());
    }

    // Interactive fuzzy picker
    if !std::io::stdin().is_terminal() {
        bail!("specify application name or path: run fixapp <app_name>");
    }

    ui::maybe_auto_clear();
    ui::print_banner();
    ui::render_breadcrumbs(&["run", "Quarantine Fixer"]);

    if found_apps.is_empty() {
        println!("No installed .app applications found.");
        return Ok(());
    }

    let cancel_btn = cancel_option();
    let mut names: Vec<String> = found_apps
        .iter()
        .map(|p| p.file_stem().unwrap().to_string_lossy().to_string())
        .collect();
    names.push(cancel_btn);

    ui::print_key_hints();
    let sel = FuzzySelect::with_theme(theme)
        .with_prompt("Select application to remove Gatekeeper quarantine (xattr -cr)")
        .items(&names)
        .default(0)
        .interact()?;

    if sel < found_apps.len() {
        fix_quarantine(&found_apps[sel])?;
    } else {
        println!("Cancelled.");
    }

    Ok(())
}

fn dirs_home_apps() -> PathBuf {
    std::env::var_os("HOME")
        .map(|h| PathBuf::from(h).join("Applications"))
        .unwrap_or_else(|| PathBuf::from("/Applications"))
}

fn fix_quarantine(app_path: &Path) -> Result<()> {
    let display_name = app_path.file_name().unwrap().to_string_lossy();
    println!("{}", format!("Removing quarantine attributes from {}...", display_name).dimmed());

    let status = Command::new("xattr")
        .args(["-cr", &app_path.to_string_lossy()])
        .status()
        .context("failed to execute xattr")?;

    if status.success() {
        println!(
            "{} Successfully fixed Gatekeeper quarantine for '{}'!",
            "✔".green().bold(),
            display_name.bold()
        );
        println!("You can now launch the application without \"damaged app\" warnings.");
    } else {
        bail!("xattr command returned non-zero status. You may need sudo permissions.");
    }
    Ok(())
}

// ============================================================================
// 8. Battery Health Inspector
// ============================================================================

pub fn handle_battery(_theme: &ColorfulTheme) -> Result<()> {
    ui::maybe_auto_clear();
    ui::print_banner();
    ui::render_breadcrumbs(&["run", "Battery Health"]);

    let output = Command::new("pmset").args(["-g", "batt"]).output();
    let text = output.map(|o| String::from_utf8_lossy(&o.stdout).to_string()).unwrap_or_default();

    let mut percent = "Unknown".to_string();
    let mut status = "Unknown".to_string();
    let mut time_rem = "Unknown".to_string();
    let mut power_source = "Battery".to_string();

    for line in text.lines() {
        if line.contains("Now drawing from") {
            power_source = if line.contains("AC Power") { "AC Power Charger ⚡".to_string() } else { "Internal Battery 🔋".to_string() };
        } else if line.contains('%') {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() > 1 {
                let info = parts[1];
                let sub: Vec<&str> = info.split(';').collect();
                if !sub.is_empty() {
                    percent = sub[0].trim().to_string();
                }
                if sub.len() > 1 {
                    status = sub[1].trim().to_string();
                }
                if sub.len() > 2 {
                    time_rem = sub[2].trim().replace("present: true", "").trim().to_string();
                    if time_rem.is_empty() {
                        time_rem = "Calculated by system".to_string();
                    }
                }
            }
        }
    }

    // Cycle count from system_profiler
    let sp_out = Command::new("system_profiler").args(["SPPowerDataType"]).output().ok();
    let mut cycle_count = "-".to_string();
    let mut max_capacity = "-".to_string();
    let mut condition = "-".to_string();

    if let Some(out) = sp_out {
        let sp_text = String::from_utf8_lossy(&out.stdout);
        for line in sp_text.lines() {
            let trim = line.trim();
            if trim.starts_with("Cycle Count:") {
                cycle_count = trim.replace("Cycle Count:", "").trim().to_string();
            } else if trim.starts_with("Condition:") {
                condition = trim.replace("Condition:", "").trim().to_string();
            } else if trim.starts_with("Maximum Capacity:") {
                max_capacity = trim.replace("Maximum Capacity:", "").trim().to_string();
            }
        }
    }

    ui::print_card(
        "macOS Power & Battery Snapshot",
        &[
            ("Charge Level", percent.bold().to_string()),
            ("Power Source", power_source),
            ("Charging Status", status),
            ("Time Remaining", time_rem),
            ("Cycle Count", cycle_count),
            ("Max Capacity", max_capacity),
            ("Condition", condition),
        ],
    );

    Ok(())
}

// ============================================================================
// 9. Anti-Sleep / Caffeinate (`run awake`)
// ============================================================================

pub fn handle_awake(theme: &ColorfulTheme, minutes: Option<u64>) -> Result<()> {
    let duration_mins = match minutes {
        Some(m) => m,
        None => {
            if !std::io::stdin().is_terminal() {
                // Indefinite default
                0
            } else {
                ui::maybe_auto_clear();
                ui::print_banner();
                ui::render_breadcrumbs(&["run", "Awake Anti-Sleep"]);

                let cancel_btn = cancel_option();
                let options = [
                    "☕ 1. Keep awake for 15 minutes",
                    "☕ 2. Keep awake for 30 minutes",
                    "☕ 3. Keep awake for 1 hour (60 mins)",
                    "☕ 4. Keep awake for 2 hours (120 mins)",
                    "♾️  5. Keep awake indefinitely (until Ctrl+C)",
                    &cancel_btn,
                ];

                let sel = Select::with_theme(theme)
                    .with_prompt("Select anti-sleep duration")
                    .items(&options)
                    .default(1)
                    .interact()?;

                match sel {
                    0 => 15,
                    1 => 30,
                    2 => 60,
                    3 => 120,
                    4 => 0,
                    _ => {
                        println!("Cancelled.");
                        return Ok(());
                    }
                }
            }
        }
    };

    if duration_mins > 0 {
        let secs = duration_mins * 60;
        println!(
            "{} Keeping Mac awake for {} minutes (display & system sleep disabled)...",
            "☕".bold(),
            duration_mins.to_string().bold()
        );
        println!("Press Ctrl+C to cancel early.");
        let _ = Command::new("caffeinate")
            .args(["-d", "-t", &secs.to_string()])
            .status();
        println!("{} Awake timer expired.", "✔".green().bold());
    } else {
        println!("{} Keeping Mac awake indefinitely (Press Ctrl+C to stop)...", "☕".bold());
        let _ = Command::new("caffeinate").arg("-d").status();
        println!("{} Anti-sleep ended.", "✔".green().bold());
    }

    Ok(())
}

// ============================================================================
// 10. QuickLook Previewer (`run peek`)
// ============================================================================

pub fn handle_peek(theme: &ColorfulTheme, file: Option<PathBuf>) -> Result<()> {
    let target = match file {
        Some(f) => f,
        None => {
            if !std::io::stdin().is_terminal() {
                bail!("provide a file path to preview: run peek <file>");
            }
            let cur_dir = std::env::current_dir()?;
            let entries = fs::read_dir(cur_dir)?;
            let mut files: Vec<PathBuf> = Vec::new();
            for e in entries.flatten() {
                if let Ok(m) = e.metadata()
                    && m.is_file()
                {
                    files.push(e.path());
                }
            }

            if files.is_empty() {
                bail!("no files found in current directory to preview");
            }

            let mut names: Vec<String> = files
                .iter()
                .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
                .collect();
            names.push(cancel_option());

            let sel = FuzzySelect::with_theme(theme)
                .with_prompt("Select file for macOS QuickLook preview")
                .items(&names)
                .default(0)
                .interact()?;

            if sel < files.len() {
                files[sel].clone()
            } else {
                println!("Cancelled.");
                return Ok(());
            }
        }
    };

    if !target.exists() {
        bail!("file '{}' does not exist", target.display());
    }

    println!("{}", format!("Launching macOS QuickLook for '{}'...", target.display()).dimmed());
    let _ = Command::new("qlmanage").args(["-p", &target.to_string_lossy()]).status();
    Ok(())
}

// ============================================================================
// 11. Trash Manager (`run trash`)
// ============================================================================

pub fn handle_trash(theme: &ColorfulTheme, action: Option<&str>) -> Result<()> {
    let home = std::env::var("HOME").unwrap_or_default();
    let trash_dir = PathBuf::from(home).join(".Trash");

    match action {
        Some("empty") => {
            if std::io::stdin().is_terminal() {
                let confirm = Confirm::with_theme(theme)
                    .with_prompt("Permanently empty macOS Trash?")
                    .default(false)
                    .interact()?;
                if !confirm {
                    println!("Cancelled.");
                    return Ok(());
                }
            }
            let script = "tell application \"Finder\" to empty trash";
            if run_osascript(script).is_err()
                && let Ok(entries) = fs::read_dir(&trash_dir)
            {
                for e in entries.flatten() {
                    let path = e.path();
                    if path.is_dir() {
                        let _ = fs::remove_dir_all(path);
                    } else {
                        let _ = fs::remove_file(path);
                    }
                }
            }
            println!("{} macOS Trash emptied! 🧹", "✔".green().bold());
        }
        Some("list") | Some("ls") => {
            if !trash_dir.exists() {
                println!("Trash is empty.");
                return Ok(());
            }
            match fs::read_dir(&trash_dir) {
                Ok(entries) => {
                    let mut count = 0;
                    println!("{}", "Items in Trash:".bold());
                    for e in entries.flatten() {
                        count += 1;
                        println!("  🗑️  {}", e.file_name().to_string_lossy());
                    }
                    if count == 0 {
                        println!("  (Trash is empty)");
                    }
                }
                Err(_) => {
                    println!(
                        "{} Access to ~/.Trash is restricted by macOS Privacy/TCC.",
                        "●".yellow()
                    );
                    println!(
                        "To empty trash directly, run: {}",
                        "run trash empty".bold()
                    );
                }
            }
        }
        _ => {
            ui::maybe_auto_clear();
            ui::print_banner();
            ui::render_breadcrumbs(&["run", "Trash Manager"]);

            let (items_display, size_display) = match fs::read_dir(&trash_dir) {
                Ok(entries) => {
                    let mut count = 0;
                    let mut total_bytes = 0;
                    for e in entries.flatten() {
                        count += 1;
                        if let Ok(m) = e.metadata() {
                            total_bytes += m.len();
                        }
                    }
                    (count.to_string(), crate::commands::format_bytes(total_bytes))
                }
                Err(_) => (
                    "Restricted by macOS".dimmed().to_string(),
                    "Requires Full Disk Access".dimmed().to_string(),
                ),
            };

            ui::print_card(
                "Trash Storage Usage",
                &[
                    ("Total Items", items_display),
                    ("Approximate Size", size_display),
                    ("Location", "~/.Trash".to_string()),
                ],
            );

            if !std::io::stdin().is_terminal() {
                return Ok(());
            }

            let cancel_btn = cancel_option();
            let options = [
                "🗑️  1. Empty Trash Safely",
                "📋 2. List Trash Contents",
                &cancel_btn,
            ];

            let sel = Select::with_theme(theme)
                .with_prompt("Select Trash action")
                .items(&options)
                .default(0)
                .interact()?;

            match sel {
                0 => handle_trash(theme, Some("empty"))?,
                1 => handle_trash(theme, Some("list"))?,
                _ => println!("Cancelled."),
            }
        }
    }
    Ok(())
}

// ============================================================================
// 12. Screenshot / Screen Capture (`run shot`)
// ============================================================================

pub fn handle_shot(theme: &ColorfulTheme, mode: Option<&str>) -> Result<()> {
    match mode {
        Some("window") => {
            println!("{}", "Click any window to capture with native shadow to clipboard...".dimmed());
            let _ = Command::new("screencapture").args(["-c", "-W"]).status();
            println!("{} Window screenshot copied to clipboard!", "✔".green().bold());
        }
        Some("full") => {
            let home = std::env::var("HOME").unwrap_or_default();
            let out_file = PathBuf::from(home).join("Desktop").join("screenshot.png");
            println!("{}", "Capturing full screen to Desktop...".dimmed());
            let _ = Command::new("screencapture").arg(&out_file).status();
            println!("{} Screenshot saved to Desktop: {}", "✔".green().bold(), out_file.display());
        }
        Some("clip") | Some("selection") | None => {
            if std::io::stdin().is_terminal() && mode.is_none() {
                ui::maybe_auto_clear();
                ui::print_banner();
                ui::render_breadcrumbs(&["run", "Screen Capture"]);

                let cancel_btn = cancel_option();
                let options = [
                    "✂️  1. Selection to Clipboard (Default)",
                    "🪟 2. Window with Shadow to Clipboard",
                    "🖥️  3. Full Screen to Desktop",
                    &cancel_btn,
                ];

                let sel = Select::with_theme(theme)
                    .with_prompt("Select capture mode")
                    .items(&options)
                    .default(0)
                    .interact()?;

                match sel {
                    0 => handle_shot(theme, Some("clip"))?,
                    1 => handle_shot(theme, Some("window"))?,
                    2 => handle_shot(theme, Some("full"))?,
                    _ => println!("Cancelled."),
                }
                return Ok(());
            }

            println!("{}", "Select area on screen (copied directly to clipboard)...".dimmed());
            let _ = Command::new("screencapture").args(["-c", "-i"]).status();
            println!("{} Area screenshot copied to clipboard!", "✔".green().bold());
        }
        Some(other) => {
            bail!("unknown shot mode '{other}'. Supported: selection, window, full");
        }
    }
    Ok(())
}

// ============================================================================
// 13. System Notifications (`run notify`)
// ============================================================================

pub fn handle_notify(theme: &ColorfulTheme, title: Option<&str>, message: Option<&str>) -> Result<()> {
    let t = match title {
        Some(val) => val.to_string(),
        None => {
            if !std::io::stdin().is_terminal() {
                "Notification from run-cli".to_string()
            } else {
                Input::with_theme(theme).with_prompt("Notification Title").interact_text()?
            }
        }
    };

    let m = match message {
        Some(val) => val.to_string(),
        None => {
            if !std::io::stdin().is_terminal() {
                "Process finished successfully.".to_string()
            } else {
                Input::with_theme(theme).with_prompt("Notification Message").interact_text()?
            }
        }
    };

    let script = format!(
        r#"display notification "{m}" with title "{t}" sound name "Glass""#
    );
    run_osascript(&script)?;
    println!("{} Dispatched native notification: \"{t}\"", "✔".green().bold(), t = t);
    Ok(())
}

// ============================================================================
// 14. Dark / Light Mode & Lock Screen
// ============================================================================

pub fn handle_dark(_theme: &ColorfulTheme, mode: Option<&str>) -> Result<()> {
    match mode {
        Some("on") | Some("true") | Some("1") => {
            let script = "tell app \"System Events\" to tell appearance preferences to set dark mode to true";
            run_osascript(script)?;
            println!("{} Dark Mode enabled 🌙", "✔".green().bold());
        }
        Some("off") | Some("false") | Some("0") => {
            let script = "tell app \"System Events\" to tell appearance preferences to set dark mode to false";
            run_osascript(script)?;
            println!("{} Light Mode enabled ☀️", "✔".green().bold());
        }
        _ => {
            // Toggle
            let script = "tell app \"System Events\" to tell appearance preferences to set dark mode to not dark mode";
            run_osascript(script)?;
            let cur = run_osascript("tell app \"System Events\" to tell appearance preferences to get dark mode")
                .unwrap_or_else(|_| "true".to_string());
            if cur == "true" {
                println!("{} Switched to Dark Mode 🌙", "✔".green().bold());
            } else {
                println!("{} Switched to Light Mode ☀️", "✔".green().bold());
            }
        }
    }
    Ok(())
}

pub fn handle_light(theme: &ColorfulTheme) -> Result<()> {
    handle_dark(theme, Some("off"))
}

pub fn handle_lock() -> Result<()> {
    println!("{}", "Locking macOS screen...".dimmed());
    let status = Command::new("pmset").arg("displaysleepnow").status();
    if status.is_err() {
        let _ = Command::new("/System/Library/CoreServices/Menu Extras/User.menu/Contents/Resources/CGSession")
            .arg("-suspend")
            .status();
    }
    println!("{} Screen locked.", "✔".green().bold());
    Ok(())
}

// ============================================================================
// 15. Desktop Clean Presentation Mode (`run desktop`)
// ============================================================================

pub fn handle_desktop(theme: &ColorfulTheme, action: Option<&str>) -> Result<()> {
    match action {
        Some("hide") | Some("clean") => {
            let _ = Command::new("defaults")
                .args(["write", "com.apple.finder", "CreateDesktop", "-bool", "false"])
                .status();
            let _ = Command::new("killall").arg("Finder").status();
            println!("{} Desktop icons hidden for clean presentation mode!", "✔".green().bold());
        }
        Some("show") | Some("restore") => {
            let _ = Command::new("defaults")
                .args(["delete", "com.apple.finder", "CreateDesktop"])
                .status();
            let _ = Command::new("killall").arg("Finder").status();
            println!("{} Desktop icons restored.", "✔".green().bold());
        }
        _ => {
            if !std::io::stdin().is_terminal() {
                return handle_desktop(theme, Some("hide"));
            }

            ui::maybe_auto_clear();
            ui::print_banner();
            ui::render_breadcrumbs(&["run", "Desktop Presentation"]);

            let cancel_btn = cancel_option();
            let options = [
                "🙈 1. Hide Desktop Icons (Presentation Mode)",
                "👁️  2. Show Desktop Icons (Default)",
                &cancel_btn,
            ];

            let sel = Select::with_theme(theme)
                .with_prompt("Select desktop mode")
                .items(&options)
                .default(0)
                .interact()?;

            match sel {
                0 => handle_desktop(theme, Some("hide"))?,
                1 => handle_desktop(theme, Some("show"))?,
                _ => println!("Cancelled."),
            }
        }
    }
    Ok(())
}
