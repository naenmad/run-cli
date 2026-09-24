use std::fs;
use std::io::{IsTerminal, Write};
use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, Result, bail};
use colored::Colorize;
use dialoguer::{Confirm, FuzzySelect, Input, Select};
use serde::{Deserialize, Serialize};

use crate::config;
use crate::ui::{self, RunTheme as ColorfulTheme};

fn electric_blue(text: &str) -> colored::ColoredString {
    ui::electric_blue(text)
}

/// Helper to format byte sizes into readable units
pub fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

pub fn cancel_option() -> String {
    format!("{}", "Cancel".bright_red().bold())
}

/// path (alias: pth, pwd) - Print current working directory
pub fn handle_path(theme: &ColorfulTheme, interactive: bool) -> Result<()> {
    let current_dir = std::env::current_dir().context("failed to get current working directory")?;
    let path_str = current_dir.to_string_lossy().to_string();

    if interactive {
        let cancel_btn = cancel_option();
        let options = ["Print path", "Copy path to clipboard (pbcopy)", &cancel_btn];
        let selection = Select::with_theme(theme)
            .with_prompt(format!("Current Directory: {path_str}"))
            .items(&options)
            .default(0)
            .interact()?;

        match selection {
            0 => println!("{path_str}"),
            1 => {
                let mut child = Command::new("pbcopy")
                    .stdin(std::process::Stdio::piped())
                    .spawn()
                    .context("failed to run pbcopy")?;
                if let Some(mut stdin) = child.stdin.take() {
                    stdin.write_all(path_str.as_bytes())?;
                }
                child.wait()?;
                println!("{}", "Copied directory path to clipboard.".green());
            }
            _ => println!("Cancelled."),
        }
    } else {
        println!("{path_str}");
    }
    Ok(())
}

/// list (alias: lst, ls) - List directory contents
pub fn handle_list(
    _theme: &ColorfulTheme,
    path: Option<PathBuf>,
    all: bool,
    long: bool,
) -> Result<()> {
    let target_dir = match path {
        Some(p) => p,
        None => std::env::current_dir().context("failed to get current directory")?,
    };

    if !target_dir.exists() {
        bail!("directory does not exist: {}", target_dir.display());
    }

    if long {
        let mut cmd = Command::new("ls");
        cmd.arg("-l");
        if all {
            cmd.arg("-a");
        }
        cmd.arg(&target_dir);
        let status = cmd.status().context("failed to execute 'ls'")?;
        if !status.success() {
            bail!("ls command failed");
        }
        return Ok(());
    }

    let entries = fs::read_dir(&target_dir)
        .with_context(|| format!("failed to read directory '{}'", target_dir.display()))?;

    let mut items: Vec<(String, bool, u64)> = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !all && name.starts_with('.') {
            continue;
        }
        let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
        let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
        items.push((name, is_dir, size));
    }

    items.sort_by(|a, b| {
        if a.1 != b.1 {
            b.1.cmp(&a.1)
        } else {
            a.0.to_lowercase().cmp(&b.0.to_lowercase())
        }
    });

    if items.is_empty() {
        println!("{}", "(empty directory)".dimmed());
        return Ok(());
    }

    for (name, is_dir, size) in items {
        if is_dir {
            println!("  {}/", electric_blue(&name).bold());
        } else {
            let size_str = format_bytes(size);
            println!("  {: <32} {}", name, size_str.dimmed());
        }
    }

    Ok(())
}

/// read (alias: red, cat) - Display file contents
pub fn handle_read(theme: &ColorfulTheme, path: Option<PathBuf>) -> Result<()> {
    let target = match path {
        Some(p) => p,
        None => {
            let current_dir = std::env::current_dir()?;
            let mut files = Vec::new();
            if let Ok(entries) = fs::read_dir(&current_dir) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.is_file()
                        && let Some(name) = p.file_name().and_then(|n| n.to_str())
                        && !name.starts_with('.')
                    {
                        files.push(p);
                    }
                }
            }

            if files.is_empty() {
                bail!("no files found in current directory to read");
            }

            let mut options: Vec<String> = files
                .iter()
                .map(|f| f.file_name().unwrap().to_string_lossy().to_string())
                .collect();
            options.push(cancel_option());

            let selection = Select::with_theme(theme)
                .with_prompt("Select file to read")
                .items(&options)
                .default(0)
                .interact()?;

            if selection == options.len() - 1 {
                println!("Cancelled.");
                return Ok(());
            }

            files[selection].clone()
        }
    };

    if !target.exists() {
        bail!("file does not exist: {}", target.display());
    }

    let content = fs::read_to_string(&target).with_context(|| {
        format!(
            "failed to read file '{}' (binary files cannot be read as text)",
            target.display()
        )
    })?;

    print!("{content}");
    if !content.ends_with('\n') {
        println!();
    }
    Ok(())
}

/// find (alias: fnd, grep) - Search pattern in file or directory
pub fn handle_find(
    theme: &ColorfulTheme,
    pattern: Option<String>,
    path: Option<PathBuf>,
) -> Result<()> {
    let resolved_pattern = match pattern {
        Some(p) => p,
        None => {
            let input: String = Input::with_theme(theme)
                .with_prompt("Search pattern (leave blank to cancel)")
                .allow_empty(true)
                .interact_text()?;
            let trimmed = input.trim().to_string();
            if trimmed.is_empty() {
                println!("Cancelled.");
                return Ok(());
            }
            trimmed
        }
    };

    let resolved_path = path.unwrap_or_else(|| PathBuf::from("."));

    let mut cmd = Command::new("grep");
    cmd.arg("-rnI");
    cmd.arg(&resolved_pattern);
    cmd.arg(&resolved_path);

    let status = cmd.status().context("failed to execute grep")?;
    if !status.success() && status.code() != Some(1) {
        bail!("search returned an error");
    }

    Ok(())
}

/// process (alias: prc, ps, top) - List active processes or system resource snapshot
pub fn handle_process(filter: Option<&str>, snapshot: bool) -> Result<()> {
    if snapshot {
        println!("{}", electric_blue("SYSTEM RESOURCE SNAPSHOT:").bold());
        let mut cmd = Command::new("top");
        cmd.args(["-l", "1", "-s", "0", "-n", "12"]);
        let status = cmd.status().context("failed to execute 'top'")?;
        if !status.success() {
            bail!("top command failed");
        }
        return Ok(());
    }

    let output = Command::new("ps")
        .args(["-eo", "pid,user,%cpu,%mem,comm"])
        .output()
        .context("failed to run 'ps'")?;

    let text = String::from_utf8_lossy(&output.stdout);
    let mut lines = text.lines();

    if let Some(header) = lines.next() {
        println!("{}", electric_blue(header).bold());
    }

    for line in lines {
        if let Some(f) = filter {
            if line.to_lowercase().contains(&f.to_lowercase()) {
                println!("{line}");
            }
        } else {
            println!("{line}");
        }
    }

    Ok(())
}

/// kill (alias: kil, stop, stp) - Terminate a process by PID or name
pub fn handle_kill(theme: &ColorfulTheme, target: Option<&str>, force: bool) -> Result<()> {
    let pid = match target {
        Some(t) => {
            if let Ok(p) = t.parse::<u32>() {
                p.to_string()
            } else {
                let output = Command::new("pgrep")
                    .arg(t)
                    .output()
                    .context("failed to search process with pgrep")?;
                let text = String::from_utf8_lossy(&output.stdout);
                let first_pid = text.lines().next().unwrap_or("").trim().to_string();
                if first_pid.is_empty() {
                    bail!("no running process found matching '{t}'");
                }
                first_pid
            }
        }
        None => {
            let output = Command::new("ps")
                .args(["-eo", "pid,comm"])
                .output()
                .context("failed to list processes")?;

            let text = String::from_utf8_lossy(&output.stdout);
            let mut proc_list = Vec::new();

            for line in text.lines().skip(1).take(20) {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    proc_list.push(trimmed.to_string());
                }
            }

            if proc_list.is_empty() {
                bail!("no processes found");
            }

            proc_list.push(cancel_option());

            let selection = Select::with_theme(theme)
                .with_prompt("Select process to terminate")
                .items(&proc_list)
                .default(0)
                .interact()?;

            if selection == proc_list.len() - 1 {
                println!("Cancelled.");
                return Ok(());
            }

            let chosen = &proc_list[selection];
            let pid_part = chosen.split_whitespace().next().unwrap_or("").to_string();
            if pid_part.is_empty() {
                bail!("invalid process selected");
            }
            pid_part
        }
    };

    let prompt = format!("Kill process {pid}?");
    let confirmed = Confirm::with_theme(theme)
        .with_prompt(prompt)
        .default(false)
        .interact()?;

    if !confirmed {
        println!("Cancelled.");
        return Ok(());
    }

    let mut cmd = Command::new("kill");
    if force {
        cmd.arg("-9");
    }
    cmd.arg(&pid);

    let status = cmd.status().context("failed to execute kill command")?;
    if !status.success() {
        bail!("failed to kill process {pid}");
    }

    println!("{}", format!("Terminated process {pid}.").green());
    Ok(())
}



/// fetch (alias: fch, get, curl, wget) - Fetch HTTP response or download file locally
pub fn handle_fetch(
    theme: &ColorfulTheme,
    url: Option<&str>,
    output: Option<PathBuf>,
    headers_only: bool,
) -> Result<()> {
    let target_url = match url {
        Some(u) => u.to_string(),
        None => {
            let input: String = Input::with_theme(theme)
                .with_prompt("Target URL (leave blank to cancel)")
                .allow_empty(true)
                .interact_text()?;
            let trimmed = input.trim().to_string();
            if trimmed.is_empty() {
                println!("Cancelled.");
                return Ok(());
            }
            trimmed
        }
    };

    let mut cmd = Command::new("curl");

    if let Some(out) = output {
        cmd.args([
            "-L",
            "--progress-bar",
            "-o",
            &out.to_string_lossy(),
            &target_url,
        ]);
        println!(
            "{}",
            format!("Downloading '{target_url}' to '{}'...", out.display()).dimmed()
        );
        let status = cmd.status().context("failed to download file with curl")?;
        if !status.success() {
            bail!("download failed");
        }
        println!("{}", "Download completed successfully!".green().bold());
    } else {
        if headers_only {
            cmd.arg("-I");
        } else {
            cmd.arg("-i");
        }
        cmd.arg(&target_url);

        let status = cmd.status().context("failed to execute curl")?;
        if !status.success() {
            bail!("fetch request failed");
        }
    }

    Ok(())
}

/// disk (alias: dsk, df, du) - Inspect disk free space or directory usage
pub fn handle_disk(theme: &ColorfulTheme, path: Option<PathBuf>, usage: bool) -> Result<()> {
    if usage || path.is_some() {
        let target = match path {
            Some(p) => p,
            None => {
                let current_dir = std::env::current_dir()?;
                let mut dirs = Vec::new();
                if let Ok(entries) = fs::read_dir(&current_dir) {
                    for entry in entries.flatten() {
                        let p = entry.path();
                        if p.is_dir()
                            && let Some(name) = p.file_name().and_then(|n| n.to_str())
                            && !name.starts_with('.')
                        {
                            dirs.push((name.to_string(), p));
                        }
                    }
                }

                if dirs.is_empty() {
                    PathBuf::from(".")
                } else {
                    let mut options: Vec<String> =
                        dirs.iter().map(|(n, _)| format!("{n}/")).collect();
                    options.insert(0, ". (Current directory)".to_string());
                    options.push(cancel_option());

                    let selection = Select::with_theme(theme)
                        .with_prompt("Select target folder to inspect usage")
                        .items(&options)
                        .default(0)
                        .interact()?;

                    if selection == options.len() - 1 {
                        println!("Cancelled.");
                        return Ok(());
                    }

                    if selection == 0 {
                        PathBuf::from(".")
                    } else {
                        dirs[selection - 1].1.clone()
                    }
                }
            }
        };

        println!(
            "{}",
            format!("Calculating disk usage for '{}'...", target.display()).dimmed()
        );
        let mut cmd = Command::new("du");
        cmd.args(["-sh", &target.to_string_lossy()]);
        let status = cmd.status().context("failed to execute 'du -sh'")?;
        if !status.success() {
            bail!("du command failed");
        }
    } else {
        let mut cmd = Command::new("df");
        cmd.arg("-h");
        let status = cmd.status().context("failed to execute 'df -h'")?;
        if !status.success() {
            bail!("df command failed");
        }
    }
    Ok(())
}

/// pack (alias: pck, tar, zip) - Create archive (.tar.gz or .zip)
pub fn handle_pack(
    theme: &ColorfulTheme,
    archive: Option<PathBuf>,
    target: Option<PathBuf>,
) -> Result<()> {
    let arc = match archive {
        Some(p) => p,
        None => {
            let input: String = Input::with_theme(theme)
                .with_prompt("Output archive name (e.g. archive.tar.gz or archive.zip)")
                .allow_empty(true)
                .interact_text()?;
            let trimmed = input.trim();
            if trimmed.is_empty() {
                println!("Cancelled.");
                return Ok(());
            }
            PathBuf::from(trimmed)
        }
    };

    let tgt = match target {
        Some(p) => p,
        None => {
            let input: String = Input::with_theme(theme)
                .with_prompt("Target folder or file to archive (leave blank to cancel)")
                .allow_empty(true)
                .interact_text()?;
            let trimmed = input.trim();
            if trimmed.is_empty() {
                println!("Cancelled.");
                return Ok(());
            }
            PathBuf::from(trimmed)
        }
    };

    let arc_str = arc.to_string_lossy();
    if arc_str.ends_with(".zip") {
        let status = Command::new("zip")
            .args(["-r", &arc_str, &tgt.to_string_lossy()])
            .status()
            .context("failed to create zip archive")?;
        if !status.success() {
            bail!("zip creation failed");
        }
    } else {
        let status = Command::new("tar")
            .args(["-czvf", &arc_str, &tgt.to_string_lossy()])
            .status()
            .context("failed to create tar archive")?;
        if !status.success() {
            bail!("tar creation failed");
        }
    }

    println!("{}", format!("Created archive: {}", arc.display()).green());
    Ok(())
}

/// unpack (alias: upk, unzip, untar) - Extract archive (.zip or .tar.gz)
pub fn handle_unpack(
    theme: &ColorfulTheme,
    archive: Option<PathBuf>,
    destination: Option<PathBuf>,
) -> Result<()> {
    let arc = match archive {
        Some(p) => p,
        None => {
            let current_dir = std::env::current_dir()?;
            let mut archives = Vec::new();
            if let Ok(entries) = fs::read_dir(&current_dir) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    let name = p.to_string_lossy();
                    if name.ends_with(".zip")
                        || name.ends_with(".tar.gz")
                        || name.ends_with(".tgz")
                        || name.ends_with(".tar")
                    {
                        archives.push(p);
                    }
                }
            }

            if archives.is_empty() {
                let input: String = Input::with_theme(theme)
                    .with_prompt("Path to archive file (leave blank to cancel)")
                    .allow_empty(true)
                    .interact_text()?;
                let trimmed = input.trim();
                if trimmed.is_empty() {
                    println!("Cancelled.");
                    return Ok(());
                }
                PathBuf::from(trimmed)
            } else {
                let mut options: Vec<String> = archives
                    .iter()
                    .map(|f| f.file_name().unwrap().to_string_lossy().to_string())
                    .collect();
                options.push(cancel_option());

                let selection = Select::with_theme(theme)
                    .with_prompt("Select archive to extract")
                    .items(&options)
                    .default(0)
                    .interact()?;

                if selection == options.len() - 1 {
                    println!("Cancelled.");
                    return Ok(());
                }
                archives[selection].clone()
            }
        }
    };

    if !arc.exists() {
        bail!("archive does not exist: {}", arc.display());
    }

    let arc_str = arc.to_string_lossy();
    if arc_str.ends_with(".zip") {
        let mut cmd = Command::new("unzip");
        cmd.arg(&arc);
        if let Some(dst) = destination {
            cmd.args(["-d", &dst.to_string_lossy()]);
        }
        let status = cmd.status().context("failed to execute 'unzip'")?;
        if !status.success() {
            bail!("unzip failed");
        }
    } else {
        let mut cmd = Command::new("tar");
        cmd.args(["-xvf", &arc_str]);
        if let Some(dst) = destination {
            cmd.args(["-C", &dst.to_string_lossy()]);
        }
        let status = cmd.status().context("failed to execute 'tar -xvf'")?;
        if !status.success() {
            bail!("tar extract failed");
        }
    }

    println!(
        "{}",
        format!("Extracted archive: {}", arc.display()).green()
    );
    Ok(())
}

/// permit (alias: prm, chmod) - Change permissions with presets
pub fn handle_permit(
    theme: &ColorfulTheme,
    mode: Option<&str>,
    path: Option<PathBuf>,
) -> Result<()> {
    let target = match path {
        Some(p) => p,
        None => {
            let input: String = Input::with_theme(theme)
                .with_prompt("Target file/directory (leave blank to cancel)")
                .allow_empty(true)
                .interact_text()?;
            let trimmed = input.trim();
            if trimmed.is_empty() {
                println!("Cancelled.");
                return Ok(());
            }
            PathBuf::from(trimmed)
        }
    };

    if !target.exists() {
        bail!("path does not exist: {}", target.display());
    }

    let resolved_mode = match mode {
        Some(m) => m.to_string(),
        None => {
            let cancel_btn = cancel_option();
            let presets = [
                "755 (rwxr-xr-x) - Executable script / application",
                "644 (rw-r--r--) - Standard readable file",
                "600 (rw-------) - Private / Secret credential",
                "+x  (Make executable)",
                &cancel_btn,
            ];

            let selection = Select::with_theme(theme)
                .with_prompt("Select permission preset")
                .items(&presets)
                .default(0)
                .interact()?;

            match selection {
                0 => "755".to_string(),
                1 => "644".to_string(),
                2 => "600".to_string(),
                3 => "+x".to_string(),
                _ => {
                    println!("Cancelled.");
                    return Ok(());
                }
            }
        }
    };

    let status = Command::new("chmod")
        .arg(&resolved_mode)
        .arg(&target)
        .status()
        .context("failed to execute chmod")?;

    if !status.success() {
        bail!("chmod command failed");
    }

    println!(
        "{}",
        format!(
            "Updated permissions: chmod {} {}",
            resolved_mode,
            target.display()
        )
        .green()
    );
    Ok(())
}

/// ping (alias: png) - Check network latency to host
pub fn handle_ping(theme: &ColorfulTheme, host: Option<&str>) -> Result<()> {
    let target_host = match host {
        Some(h) => h.to_string(),
        None => {
            let cancel_btn = cancel_option();
            let hosts = [
                "1.1.1.1 (Cloudflare DNS)",
                "8.8.8.8 (Google DNS)",
                "google.com",
                "Custom host...",
                &cancel_btn,
            ];

            let selection = Select::with_theme(theme)
                .with_prompt("Select target host to ping")
                .items(&hosts)
                .default(0)
                .interact()?;

            match selection {
                0 => "1.1.1.1".to_string(),
                1 => "8.8.8.8".to_string(),
                2 => "google.com".to_string(),
                3 => {
                    let input: String = Input::with_theme(theme)
                        .with_prompt("Enter hostname or IP")
                        .allow_empty(true)
                        .interact_text()?;
                    let trimmed = input.trim().to_string();
                    if trimmed.is_empty() {
                        println!("Cancelled.");
                        return Ok(());
                    }
                    trimmed
                }
                _ => {
                    println!("Cancelled.");
                    return Ok(());
                }
            }
        }
    };

    println!(
        "{}",
        format!("Pinging {target_host} (4 packets)...").dimmed()
    );
    let status = Command::new("ping")
        .args(["-c", "4", &target_host])
        .status()
        .context("failed to execute ping")?;

    if !status.success() {
        bail!("ping request failed");
    }
    Ok(())
}

/// whoami (alias: who, user) - Print user and system details
pub fn handle_whoami() -> Result<()> {
    let user = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());
    let home = std::env::var("HOME").unwrap_or_else(|_| "-".to_string());
    let host_output = Command::new("hostname").output().ok();
    let host = host_output
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| "localhost".to_string());

    println!("{}", electric_blue("SYSTEM IDENTITY:").bold());
    println!("  User:     {}", user.bold());
    println!("  Host:     {}", host);
    println!("  Home:     {}", home);

    let id_output = Command::new("id").output().ok();
    if let Some(id_res) = id_output {
        let id_str = String::from_utf8_lossy(&id_res.stdout).trim().to_string();
        println!("  Identity: {id_str}");
    }

    Ok(())
}

/// time (alias: tim, date, dat) - Display formatted current date and time
pub fn handle_time(format: Option<&str>) -> Result<()> {
    let mut cmd = Command::new("date");
    if let Some(fmt) = format {
        cmd.arg(format!("+{fmt}"));
    }
    let status = cmd.status().context("failed to execute 'date'")?;
    if !status.success() {
        bail!("date command failed");
    }
    Ok(())
}

/// history (alias: his) - Display recent shell command history
pub fn handle_history(limit: Option<usize>) -> Result<()> {
    let home = std::env::var("HOME").unwrap_or_default();
    let zsh_history = PathBuf::from(&home).join(".zsh_history");
    let bash_history = PathBuf::from(&home).join(".bash_history");

    let history_file = if zsh_history.exists() {
        zsh_history
    } else if bash_history.exists() {
        bash_history
    } else {
        bail!("no shell history file found in home directory");
    };

    let bytes = fs::read(&history_file).context("failed to read shell history")?;
    let content = String::from_utf8_lossy(&bytes);

    let mut history_lines = Vec::new();
    for line in content.lines() {
        let clean_cmd = if let Some(idx) = line.find(';') {
            &line[idx + 1..]
        } else {
            line
        };
        if !clean_cmd.trim().is_empty() {
            history_lines.push(clean_cmd.trim().to_string());
        }
    }

    let max_count = limit.unwrap_or(20);
    let start_idx = history_lines.len().saturating_sub(max_count);

    println!("{}", electric_blue("RECENT COMMAND HISTORY:").bold());
    for (i, cmd) in history_lines.iter().skip(start_idx).enumerate() {
        println!("  {: >3}  {}", start_idx + i + 1, cmd);
    }

    Ok(())
}

/// which (alias: whc, loc) - Locate binary in PATH
pub fn handle_which(theme: &ColorfulTheme, binary: Option<&str>) -> Result<()> {
    let target_bin = match binary {
        Some(b) => b.to_string(),
        None => {
            let input: String = Input::with_theme(theme)
                .with_prompt("Executable binary name (leave blank to cancel)")
                .allow_empty(true)
                .interact_text()?;
            let trimmed = input.trim().to_string();
            if trimmed.is_empty() {
                println!("Cancelled.");
                return Ok(());
            }
            trimmed
        }
    };

    let output = Command::new("which")
        .arg(&target_bin)
        .output()
        .context("failed to execute which")?;

    if output.status.success() {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        println!("{} -> {}", electric_blue(&target_bin).bold(), path.green());
    } else {
        bail!("executable binary '{target_bin}' was not found in PATH");
    }

    Ok(())
}

/// env (alias: env) - Print or search environment variables
pub fn handle_env(key: Option<&str>) -> Result<()> {
    let mut vars: Vec<(String, String)> = std::env::vars().collect();
    vars.sort_by(|a, b| a.0.cmp(&b.0));

    if let Some(query) = key {
        let q_lower = query.to_lowercase();
        let matches: Vec<&(String, String)> = vars
            .iter()
            .filter(|(k, v)| {
                k.to_lowercase().contains(&q_lower) || v.to_lowercase().contains(&q_lower)
            })
            .collect();

        if matches.is_empty() {
            println!("No environment variables matching '{query}' found.");
        } else {
            for (k, v) in matches {
                println!("{}={v}", electric_blue(k).bold());
            }
        }
    } else {
        for (k, v) in vars {
            println!("{}={v}", electric_blue(&k).bold());
        }
    }

    Ok(())
}

/// test (alias: tst) - Smart test runner for active project
pub fn handle_test() -> Result<()> {
    let current_dir = std::env::current_dir().context("failed to get current directory")?;

    if current_dir.join("Cargo.toml").exists() {
        println!(
            "{}",
            electric_blue("Detected Rust project. Running 'cargo test'...").bold()
        );
        let status = Command::new("cargo")
            .arg("test")
            .status()
            .context("failed to run cargo test")?;
        if !status.success() {
            bail!("tests failed with exit code: {}", status);
        }
        return Ok(());
    }

    if current_dir.join("package.json").exists() {
        let runner = if current_dir.join("pnpm-lock.yaml").exists() {
            "pnpm"
        } else if current_dir.join("bun.lockb").exists() || current_dir.join("bun.lock").exists() {
            "bun"
        } else if current_dir.join("yarn.lock").exists() {
            "yarn"
        } else {
            "npm"
        };
        println!(
            "{}",
            electric_blue(&format!(
                "Detected Node.js project. Running '{runner} test'..."
            ))
            .bold()
        );
        let status = Command::new(runner)
            .arg("test")
            .status()
            .context("failed to run node test runner")?;
        if !status.success() {
            bail!("tests failed with exit code: {}", status);
        }
        return Ok(());
    }

    // Python check
    if current_dir.join("pytest.ini").exists()
        || current_dir.join("pyproject.toml").exists()
        || current_dir.join("requirements.txt").exists()
        || current_dir.join("tests").is_dir()
        || current_dir.join("test").is_dir()
    {
        println!(
            "{}",
            electric_blue("Detected Python project. Running tests...").bold()
        );
        let pytest_status = Command::new("pytest").status();
        match pytest_status {
            Ok(status) if status.success() => return Ok(()),
            Ok(status) => bail!("pytest failed with exit code: {}", status),
            Err(_) => {
                let status = Command::new("python3")
                    .args(["-m", "unittest", "discover"])
                    .status()
                    .context("failed to run python tests")?;
                if !status.success() {
                    bail!("python tests failed with exit code: {}", status);
                }
                return Ok(());
            }
        }
    }

    if current_dir.join("pubspec.yaml").exists() {
        println!(
            "{}",
            electric_blue("Detected Flutter/Dart project. Running 'flutter test'...").bold()
        );
        let status = Command::new("flutter")
            .arg("test")
            .status()
            .context("failed to run flutter test")?;
        if !status.success() {
            bail!("tests failed with exit code: {}", status);
        }
        return Ok(());
    }

    if current_dir.join("go.mod").exists() {
        println!(
            "{}",
            electric_blue("Detected Go project. Running 'go test ./...'...").bold()
        );
        let status = Command::new("go")
            .args(["test", "./..."])
            .status()
            .context("failed to run go test")?;
        if !status.success() {
            bail!("tests failed with exit code: {}", status);
        }
        return Ok(());
    }

    if current_dir.join("Makefile").exists() {
        println!(
            "{}",
            electric_blue("Detected Makefile. Running 'make test'...").bold()
        );
        let status = Command::new("make")
            .arg("test")
            .status()
            .context("failed to run make test")?;
        if !status.success() {
            bail!("make test failed with exit code: {}", status);
        }
        return Ok(());
    }

    bail!("No recognized project configuration found for automated testing in current directory.");
}

fn calculate_dir_size(path: &std::path::Path) -> u64 {
    let mut total = 0;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                total += calculate_dir_size(&p);
            } else if let Ok(meta) = entry.metadata() {
                total += meta.len();
            }
        }
    }
    total
}

/// clean (alias: cln) - Scans and removes heavy build caches & disposable folders
pub fn handle_clean(theme: &ColorfulTheme, target_path: Option<PathBuf>) -> Result<()> {
    let root = match target_path {
        Some(p) => p,
        None => std::env::current_dir().context("failed to get current directory")?,
    };

    println!(
        "{}",
        electric_blue(&format!(
            "Scanning for disposable build caches & artifacts in '{}'...",
            root.display()
        ))
        .bold()
    );

    let junk_names = [
        "target",
        "node_modules",
        "__pycache__",
        ".pytest_cache",
        ".venv",
        "venv",
        ".build",
        "build",
        ".dart_tool",
        "DerivedData",
        ".DS_Store",
    ];

    let mut found_items: Vec<(PathBuf, u64)> = Vec::new();

    fn scan_dir(
        dir: &std::path::Path,
        depth: usize,
        junk: &[&str],
        results: &mut Vec<(PathBuf, u64)>,
    ) {
        if depth > 3 {
            return;
        }
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                let name = match p.file_name().and_then(|n| n.to_str()) {
                    Some(n) => n,
                    None => continue,
                };

                if junk.contains(&name) {
                    let size = if p.is_dir() {
                        calculate_dir_size(&p)
                    } else {
                        p.metadata().map(|m| m.len()).unwrap_or(0)
                    };
                    results.push((p, size));
                } else if p.is_dir()
                    && !name.starts_with('.')
                    && name != "node_modules"
                    && name != "target"
                {
                    scan_dir(&p, depth + 1, junk, results);
                }
            }
        }
    }

    let mut sp = ui::Spinner::start("Scanning for disposable build caches & artifacts...");
    scan_dir(&root, 0, &junk_names, &mut found_items);
    sp.stop();

    if found_items.is_empty() {
        println!(
            "{}",
            "No disposable build artifacts or caches detected. Workspace is clean!"
                .green()
                .bold()
        );
        return Ok(());
    }

    let total_reclaimable: u64 = found_items.iter().map(|(_, sz)| sz).sum();

    let rows = [
        ("Directory", root.display().to_string()),
        (
            "Artifacts",
            format!("{} target(s) found", found_items.len())
                .yellow()
                .bold()
                .to_string(),
        ),
        (
            "Reclaimable",
            format_bytes(total_reclaimable).green().bold().to_string(),
        ),
    ];
    ui::print_card("🧹 CLEANUP TARGETS DETECTED", &rows);

    for (p, sz) in &found_items {
        let rel_path = p.strip_prefix(&root).unwrap_or(p);
        println!(
            "  • {: <32} {}",
            rel_path.display().to_string().yellow(),
            format_bytes(*sz).dimmed()
        );
    }
    println!();

    let cancel_btn = cancel_option();
    let action_options = [
        format!(
            "Delete all items (reclaim {})",
            format_bytes(total_reclaimable)
        ),
        "Select specific items to delete".to_string(),
        cancel_btn,
    ];

    let selection = Select::with_theme(theme)
        .with_prompt("Choose cleaning action")
        .items(&action_options)
        .default(0)
        .interact()?;

    match selection {
        0 => {
            let confirm = Confirm::with_theme(theme)
                .with_prompt(format!(
                    "Are you sure you want to delete all {} items?",
                    found_items.len()
                ))
                .default(false)
                .interact()?;

            if !confirm {
                println!("Cancelled.");
                return Ok(());
            }

            for (p, _) in &found_items {
                if p.is_dir() {
                    let _ = fs::remove_dir_all(p);
                } else {
                    let _ = fs::remove_file(p);
                }
            }
            println!(
                "{}",
                format!(
                    "Successfully cleaned all artifacts! Reclaimed {}.",
                    format_bytes(total_reclaimable)
                )
                .green()
                .bold()
            );
        }
        1 => {
            let mut item_labels: Vec<String> = found_items
                .iter()
                .map(|(p, sz)| {
                    let rel = p.strip_prefix(&root).unwrap_or(p);
                    format!("{} ({})", rel.display(), format_bytes(*sz))
                })
                .collect();
            item_labels.push(cancel_option());

            let chosen = Select::with_theme(theme)
                .with_prompt("Select item to delete")
                .items(&item_labels)
                .default(0)
                .interact()?;

            if chosen >= found_items.len() {
                println!("Cancelled.");
                return Ok(());
            }

            let (p, sz) = &found_items[chosen];
            let confirm = Confirm::with_theme(theme)
                .with_prompt(format!("Delete '{}' ({})?", p.display(), format_bytes(*sz)))
                .default(false)
                .interact()?;

            if confirm {
                if p.is_dir() {
                    fs::remove_dir_all(p)?;
                } else {
                    fs::remove_file(p)?;
                }
                println!(
                    "{}",
                    format!(
                        "Deleted '{}'. Reclaimed {}.",
                        p.display(),
                        format_bytes(*sz)
                    )
                    .green()
                    .bold()
                );
            } else {
                println!("Cancelled.");
            }
        }
        _ => {
            println!("Cancelled.");
        }
    }

    Ok(())
}

/// sync (alias: snc, git) - One-step git status, commit, and push
pub fn handle_sync(
    theme: &ColorfulTheme,
    message: Option<String>,
    switch_branch: bool,
) -> Result<()> {
    let check_git = Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .output();

    match check_git {
        Ok(output) if output.status.success() => {}
        _ => bail!("Current directory is not inside a git repository."),
    }

    if switch_branch {
        let output = Command::new("git")
            .args(["branch", "--format=%(refname:short)"])
            .output()
            .context("failed to list git branches")?;
        let text = String::from_utf8_lossy(&output.stdout);
        let mut branches: Vec<String> = text
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect();

        if branches.is_empty() {
            println!("No branches found.");
            return Ok(());
        }

        branches.push(cancel_option());

        let selection = Select::with_theme(theme)
            .with_prompt("Select branch to switch to")
            .items(&branches)
            .default(0)
            .interact()?;

        if selection >= branches.len() - 1 {
            println!("Cancelled.");
            return Ok(());
        }

        let target_branch = &branches[selection];
        let status = Command::new("git")
            .args(["checkout", target_branch])
            .status()?;
        if !status.success() {
            bail!("failed to switch to branch '{target_branch}'");
        }
        println!(
            "{}",
            format!("Switched to branch '{target_branch}'.")
                .green()
                .bold()
        );
        return Ok(());
    }

    let status_output = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .context("failed to check git status")?;
    let status_text = String::from_utf8_lossy(&status_output.stdout);

    if status_text.trim().is_empty() {
        println!(
            "{}",
            electric_blue("Working tree clean. Pulling latest changes from remote...").bold()
        );
        let pull_status = Command::new("git").args(["pull", "--rebase"]).status()?;
        if pull_status.success() {
            println!(
                "{}",
                "Repository is clean and up-to-date with remote."
                    .green()
                    .bold()
            );
        } else {
            bail!("git pull failed");
        }
        return Ok(());
    }

    let current_branch = Command::new("git")
        .args(["branch", "--show-current"])
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| "detached".to_string());

    let branch_badge = format!(
        "{} {}",
        current_branch.magenta().bold(),
        " 🌿 [Git] ".bold().bright_green().on_black()
    );
    let rows = [
        ("Branch", branch_badge),
        (
            "Modified Files",
            format!("{} file(s) changed", status_text.lines().count())
                .yellow()
                .bold()
                .to_string(),
        ),
    ];
    ui::print_card("GIT WORKSPACE STATUS", &rows);

    for line in status_text.lines().take(15) {
        println!("  {line}");
    }
    if status_text.lines().count() > 15 {
        println!(
            "  ... and {} more file(s)",
            status_text.lines().count() - 15
        );
    }
    println!();

    let commit_msg = match message {
        Some(m) if !m.trim().is_empty() => m,
        _ => {
            let input: String = Input::with_theme(theme)
                .with_prompt("Commit message (leave blank to cancel)")
                .allow_empty(true)
                .interact_text()?;
            let trimmed = input.trim().to_string();
            if trimmed.is_empty() {
                println!("Cancelled.");
                return Ok(());
            }
            trimmed
        }
    };

    println!(
        "{}",
        electric_blue("Staging and committing all changes...").bold()
    );
    let add_status = Command::new("git").args(["add", "-A"]).status()?;
    if !add_status.success() {
        bail!("git add failed");
    }

    let commit_status = Command::new("git")
        .args(["commit", "-m", &commit_msg])
        .status()?;
    if !commit_status.success() {
        bail!("git commit failed");
    }

    println!("{}", electric_blue("Pushing changes to remote...").bold());
    let push_status = Command::new("git").arg("push").status()?;
    if !push_status.success() {
        bail!("git push failed");
    }

    println!(
        "{}",
        "Successfully synced, committed, and pushed changes!"
            .green()
            .bold()
    );
    Ok(())
}

fn copy_to_clipboard(text: &str) -> Result<()> {
    let mut child = Command::new("pbcopy")
        .stdin(std::process::Stdio::piped())
        .spawn()
        .context("failed to run pbcopy")?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(text.as_bytes())?;
    }
    child.wait()?;
    Ok(())
}

/// network (alias: net, ip) - Inspect LAN and public IP addresses
pub fn handle_net(theme: &ColorfulTheme) -> Result<()> {
    let mut sp = ui::Spinner::start("Inspecting network interfaces and IP addresses...");

    let mut local_ips = Vec::new();
    for iface in ["en0", "en1", "en2", "en3", "en4", "bridge0"] {
        if let Ok(output) = Command::new("ipconfig").args(["getifaddr", iface]).output() {
            let ip = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !ip.is_empty() {
                local_ips.push((iface.to_string(), ip));
            }
        }
    }

    let public_ip = Command::new("curl")
        .args(["-s", "--max-time", "3", "https://api.ipify.org"])
        .output()
        .ok()
        .and_then(|out| {
            if out.status.success() {
                let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !s.is_empty() { Some(s) } else { None }
            } else {
                None
            }
        });

    sp.stop();

    let mut rows: Vec<(&str, String)> = Vec::new();
    if local_ips.is_empty() {
        rows.push(("Local IP", "Not connected to LAN".dimmed().to_string()));
    } else {
        for (iface, ip) in &local_ips {
            rows.push((
                iface.as_str(),
                format!(
                    "{} {}",
                    ip.bold(),
                    " 🌐 [LAN] ".bold().bright_blue().on_black()
                ),
            ));
        }
    }

    match &public_ip {
        Some(pub_ip) => rows.push((
            "Public IP",
            format!(
                "{} {}",
                pub_ip.bold().green(),
                " 🌍 [WAN] ".bold().bright_green().on_black()
            ),
        )),
        None => rows.push(("Public IP", "Offline / unreachable".dimmed().to_string())),
    }

    ui::print_card("🌐 NETWORK INTERFACE SUMMARY", &rows);

    let cancel_btn = cancel_option();
    let mut copy_options = Vec::new();
    for (iface, ip) in &local_ips {
        copy_options.push(format!("Copy Local IP ({}) to clipboard: {}", iface, ip));
    }
    if let Some(pub_ip) = &public_ip {
        copy_options.push(format!("Copy Public IP to clipboard: {}", pub_ip));
    }
    copy_options.push(cancel_btn);

    let selection = Select::with_theme(theme)
        .with_prompt("Select action")
        .items(&copy_options)
        .default(0)
        .interact()?;

    if selection < local_ips.len() {
        let ip_to_copy = &local_ips[selection].1;
        copy_to_clipboard(ip_to_copy)?;
        println!(
            "{}",
            format!("Copied '{}' to clipboard!", ip_to_copy)
                .green()
                .bold()
        );
    } else if public_ip.is_some() && selection == local_ips.len() {
        let pub_ip = public_ip.unwrap();
        copy_to_clipboard(&pub_ip)?;
        println!(
            "{}",
            format!("Copied '{}' to clipboard!", pub_ip).green().bold()
        );
    } else {
        println!("Cancelled.");
    }

    Ok(())
}

/// share (alias: shr) - Instant local HTTP file server on local network
pub fn handle_share(theme: &ColorfulTheme, port: Option<u16>, path: Option<PathBuf>) -> Result<()> {
    let current_dir = match path {
        Some(p) => p,
        None => std::env::current_dir().context("failed to get current directory")?,
    };

    let selected_port = match port {
        Some(p) => p,
        None => {
            let input: String = Input::with_theme(theme)
                .with_prompt("Port to serve on (press Enter for 8000)")
                .allow_empty(true)
                .default("8000".to_string())
                .interact_text()?;
            input.trim().parse::<u16>().unwrap_or(8000)
        }
    };

    let local_ip = Command::new("ipconfig")
        .args(["getifaddr", "en0"])
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "127.0.0.1".to_string());

    println!();
    println!("{}", electric_blue("LOCAL NETWORK FILE SERVER").bold());
    println!("  Directory:  {}", current_dir.display().to_string().bold());
    println!(
        "  Local:      {}",
        format!("http://localhost:{selected_port}").green()
    );
    println!(
        "  LAN Access: {}",
        format!("http://{local_ip}:{selected_port}").bold().green()
    );
    println!();
    println!("{}", "Press Ctrl+C to stop sharing.".dimmed());
    println!();

    let status = Command::new("python3")
        .args([
            "-m",
            "http.server",
            &selected_port.to_string(),
            "--directory",
        ])
        .arg(&current_dir)
        .status()
        .context("failed to start python3 http.server")?;

    if !status.success() {
        bail!("http server stopped unexpectedly");
    }

    Ok(())
}

/// bench (alias: bnc, time) - High-resolution command execution benchmark
pub fn handle_bench(theme: &ColorfulTheme, command_args: &[String]) -> Result<()> {
    let target_cmd = if command_args.is_empty() {
        let input: String = Input::with_theme(theme)
            .with_prompt("Command to benchmark (leave blank to cancel)")
            .allow_empty(true)
            .interact_text()?;
        let trimmed = input.trim().to_string();
        if trimmed.is_empty() {
            println!("Cancelled.");
            return Ok(());
        }
        trimmed
    } else {
        command_args.join(" ")
    };

    println!(
        "{}",
        electric_blue(&format!("Benchmarking: '{}'...", target_cmd)).bold()
    );
    println!(
        "{}",
        "--------------------------------------------------".dimmed()
    );

    let start = std::time::Instant::now();
    let status = Command::new("sh")
        .arg("-c")
        .arg(&target_cmd)
        .status()
        .with_context(|| format!("failed to execute command '{target_cmd}'"))?;
    let duration = start.elapsed();

    println!(
        "{}",
        "--------------------------------------------------".dimmed()
    );
    let exit_val = if status.success() {
        format!("{}", "0 (Success) 🟢".green().bold())
    } else {
        format!(
            "{}",
            format!("{} (Failed) 🔴", status.code().unwrap_or(-1))
                .red()
                .bold()
        )
    };

    let rows = [
        ("Command", target_cmd.yellow().bold().to_string()),
        (
            "Duration",
            format!("{:.2?} ({:.3}s)", duration, duration.as_secs_f64())
                .green()
                .bold()
                .to_string(),
        ),
        ("Exit Code", exit_val),
    ];
    ui::print_card("⏱️  BENCHMARK RESULT", &rows);

    Ok(())
}

/// speedtest (alias: spd, speed) - Measure network bandwidth throughput and responsiveness
pub fn handle_speedtest(_theme: &ColorfulTheme, sequential: bool) -> Result<()> {
    let has_network_quality = Command::new("which")
        .arg("networkQuality")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if has_network_quality {
        let mut sp = ui::Spinner::start(
            "Testing internet bandwidth throughput, latency & responsiveness...",
        );
        let mut cmd = Command::new("networkQuality");
        cmd.arg("-c");
        if sequential {
            cmd.arg("-s");
        }
        let output = cmd.output().context("failed to execute networkQuality")?;
        sp.stop();

        if output.status.success() {
            let json_str = String::from_utf8_lossy(&output.stdout);

            let dl_bps = parse_json_number(&json_str, "dl_throughput");
            let ul_bps = parse_json_number(&json_str, "ul_throughput");
            let base_rtt = parse_json_float(&json_str, "base_rtt");
            let responsiveness = parse_json_float(&json_str, "responsiveness");
            let iface = parse_json_string(&json_str, "interface_name");
            let endpoint = parse_json_string(&json_str, "test_endpoint");

            let mut rows = Vec::new();
            if let Some(dl) = dl_bps {
                rows.push((
                    "Download",
                    format!("{} 🟢", format_bps(dl)).green().bold().to_string(),
                ));
            }
            if let Some(ul) = ul_bps {
                rows.push((
                    "Upload",
                    format!("{} 🟢", format_bps(ul)).green().bold().to_string(),
                ));
            }
            if let Some(rtt) = base_rtt {
                rows.push(("Base Latency", format!("{:.1} ms", rtt)));
            }
            if let Some(rpm) = responsiveness {
                rows.push(("Responsiveness", responsiveness_rating(rpm)));
            }
            if let Some(i) = iface {
                rows.push((
                    "Interface",
                    format!(
                        "{} {}",
                        i.bold(),
                        " 🌐 [LAN] ".bold().bright_blue().on_black()
                    ),
                ));
            }
            if let Some(ep) = endpoint {
                rows.push(("Server", ep));
            }

            ui::print_card("🚀 NETWORK SPEEDTEST RESULT", &rows);
            return Ok(());
        }
    }

    // Fallback: check speedtest-cli
    let has_speedtest_cli = Command::new("which")
        .arg("speedtest-cli")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if has_speedtest_cli {
        println!("{}", electric_blue("Running speedtest-cli...").bold());
        let status = Command::new("speedtest-cli").arg("--simple").status()?;
        if status.success() {
            return Ok(());
        }
    }

    // Fallback: fast curl download test
    let mut sp =
        ui::Spinner::start("Measuring download speed via Cloudflare CDN (10MB payload)...");
    let start = std::time::Instant::now();
    let curl_status = Command::new("curl")
        .args([
            "-s",
            "-o",
            "/dev/null",
            "-w",
            "%{speed_download}",
            "https://speed.cloudflare.com/__down?bytes=10000000",
        ])
        .output();
    sp.stop();

    if let Ok(out) = curl_status
        && out.status.success()
    {
        let speed_str = String::from_utf8_lossy(&out.stdout).trim().to_string();
        let bytes_per_sec = speed_str.parse::<f64>().unwrap_or(0.0);
        let mbps = (bytes_per_sec * 8.0) / 1_000_000.0;
        let duration = start.elapsed();

        let rows = [
            (
                "Download",
                format!("{:.2} Mbps 🟢", mbps).green().bold().to_string(),
            ),
            ("Payload", "10 MB".to_string()),
            (
                "Duration",
                format!("{:.2?} ({:.2}s)", duration, duration.as_secs_f64()),
            ),
            ("Server", "Cloudflare Speed Test CDN".to_string()),
        ];
        ui::print_card("🚀 NETWORK SPEEDTEST RESULT (FALLBACK)", &rows);
        return Ok(());
    }

    bail!(
        "unable to run speedtest: neither networkQuality nor working internet connection available"
    );
}

fn parse_json_number(json: &str, key: &str) -> Option<u64> {
    let pattern = format!("\"{}\"", key);
    if let Some(pos) = json.find(&pattern) {
        let after = &json[pos + pattern.len()..];
        if let Some(colon) = after.find(':') {
            let val_str = after[colon + 1..].trim_start();
            let num_str: String = val_str.chars().take_while(|c| c.is_ascii_digit()).collect();
            return num_str.parse::<u64>().ok();
        }
    }
    None
}

fn parse_json_float(json: &str, key: &str) -> Option<f64> {
    let pattern = format!("\"{}\"", key);
    if let Some(pos) = json.find(&pattern) {
        let after = &json[pos + pattern.len()..];
        if let Some(colon) = after.find(':') {
            let val_str = after[colon + 1..].trim_start();
            let num_str: String = val_str
                .chars()
                .take_while(|c| c.is_ascii_digit() || *c == '.')
                .collect();
            return num_str.parse::<f64>().ok();
        }
    }
    None
}

fn parse_json_string(json: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{}\"", key);
    if let Some(pos) = json.find(&pattern) {
        let after = &json[pos + pattern.len()..];
        if let Some(colon) = after.find(':') {
            let val_str = after[colon + 1..].trim_start();
            if let Some(quote_start) = val_str.find('"') {
                let rest = &val_str[quote_start + 1..];
                if let Some(quote_end) = rest.find('"') {
                    return Some(rest[..quote_end].to_string());
                }
            }
        }
    }
    None
}

fn format_bps(bps: u64) -> String {
    let mbps = bps as f64 / 1_000_000.0;
    if mbps >= 1000.0 {
        format!("{:.2} Gbps", mbps / 1000.0)
    } else {
        format!("{:.2} Mbps", mbps)
    }
}

fn responsiveness_rating(rpm: f64) -> String {
    if rpm >= 1000.0 {
        format!("High ({:.0} RPM) 🟢", rpm)
    } else if rpm >= 400.0 {
        format!("Medium ({:.0} RPM) 🟡", rpm)
    } else {
        format!("Low ({:.0} RPM) 🔴", rpm)
    }
}

/// docker (alias: dck) - Interactive Docker container, image, and storage manager
pub fn handle_docker(theme: &ColorfulTheme) -> Result<()> {
    let has_docker = Command::new("which")
        .arg("docker")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_docker {
        bail!("docker CLI is not installed or not in PATH");
    }

    // Check daemon connection
    let daemon_check = Command::new("docker").args(["ps", "-q"]).output();
    let daemon_ok = daemon_check.map(|o| o.status.success()).unwrap_or(false);

    if !daemon_ok {
        let rows = [
            ("Status", "Docker Daemon Not Running 🔴".to_string()),
            (
                "Guidance",
                "Please start OrbStack or Docker Desktop application".to_string(),
            ),
        ];
        ui::print_card("🐳 DOCKER DAEMON STATUS", &rows);
        return Ok(());
    }

    let cancel_btn = cancel_option();
    let options = [
        "📦  Containers   (list, inspect, restart, stop, follow logs)",
        "🖼️   Images       (list local images and disk usage)",
        "🧹  System Prune (clean stopped containers, dangling images & caches)",
        "🚀  Compose Up   (run 'docker compose up -d')",
        "🛑  Compose Down (run 'docker compose down')",
        cancel_btn.as_str(),
    ];

    ui::print_key_hints();
    let selection = Select::with_theme(theme)
        .with_prompt("Select Docker action")
        .items(&options)
        .default(0)
        .interact()?;

    match selection {
        0 => manage_containers(theme)?,
        1 => {
            println!("{}", electric_blue("Local Docker Images:").bold());
            let status = Command::new("docker")
                .args([
                    "images",
                    "--format",
                    "table {{.Repository}}\t{{.Tag}}\t{{.Size}}\t{{.CreatedSince}}",
                ])
                .status()?;
            if !status.success() {
                bail!("failed to list docker images");
            }
        }
        2 => {
            let confirm = Confirm::with_theme(theme)
                .with_prompt("Prune all stopped containers, unused networks, and dangling images?")
                .default(false)
                .interact()?;
            if confirm {
                println!(
                    "{}",
                    electric_blue("Pruning unused Docker resources...").bold()
                );
                let status = Command::new("docker")
                    .args(["system", "prune", "-f"])
                    .status()?;
                if status.success() {
                    println!("{}", "Docker system pruned successfully! 🟢".green().bold());
                }
            } else {
                println!("Cancelled.");
            }
        }
        3 => {
            println!(
                "{}",
                electric_blue("Starting Docker Compose services (detached)...").bold()
            );
            let status = Command::new("docker")
                .args(["compose", "up", "-d"])
                .status()?;
            if !status.success() {
                bail!("docker compose up failed");
            }
        }
        4 => {
            println!(
                "{}",
                electric_blue("Stopping Docker Compose services...").bold()
            );
            let status = Command::new("docker").args(["compose", "down"]).status()?;
            if !status.success() {
                bail!("docker compose down failed");
            }
        }
        _ => {
            println!("Cancelled.");
        }
    }

    Ok(())
}

fn manage_containers(theme: &ColorfulTheme) -> Result<()> {
    let output = Command::new("docker")
        .args([
            "ps",
            "-a",
            "--format",
            "{{.ID}}\t{{.Names}}\t{{.Status}}\t{{.Image}}",
        ])
        .output()
        .context("failed to list containers")?;

    let text = String::from_utf8_lossy(&output.stdout);
    let mut containers = Vec::new();
    for line in text.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 4 {
            containers.push((
                parts[0].to_string(), // ID
                parts[1].to_string(), // Name
                parts[2].to_string(), // Status
                parts[3].to_string(), // Image
            ));
        }
    }

    if containers.is_empty() {
        println!(
            "{}",
            "No Docker containers found (running or stopped).".dimmed()
        );
        return Ok(());
    }

    let mut menu_items: Vec<String> = containers
        .iter()
        .map(|(id, name, status, _image)| {
            let status_badge = if status.to_lowercase().starts_with("up") {
                " 🟢 [UP] ".bold().bright_green().on_black()
            } else {
                " 🔴 [EXITED] ".bold().bright_red().on_black()
            };
            format!(
                "{:<20} {} {:<12} ({})",
                name.bold(),
                status_badge,
                id.dimmed(),
                status
            )
        })
        .collect();
    menu_items.push(cancel_option());

    ui::print_key_hints();
    let selection = FuzzySelect::with_theme(theme)
        .with_prompt("Select container to manage (type to filter)")
        .items(&menu_items)
        .default(0)
        .interact()?;

    if selection >= containers.len() {
        println!("Cancelled.");
        return Ok(());
    }

    let (cid, cname, _, _) = &containers[selection];

    let cancel_btn = cancel_option();
    let actions = [
        "📜  Follow Live Logs (Ctrl+C to exit)",
        "🔄  Restart Container",
        "🛑  Stop Container",
        "▶️   Start Container",
        cancel_btn.as_str(),
    ];

    let action = Select::with_theme(theme)
        .with_prompt(format!("Manage container '{cname}'"))
        .items(&actions)
        .default(0)
        .interact()?;

    match action {
        0 => {
            println!(
                "{}",
                electric_blue(&format!("Streaming logs for '{cname}' (Ctrl+C to exit)...")).bold()
            );
            let _ = Command::new("docker").args(["logs", "-f", cid]).status();
        }
        1 => {
            println!(
                "{}",
                electric_blue(&format!("Restarting container '{cname}'...")).bold()
            );
            let status = Command::new("docker").args(["restart", cid]).status()?;
            if status.success() {
                println!(
                    "{}",
                    format!("Container '{cname}' restarted! 🟢").green().bold()
                );
            }
        }
        2 => {
            println!(
                "{}",
                electric_blue(&format!("Stopping container '{cname}'...")).bold()
            );
            let status = Command::new("docker").args(["stop", cid]).status()?;
            if status.success() {
                println!(
                    "{}",
                    format!("Container '{cname}' stopped! 🛑").yellow().bold()
                );
            }
        }
        3 => {
            println!(
                "{}",
                electric_blue(&format!("Starting container '{cname}'...")).bold()
            );
            let status = Command::new("docker").args(["start", cid]).status()?;
            if status.success() {
                println!(
                    "{}",
                    format!("Container '{cname}' started! 🟢").green().bold()
                );
            }
        }
        _ => {
            println!("Cancelled.");
        }
    }

    Ok(())
}

/// secret (alias: sec, dotenv) - Smart .env validator, diff checker and secure inspector
pub fn handle_secret(theme: &ColorfulTheme, fix: bool) -> Result<()> {
    let current_dir = std::env::current_dir().context("failed to get current directory")?;

    let env_file = current_dir.join(".env");
    let example_file = [
        current_dir.join(".env.example"),
        current_dir.join(".env.sample"),
        current_dir.join(".env.template"),
    ]
    .into_iter()
    .find(|p| p.exists());

    if !env_file.exists() && example_file.is_none() {
        println!(
            "{}",
            "No .env or .env.example files found in current directory.".dimmed()
        );
        return Ok(());
    }

    fn parse_env_file(path: &std::path::Path) -> Vec<(String, String)> {
        let mut vars = Vec::new();
        if let Ok(content) = fs::read_to_string(path) {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    continue;
                }
                if let Some((k, v)) = trimmed.split_once('=') {
                    vars.push((k.trim().to_string(), v.trim().to_string()));
                }
            }
        }
        vars
    }

    let env_vars = if env_file.exists() {
        parse_env_file(&env_file)
    } else {
        Vec::new()
    };

    let example_vars = example_file
        .as_ref()
        .map(|p| parse_env_file(p))
        .unwrap_or_default();

    let env_keys: std::collections::HashSet<String> =
        env_vars.iter().map(|(k, _)| k.clone()).collect();
    let example_keys: std::collections::HashSet<String> =
        example_vars.iter().map(|(k, _)| k.clone()).collect();

    let missing_keys: Vec<String> = example_keys
        .iter()
        .filter(|k| !env_keys.contains(*k))
        .cloned()
        .collect();

    let status_val = if missing_keys.is_empty() {
        "All required variables defined 🟢"
            .green()
            .bold()
            .to_string()
    } else {
        format!("Missing {} required variable(s) 🔴", missing_keys.len())
            .red()
            .bold()
            .to_string()
    };

    let rows = [
        (
            ".env Path",
            if env_file.exists() {
                ".env (present)".green().bold().to_string()
            } else {
                ".env (missing)".red().bold().to_string()
            },
        ),
        (
            ".env.example",
            if example_file.is_some() {
                "Present 🟢".to_string()
            } else {
                "None found".dimmed().to_string()
            },
        ),
        ("Total Defined", format!("{} variables", env_vars.len())),
        ("Validation", status_val),
    ];
    ui::print_card("🔐 ENVIRONMENT SECRET VALIDATOR", &rows);

    if !missing_keys.is_empty() {
        println!("{}", "MISSING REQUIRED VARIABLES:".bold().red());
        for k in &missing_keys {
            println!(
                "  {} {}",
                "🔴 [MISSING]".bold().bright_red().on_black(),
                k.bold()
            );
        }
        println!();

        if fix
            || Confirm::with_theme(theme)
                .with_prompt("Append missing variables from .env.example into .env?")
                .default(true)
                .interact()?
        {
            let mut to_append = String::new();
            to_append.push_str("\n# Appended by 'run secret'\n");
            for k in &missing_keys {
                let default_val = example_vars
                    .iter()
                    .find(|(ek, _)| ek == k)
                    .map(|(_, v)| v.as_str())
                    .unwrap_or("");
                to_append.push_str(&format!("{k}={default_val}\n"));
            }
            let mut file = fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&env_file)
                .context("failed to update .env file")?;
            file.write_all(to_append.as_bytes())?;
            println!(
                "{}",
                "Appended missing variables to .env successfully! 🟢"
                    .green()
                    .bold()
            );
        }
    }

    if !env_vars.is_empty() {
        println!("{}", "DEFINED VARIABLES (SAFELY MASKED):".bold());
        for (k, v) in env_vars.iter().take(20) {
            let masked = if v.len() > 6 {
                format!("{}****{}", &v[..2], &v[v.len() - 2..])
            } else {
                "******".to_string()
            };
            println!("  {: <24} = {}", k.bold(), masked.dimmed());
        }
        if env_vars.len() > 20 {
            println!("  ... and {} more variables", env_vars.len() - 20);
        }
        println!();
    }

    Ok(())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Snippet {
    pub title: String,
    pub command: String,
}

fn snippets_file_path() -> Result<PathBuf> {
    let dir = crate::config::config_dir()?;
    fs::create_dir_all(&dir)?;
    Ok(dir.join("snippets.json"))
}

fn load_snippets() -> Vec<Snippet> {
    let path = match snippets_file_path() {
        Ok(p) => p,
        Err(_) => return default_snippets(),
    };

    if !path.exists() {
        let defaults = default_snippets();
        if let Ok(json) = serde_json::to_string_pretty(&defaults) {
            let _ = fs::write(&path, json);
        }
        return defaults;
    }

    fs::read_to_string(&path)
        .ok()
        .and_then(|c| serde_json::from_str(&c).ok())
        .unwrap_or_else(default_snippets)
}

fn save_snippets(snippets: &[Snippet]) -> Result<()> {
    let path = snippets_file_path()?;
    let json = serde_json::to_string_pretty(snippets)?;
    fs::write(path, json)?;
    Ok(())
}

fn default_snippets() -> Vec<Snippet> {
    vec![
        Snippet {
            title: "Git undo last commit (keep changes staged)".to_string(),
            command: "git reset --soft HEAD~1".to_string(),
        },
        Snippet {
            title: "Docker stop all running containers".to_string(),
            command: "docker stop $(docker ps -a -q)".to_string(),
        },
        Snippet {
            title: "FFmpeg convert video to animated GIF".to_string(),
            command: "ffmpeg -i input.mp4 -vf \"fps=10,scale=640:-1:flags=lanczos\" output.gif"
                .to_string(),
        },
        Snippet {
            title: "Find process listening on specific port".to_string(),
            command: "lsof -iTCP -sTCP:LISTEN -P".to_string(),
        },
        Snippet {
            title: "Tar create compressed archive".to_string(),
            command: "tar -czvf archive.tar.gz folder/".to_string(),
        },
    ]
}

/// memo (alias: mem, clip) - Developer quick command snippet bookmark vault
pub fn handle_memo(
    theme: &ColorfulTheme,
    action: Option<&str>,
    arg1: Option<&str>,
    arg2: Option<&str>,
) -> Result<()> {
    let mut snippets = load_snippets();

    match action.map(|a| a.to_lowercase()).as_deref() {
        Some("add") => {
            let title = match arg1 {
                Some(t) => t.to_string(),
                None => {
                    let input: String = Input::with_theme(theme)
                        .with_prompt("Snippet title (leave blank to cancel)")
                        .allow_empty(true)
                        .interact_text()?;
                    let trimmed = input.trim().to_string();
                    if trimmed.is_empty() {
                        println!("Cancelled.");
                        return Ok(());
                    }
                    trimmed
                }
            };
            let command = match arg2 {
                Some(c) => c.to_string(),
                None => {
                    let input: String = Input::with_theme(theme)
                        .with_prompt("Snippet command (leave blank to cancel)")
                        .allow_empty(true)
                        .interact_text()?;
                    let trimmed = input.trim().to_string();
                    if trimmed.is_empty() {
                        println!("Cancelled.");
                        return Ok(());
                    }
                    trimmed
                }
            };
            snippets.push(Snippet {
                title: title.clone(),
                command: command.clone(),
            });
            save_snippets(&snippets)?;
            println!("{}", format!("Added snippet '{title}'! 🟢").green().bold());
        }
        Some("remove") | Some("rm") | Some("del") => {
            if snippets.is_empty() {
                println!("{}", "No snippets available to remove.".dimmed());
                return Ok(());
            }
            let mut items: Vec<String> = snippets
                .iter()
                .map(|s| format!("{:<36} {}", s.title.bold(), s.command.dimmed()))
                .collect();
            items.push(cancel_option());

            let selection = Select::with_theme(theme)
                .with_prompt("Select snippet to remove")
                .items(&items)
                .default(0)
                .interact()?;

            if selection >= snippets.len() {
                println!("Cancelled.");
                return Ok(());
            }

            let removed = snippets.remove(selection);
            save_snippets(&snippets)?;
            println!(
                "{}",
                format!("Removed snippet '{}' 🗑️", removed.title)
                    .yellow()
                    .bold()
            );
        }
        _ => {
            if snippets.is_empty() {
                println!(
                    "{}",
                    "Snippet vault is empty. Add one with 'run memo add'!".dimmed()
                );
                return Ok(());
            }

            let mut items: Vec<String> = snippets
                .iter()
                .map(|s| format!("{:<36} {}", s.title.bold(), s.command.dimmed()))
                .collect();
            items.push(cancel_option());

            ui::print_key_hints();
            let selection = FuzzySelect::with_theme(theme)
                .with_prompt("Select snippet to copy to clipboard (type to filter)")
                .items(&items)
                .default(0)
                .interact()?;

            if selection >= snippets.len() {
                println!("Cancelled.");
                return Ok(());
            }

            let selected = &snippets[selection];
            copy_to_clipboard(&selected.command)?;
            println!(
                "{}",
                format!("Copied to clipboard: '{}' 📋", selected.command)
                    .green()
                    .bold()
            );
        }
    }

    Ok(())
}

/// config (alias: cfg) - Manage CLI settings, colors, editor, and auto-clear
pub fn handle_config(
    theme: &ColorfulTheme,
    action: Option<&str>,
    key: Option<&str>,
    val: Option<&str>,
) -> Result<()> {
    match action {
        Some("get") => {
            let key_name =
                key.context("missing configuration key (e.g. 'run cfg get primary_color')")?;
            let cfg = config::load_config();
            match key_name.to_lowercase().as_str() {
                "primary_color" | "color" => println!(
                    "{}",
                    cfg.primary_color.unwrap_or_else(|| "electric-blue".into())
                ),
                "auto_clear" | "clear" => println!("{}", cfg.auto_clear.unwrap_or(false)),
                "compact_mode" | "compact" => println!("{}", cfg.compact_mode.unwrap_or(false)),
                "default_ide" | "ide" => {
                    println!("{}", cfg.default_ide.unwrap_or_else(|| "ask".into()))
                }
                "search_engine" | "engine" => {
                    println!("{}", cfg.search_engine.unwrap_or_else(|| "google".into()))
                }
                "custom_hubs" | "hubs" => {
                    for h in cfg.custom_hubs.unwrap_or_default() {
                        println!("{}", h.display());
                    }
                }
                other => bail!(
                    "unknown configuration key '{other}'. Valid keys: primary_color, auto_clear, compact_mode, default_ide, search_engine, custom_hubs"
                ),
            }
        }
        Some("set") => {
            let key_name = key.context(
                "missing configuration key (e.g. 'run cfg set primary_color \"#ff007f\"')",
            )?;
            let val_str =
                val.context("missing value to set (e.g. 'run cfg set primary_color \"#ff007f\"')")?;
            let mut cfg = config::load_config();
            match key_name.to_lowercase().as_str() {
                "primary_color" | "color" => {
                    cfg.primary_color = Some(val_str.to_string());
                    config::save_config(&cfg)?;
                    println!("{} Set primary_color to '{}'", "✔".green().bold(), val_str);
                }
                "auto_clear" | "clear" => {
                    let b =
                        val_str == "true" || val_str == "1" || val_str == "yes" || val_str == "on";
                    cfg.auto_clear = Some(b);
                    config::save_config(&cfg)?;
                    println!("{} Set auto_clear to {}", "✔".green().bold(), b);
                }
                "compact_mode" | "compact" => {
                    let b =
                        val_str == "true" || val_str == "1" || val_str == "yes" || val_str == "on";
                    cfg.compact_mode = Some(b);
                    config::save_config(&cfg)?;
                    println!("{} Set compact_mode to {}", "✔".green().bold(), b);
                }
                "default_ide" | "ide" => {
                    cfg.default_ide = Some(val_str.to_string());
                    config::save_config(&cfg)?;
                    println!("{} Set default_ide to '{}'", "✔".green().bold(), val_str);
                }
                "search_engine" | "engine" => {
                    cfg.search_engine = Some(val_str.to_lowercase());
                    config::save_config(&cfg)?;
                    println!("{} Set search_engine to '{}'", "✔".green().bold(), val_str.to_lowercase());
                }
                "custom_hubs" | "hubs" => {
                    let mut hubs = cfg.custom_hubs.unwrap_or_default();
                    hubs.push(PathBuf::from(val_str));
                    cfg.custom_hubs = Some(hubs);
                    config::save_config(&cfg)?;
                    println!("{} Added '{}' to custom_hubs", "✔".green().bold(), val_str);
                }
                other => bail!(
                    "unknown configuration key '{other}'. Valid keys: primary_color, auto_clear, compact_mode, default_ide, search_engine, custom_hubs"
                ),
            }
        }
        Some("path") => {
            let path = config::config_path()?;
            println!("{}", path.display());
        }
        Some("edit") => {
            let path = config::config_path()?;
            if !path.exists() {
                let _ = config::load_config();
            }
            let _ = Command::new("open").arg(&path).status();
            println!("Opened config file in system editor: {}", path.display());
        }
        Some("reset") => {
            if Confirm::with_theme(theme)
                .with_prompt("Reset configuration to factory defaults?")
                .default(false)
                .interact()?
            {
                let path = config::config_path()?;
                if path.exists() {
                    let _ = fs::remove_file(path);
                }
                let _ = config::load_config();
                println!("{} Configuration reset to default.", "✔".green().bold());
            } else {
                println!("Cancelled.");
            }
        }
        None | Some("interactive") | Some("menu") => {
            manage_config_interactive(theme)?;
        }
        Some(other) => {
            bail!("unknown config action '{other}'. Usage: run config [get|set|path|edit|reset]");
        }
    }
    Ok(())
}

fn manage_config_interactive(theme: &ColorfulTheme) -> Result<()> {
    ui::maybe_auto_clear();
    ui::print_banner();
    ui::render_breadcrumbs(&["run", "Configuration"]);

    let cfg = config::load_config();
    let primary = cfg
        .primary_color
        .clone()
        .unwrap_or_else(|| "electric-blue".to_string());
    let (r, g, b) = config::parse_color(&primary);
    let swatch = format!("■ {}", primary)
        .truecolor(r, g, b)
        .bold()
        .to_string();
    let auto_clear_str = if cfg.auto_clear.unwrap_or(false) {
        "Enabled 🟢".green().to_string()
    } else {
        "Disabled ⚪".dimmed().to_string()
    };
    let compact_mode_str = if cfg.compact_mode.unwrap_or(false) {
        "Enabled 🟢".green().to_string()
    } else {
        "Disabled ⚪".dimmed().to_string()
    };
    let ide_str = cfg
        .default_ide
        .clone()
        .unwrap_or_else(|| "ask (Prompt each time)".to_string());
    let hubs_count = format!(
        "{} directory hubs registered",
        cfg.custom_hubs.as_ref().map(|h| h.len()).unwrap_or(0)
    );
    let engine_str = cfg
        .search_engine
        .clone()
        .unwrap_or_else(|| "google".to_string());
    let config_file_path = config::config_path()
        .map(|p| {
            let s = p.to_string_lossy().to_string();
            if let Ok(home) = std::env::var("HOME")
                && s.starts_with(&home)
            {
                return format!("~{}", &s[home.len()..]);
            }
            s
        })
        .unwrap_or_default();

    ui::print_card(
        "Active Configuration",
        &[
            ("Primary Color", swatch),
            ("Default Editor", ide_str),
            ("Search Engine", engine_str),
            ("Auto Clear", auto_clear_str),
            ("Compact Mode", compact_mode_str),
            ("Custom Hubs", hubs_count),
            ("Config File", config_file_path),
        ],
    );

    let cancel_btn = cancel_option();
    let options = [
        "🎨 1. Change Primary Color (Palette or Custom HEX)",
        "🖥️  2. Set Default Project Editor / IDE",
        "🔍 3. Set Default Web Search Engine (Google, DuckDuckGo, Brave, etc.)",
        "🧹 4. Toggle Auto-Clear Terminal Screen",
        "🗜️  5. Toggle Compact Mode (Minimalist Banner & Layout)",
        "📁 6. Add Custom Workspace Project Hub",
        "📝 7. Open config.toml in Editor",
        "🔄 8. Reset Configuration to Factory Defaults",
        &cancel_btn,
    ];

    ui::print_key_hints();
    let selection = Select::with_theme(theme)
        .with_prompt("Select setting to configure")
        .items(&options)
        .default(0)
        .interact()?;

    match selection {
        0 => {
            // Color palette picker
            let cancel_c = cancel_option();
            let color_palette = [
                "🔵 Electric Blue   (#00a2ff - Signature Default)",
                "🟣 Neon Violet     (#8b5cf6)",
                "🟢 Emerald Green   (#10b981)",
                "🟠 Cyber Amber     (#f59e0b)",
                "🔴 Hot Rose        (#f43f5e)",
                "🐬 Aqua Cyan       (#06b6d4)",
                "✨ Custom HEX Code (#RRGGBB)...",
                &cancel_c,
            ];

            let col_sel = Select::with_theme(theme)
                .with_prompt("Choose primary theme accent color")
                .items(&color_palette)
                .default(0)
                .interact()?;

            let mut new_cfg = config::load_config();
            match col_sel {
                0 => new_cfg.primary_color = Some("electric-blue".to_string()),
                1 => new_cfg.primary_color = Some("violet".to_string()),
                2 => new_cfg.primary_color = Some("emerald".to_string()),
                3 => new_cfg.primary_color = Some("amber".to_string()),
                4 => new_cfg.primary_color = Some("rose".to_string()),
                5 => new_cfg.primary_color = Some("cyan".to_string()),
                6 => {
                    let hex_input: String = Input::with_theme(theme)
                        .with_prompt("Enter HEX color code (e.g. #ff007f or #10b981)")
                        .validate_with(|input: &String| -> Result<(), &str> {
                            let clean = input.trim().trim_start_matches('#');
                            if clean.len() == 6 && clean.chars().all(|c| c.is_ascii_hexdigit()) {
                                Ok(())
                            } else {
                                Err("Please enter a valid 6-character hex code, e.g. #ff007f")
                            }
                        })
                        .interact_text()?;
                    let formatted = if hex_input.starts_with('#') {
                        hex_input.to_lowercase()
                    } else {
                        format!("#{}", hex_input.to_lowercase())
                    };
                    new_cfg.primary_color = Some(formatted);
                }
                _ => {
                    println!("Cancelled.");
                    return Ok(());
                }
            }
            config::save_config(&new_cfg)?;
            let chosen = new_cfg.primary_color.unwrap();
            let (r, g, b) = config::parse_color(&chosen);
            println!(
                "{}",
                format!("Primary color updated to '{}'! 🎨", chosen)
                    .truecolor(r, g, b)
                    .bold()
            );
        }
        1 => {
            // Default IDE picker
            let cancel_i = cancel_option();
            let ide_choices = [
                "Antigravity IDE (antigravity)",
                "Cursor (cursor)",
                "Visual Studio Code (vscode)",
                "Xcode (xcode)",
                "Switch Terminal Directory Only (terminal)",
                "Ask every time (ask - default)",
                &cancel_i,
            ];

            let ide_sel = Select::with_theme(theme)
                .with_prompt("Select default project editor")
                .items(&ide_choices)
                .default(0)
                .interact()?;

            let mut new_cfg = config::load_config();
            match ide_sel {
                0 => new_cfg.default_ide = Some("antigravity".to_string()),
                1 => new_cfg.default_ide = Some("cursor".to_string()),
                2 => new_cfg.default_ide = Some("vscode".to_string()),
                3 => new_cfg.default_ide = Some("xcode".to_string()),
                4 => new_cfg.default_ide = Some("terminal".to_string()),
                5 => new_cfg.default_ide = Some("ask".to_string()),
                _ => {
                    println!("Cancelled.");
                    return Ok(());
                }
            }
            config::save_config(&new_cfg)?;
            println!(
                "{}",
                format!(
                    "Default editor updated to '{}'! 🖥️",
                    new_cfg.default_ide.unwrap()
                )
                .green()
                .bold()
            );
        }
        2 => {
            // Default Search Engine picker
            let cancel_e = cancel_option();
            let engine_choices = [
                "Google (google - default)",
                "DuckDuckGo (duckduckgo)",
                "Brave Search (brave)",
                "Perplexity AI (perplexity)",
                "Bing (bing)",
                "Kagi (kagi)",
                &cancel_e,
            ];

            let eng_sel = Select::with_theme(theme)
                .with_prompt("Select default web search engine")
                .items(&engine_choices)
                .default(0)
                .interact()?;

            let mut new_cfg = config::load_config();
            match eng_sel {
                0 => new_cfg.search_engine = Some("google".to_string()),
                1 => new_cfg.search_engine = Some("duckduckgo".to_string()),
                2 => new_cfg.search_engine = Some("brave".to_string()),
                3 => new_cfg.search_engine = Some("perplexity".to_string()),
                4 => new_cfg.search_engine = Some("bing".to_string()),
                5 => new_cfg.search_engine = Some("kagi".to_string()),
                _ => {
                    println!("Cancelled.");
                    return Ok(());
                }
            }
            config::save_config(&new_cfg)?;
            println!(
                "{}",
                format!(
                    "Default search engine updated to '{}'! 🔍",
                    new_cfg.search_engine.unwrap()
                )
                .green()
                .bold()
            );
        }
        3 => {
            // Toggle auto_clear
            let mut new_cfg = config::load_config();
            let current = new_cfg.auto_clear.unwrap_or(false);
            new_cfg.auto_clear = Some(!current);
            config::save_config(&new_cfg)?;
            if !current {
                println!("{}", "Auto-clear enabled! 🧹".green().bold());
            } else {
                println!("{}", "Auto-clear disabled. ⚪".yellow().bold());
            }
        }
        4 => {
            // Toggle compact_mode
            let mut new_cfg = config::load_config();
            let current = new_cfg.compact_mode.unwrap_or(false);
            new_cfg.compact_mode = Some(!current);
            config::save_config(&new_cfg)?;
            if !current {
                println!("{}", "Compact mode enabled! 🗜️".green().bold());
            } else {
                println!("{}", "Compact mode disabled. ⚪".yellow().bold());
            }
        }
        5 => {
            // Add custom hub
            let hub_input: String = Input::with_theme(theme)
                .with_prompt("Enter directory path to scan for projects (leave blank to cancel)")
                .allow_empty(true)
                .interact_text()?;

            let trimmed = hub_input.trim();
            if trimmed.is_empty() {
                println!("Cancelled.");
                return Ok(());
            }

            let mut new_cfg = config::load_config();
            let mut hubs = new_cfg.custom_hubs.unwrap_or_default();
            let p = PathBuf::from(trimmed);
            if !hubs.contains(&p) {
                hubs.push(p);
                new_cfg.custom_hubs = Some(hubs);
                config::save_config(&new_cfg)?;
                println!(
                    "{}",
                    format!("Added '{}' to project scan hubs! 📁", trimmed)
                        .green()
                        .bold()
                );
            } else {
                println!("Hub already registered in configuration.");
            }
        }
        6 => {
            let path = config::config_path()?;
            if !path.exists() {
                let _ = config::load_config();
            }
            let _ = Command::new("open").arg(&path).status();
            println!("Opened config file in editor: {}", path.display());
        }
        7 if Confirm::with_theme(theme)
            .with_prompt("Reset configuration to factory defaults?")
            .default(false)
            .interact()? =>
        {
            let path = config::config_path()?;
            if path.exists() {
                let _ = fs::remove_file(path);
            }
            let _ = config::load_config();
            println!(
                "{} Configuration reset to factory defaults.",
                "✔".green().bold()
            );
        }
        _ => {
            println!("Cancelled.");
        }
    }

    Ok(())
}

#[derive(Debug, Clone)]
struct PortProcess {
    command: String,
    pid: u32,
    user: String,
    port: u16,
    name: String,
}

fn scan_listening_ports() -> Vec<PortProcess> {
    let output = Command::new("lsof")
        .args(["-iTCP", "-sTCP:LISTEN", "-P", "-n"])
        .output();

    let Ok(out) = output else {
        return Vec::new();
    };

    let text = String::from_utf8_lossy(&out.stdout);
    let mut results = Vec::new();

    for line in text.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 9 {
            continue;
        }

        let cmd = parts[0].to_string();
        let Ok(pid) = parts[1].parse::<u32>() else {
            continue;
        };
        let user = parts[2].to_string();
        let name = parts[8].to_string();

        // Extract port from NAME (e.g., "*:8000" or "127.0.0.1:3000")
        if let Some(pos) = name.rfind(':') {
            let port_str = &name[pos + 1..];
            if let Ok(port) = port_str.parse::<u16>() {
                results.push(PortProcess {
                    command: cmd,
                    pid,
                    user,
                    port,
                    name,
                });
            }
        }
    }

    results.sort_by_key(|p| p.port);
    results.dedup_by_key(|p| (p.port, p.pid));
    results
}

fn kill_process_pid(pid: u32) -> Result<()> {
    // Try graceful SIGTERM first
    let _ = Command::new("kill").args(["-15", &pid.to_string()]).status();
    std::thread::sleep(std::time::Duration::from_millis(200));

    // Check if still running
    let check = Command::new("kill").args(["-0", &pid.to_string()]).status();
    if let Ok(st) = check
        && st.success()
    {
        // Force SIGKILL
        let _ = Command::new("kill").args(["-9", &pid.to_string()]).status();
    }
    Ok(())
}

/// Inspects active listening ports and terminates conflicting processes
pub fn handle_port(theme: &ColorfulTheme, port: Option<&str>, kill: bool) -> Result<()> {
    ui::maybe_auto_clear();
    let ports = scan_listening_ports();

    let target_port: Option<u16> = match port {
        Some(p) => {
            let clean = p.trim_start_matches(':');
            match clean.parse::<u16>() {
                Ok(val) => Some(val),
                Err(_) => bail!("invalid port number '{p}'. Must be between 1 and 65535."),
            }
        }
        None => None,
    };

    if let Some(target_port) = target_port {
        let matches: Vec<&PortProcess> = ports.iter().filter(|p| p.port == target_port).collect();

        if matches.is_empty() {
            println!(
                "{} No listening process detected on port {}.",
                "●".blue(),
                target_port.to_string().bold()
            );
            return Ok(());
        }

        ui::print_banner();
        ui::render_breadcrumbs(&["run", "Port Inspector"]);

        for proc in &matches {
            ui::print_card(
                &format!("Port {} Active Listener", proc.port),
                &[
                    ("Port", proc.port.to_string()),
                    ("Process", proc.command.clone()),
                    ("PID", proc.pid.to_string()),
                    ("User", proc.user.clone()),
                    ("Address", proc.name.clone()),
                ],
            );

            if kill {
                kill_process_pid(proc.pid)?;
                println!(
                    "{} Terminated process {} (PID: {}) on port {}.",
                    "✔".green().bold(),
                    proc.command.bold(),
                    proc.pid,
                    proc.port
                );
            } else if std::io::stdin().is_terminal() {
                let should_kill = Confirm::with_theme(theme)
                    .with_prompt(format!(
                        "Terminate process '{}' (PID: {}) on port {}?",
                        proc.command, proc.pid, proc.port
                    ))
                    .default(false)
                    .interact()?;

                if should_kill {
                    kill_process_pid(proc.pid)?;
                    println!(
                        "{} Terminated process {} (PID: {}) on port {}.",
                        "✔".green().bold(),
                        proc.command.bold(),
                        proc.pid,
                        proc.port
                    );
                } else {
                    println!("Process left running.");
                }
            }
        }
        return Ok(());
    }

    // No port argument: show all active listening ports
    ui::print_banner();
    ui::render_breadcrumbs(&["run", "Active Listening Ports"]);

    if ports.is_empty() {
        println!("No active listening TCP ports detected.");
        return Ok(());
    }

    if !std::io::stdin().is_terminal() {
        println!("{:<8} {:<8} {:<18} {:<12} ADDRESS", "PORT", "PID", "PROCESS", "USER");
        println!("{}", "─".repeat(60));
        for p in &ports {
            println!(
                "{:<8} {:<8} {:<18} {:<12} {}",
                p.port, p.pid, p.command, p.user, p.name
            );
        }
        return Ok(());
    }

    let cancel_btn = cancel_option();
    let mut menu_items: Vec<String> = ports
        .iter()
        .map(|p| {
            format!(
                "Port {:<6} ➔  {:<16} (PID: {:<6} User: {})",
                p.port.to_string().bold(),
                p.command.cyan(),
                p.pid,
                p.user.dimmed()
            )
        })
        .collect();
    menu_items.push(cancel_btn);

    ui::print_key_hints();
    let selection = Select::with_theme(theme)
        .with_prompt("Select a port to inspect or terminate")
        .items(&menu_items)
        .default(0)
        .interact()?;

    if selection >= ports.len() {
        println!("Cancelled.");
        return Ok(());
    }

    let selected = &ports[selection];
    ui::print_card(
        &format!("Port {} Details", selected.port),
        &[
            ("Port", selected.port.to_string()),
            ("Process", selected.command.clone()),
            ("PID", selected.pid.to_string()),
            ("User", selected.user.clone()),
            ("Address", selected.name.clone()),
        ],
    );

    let actions = ["Kill / Terminate Process", "Cancel"];
    let act_sel = Select::with_theme(theme)
        .with_prompt(format!("Action for PID {} ({})", selected.pid, selected.command))
        .items(&actions)
        .default(0)
        .interact()?;

    if act_sel == 0 {
        kill_process_pid(selected.pid)?;
        println!(
            "{} Successfully killed process {} (PID: {}) on port {}! 🧹",
            "✔".green().bold(),
            selected.command.bold(),
            selected.pid,
            selected.port
        );
    } else {
        println!("Cancelled.");
    }

    Ok(())
}

/// Manages custom developer aliases and shortcuts
pub fn handle_alias(
    theme: &ColorfulTheme,
    action: Option<&str>,
    name: Option<&str>,
    target: Option<&str>,
) -> Result<()> {
    match action {
        Some("list") | Some("ls") => {
            let aliases = config::get_aliases();
            if aliases.is_empty() {
                println!("No custom aliases registered yet. Add one with: run alias add <name> \"<command>\"");
                return Ok(());
            }
            ui::print_banner();
            ui::render_breadcrumbs(&["run", "Custom Aliases"]);
            let rows: Vec<(&str, String)> = aliases
                .iter()
                .map(|(k, v)| (k.as_str(), v.clone()))
                .collect();
            ui::print_card("Configured Developer Aliases", &rows);
        }
        Some("add") | Some("set") => {
            let a_name = name.context("missing alias name (e.g. 'run alias add c \"cargo check\"')")?;
            let a_target = target.context("missing target command (e.g. 'run alias add c \"cargo check\"')")?;
            config::set_alias(a_name, a_target)?;
            println!(
                "{} Registered alias: {} ➔ '{}'",
                "✔".green().bold(),
                a_name.bold(),
                a_target.cyan()
            );
        }
        Some("remove") | Some("rm") | Some("del") => {
            let a_name = name.context("missing alias name (e.g. 'run alias rm c')")?;
            if config::remove_alias(a_name)? {
                println!("{} Removed alias '{}'", "✔".green().bold(), a_name);
            } else {
                println!("Alias '{}' not found.", a_name);
            }
        }
        None => {
            // Interactive management
            ui::maybe_auto_clear();
            ui::print_banner();
            ui::render_breadcrumbs(&["run", "Aliases"]);

            let aliases = config::get_aliases();
            let mut card_rows = Vec::new();
            for (k, v) in &aliases {
                card_rows.push((k.as_str(), v.clone()));
            }
            if !card_rows.is_empty() {
                ui::print_card("Active Aliases", &card_rows);
            }

            let cancel_btn = cancel_option();
            let options = [
                "➕ 1. Add New Command Alias",
                "🗑️  2. Remove Existing Alias",
                "📋 3. View All Aliases",
                &cancel_btn,
            ];

            let sel = Select::with_theme(theme)
                .with_prompt("Select alias action")
                .items(&options)
                .default(0)
                .interact()?;

            match sel {
                0 => {
                    let a_name: String = Input::with_theme(theme)
                        .with_prompt("Alias name / trigger (e.g. 'c' or 'devs')")
                        .interact_text()?;
                    let a_target: String = Input::with_theme(theme)
                        .with_prompt(format!("Target shell command for 'run {}'", a_name))
                        .interact_text()?;

                    config::set_alias(&a_name, &a_target)?;
                    println!(
                        "{} Registered alias: {} ➔ '{}'",
                        "✔".green().bold(),
                        a_name.bold(),
                        a_target.cyan()
                    );
                }
                1 => {
                    if aliases.is_empty() {
                        println!("No aliases registered to remove.");
                        return Ok(());
                    }
                    let cancel_rm = cancel_option();
                    let mut keys: Vec<String> = aliases
                        .iter()
                        .map(|(k, v)| format!("{:<12} ➔  {}", k.bold(), v.dimmed()))
                        .collect();
                    keys.push(cancel_rm);

                    let rm_sel = Select::with_theme(theme)
                        .with_prompt("Select alias to remove")
                        .items(&keys)
                        .default(0)
                        .interact()?;

                    if rm_sel < aliases.len() {
                        let key_to_remove = aliases.keys().nth(rm_sel).unwrap();
                        config::remove_alias(key_to_remove)?;
                        println!("{} Removed alias '{}'", "✔".green().bold(), key_to_remove);
                    } else {
                        println!("Cancelled.");
                    }
                }
                2 => {
                    if aliases.is_empty() {
                        println!("No aliases configured yet.");
                    }
                }
                _ => {
                    println!("Cancelled.");
                }
            }
        }
        Some(other) => {
            bail!("unknown alias action '{other}'. Usage: run alias [list|add|rm]");
        }
    }
    Ok(())
}

/// Displays terminal command execution analytics and productivity stats
pub fn handle_stats(_theme: &ColorfulTheme) -> Result<()> {
    ui::maybe_auto_clear();
    ui::print_banner();
    ui::render_breadcrumbs(&["run", "Usage Analytics"]);

    let stats = config::load_command_stats();
    let total_runs = stats.total_runs.to_string();
    let first_used = stats.first_used.unwrap_or_else(|| "Just now".to_string());
    let last_used = stats.last_used.unwrap_or_else(|| "Just now".to_string());
    let unique_count = stats.command_counts.len().to_string();

    ui::print_card(
        "CLI Productivity Snapshot",
        &[
            ("Total Commands Run", total_runs),
            ("Unique Commands", unique_count),
            ("First Tracked Run", first_used),
            ("Last Run Timestamp", last_used),
        ],
    );

    if stats.command_counts.is_empty() {
        println!("No command stats recorded yet. Run commands to see analytics!");
        return Ok(());
    }

    // Sort commands by count descending
    let mut sorted_counts: Vec<(&String, &u64)> = stats.command_counts.iter().collect();
    sorted_counts.sort_by(|a, b| b.1.cmp(a.1));

    let max_val = *sorted_counts.first().map(|(_, c)| *c).unwrap_or(&1);

    println!("{}", "Top Command Usage Frequency:".bold());
    println!();

    for (rank, (cmd, count)) in sorted_counts.iter().take(8).enumerate() {
        let bar_len = if max_val > 0 {
            ((**count as f64 / max_val as f64) * 20.0).round() as usize
        } else {
            1
        };
        let bar_filled = "■".repeat(bar_len);
        let bar_empty = "□".repeat(20 - bar_len.min(20));
        let bar = format!("{}{}", bar_filled.green(), bar_empty.dimmed());

        println!(
            "  {}. {:<14} {} {:>4} runs",
            rank + 1,
            ui::primary_colored(cmd).bold(),
            bar,
            count
        );
    }
    println!();

    Ok(())
}

// ============================================================================
// Timer & Pomodoro (`run timer`, `run pomo`)
// ============================================================================

pub fn handle_timer(theme: &ColorfulTheme, minutes: Option<u64>) -> Result<()> {
    let total_secs = match minutes {
        Some(m) if m > 0 => m * 60,
        _ => {
            if !std::io::stdin().is_terminal() {
                25 * 60
            } else {
                ui::maybe_auto_clear();
                ui::print_banner();
                ui::render_breadcrumbs(&["run", "Focus & Pomodoro Timer"]);

                let cancel_btn = ui::cancel_option();
                let options = [
                    "🍅 1. 25 Minutes (Standard Pomodoro)",
                    "☕ 2. 5 Minutes (Short Break)",
                    "🛋️  3. 15 Minutes (Long Break)",
                    "⚡ 4. 45 Minutes (Deep Work Session)",
                    "🎯 5. 60 Minutes (Power Hour)",
                    "⏱️  6. Custom Minutes...",
                    &cancel_btn,
                ];

                let sel = Select::with_theme(theme)
                    .with_prompt("Select timer duration")
                    .items(&options)
                    .default(0)
                    .interact()?;

                if sel >= options.len() - 1 {
                    println!("Cancelled.");
                    return Ok(());
                }

                match sel {
                    0 => 25 * 60,
                    1 => 5 * 60,
                    2 => 15 * 60,
                    3 => 45 * 60,
                    4 => 60 * 60,
                    5 => {
                        let mins: u64 = Input::with_theme(theme)
                            .with_prompt("Enter duration in minutes")
                            .default(30)
                            .interact_text()?;
                        mins * 60
                    }
                    _ => return Ok(()),
                }
            }
        }
    };

    println!();
    println!("{} Timer started for {} minutes. Press Ctrl+C to stop.", "⏱️".bold(), total_secs / 60);
    println!();

    let start = std::time::Instant::now();
    let total_dur = std::time::Duration::from_secs(total_secs);

    while start.elapsed() < total_dur {
        let elapsed = start.elapsed();
        let remaining = total_dur.saturating_sub(elapsed);
        let rem_secs = remaining.as_secs();
        let rem_min = rem_secs / 60;
        let rem_sec = rem_secs % 60;

        let pct = (elapsed.as_secs_f64() / total_dur.as_secs_f64()).clamp(0.0, 1.0);
        let bar_width: usize = 25;
        let filled = (pct * bar_width as f64).round() as usize;
        let empty = bar_width.saturating_sub(filled);
        let bar = format!("{}{}", "■".repeat(filled).green(), "□".repeat(empty).dimmed());

        print!("\r  ⏳ [{bar}] {:02}:{:02} remaining ({:.0}%)   ", rem_min, rem_sec, pct * 100.0);
        let _ = std::io::stdout().flush();

        std::thread::sleep(std::time::Duration::from_millis(500));
    }

    println!("\n");
    println!("{} Time is up! Great work! 🎉", "✔".green().bold());

    // Native macOS chime + banner notification
    let _ = crate::mac::handle_notify(theme, Some("Timer Finished! ⏰"), Some("Your focus session has completed."));
    Ok(())
}

// ============================================================================
// UUID Generator (`run uuid`, `run uid`)
// ============================================================================

pub fn handle_uuid() -> Result<()> {
    let output = Command::new("/usr/bin/uuidgen")
        .output()
        .context("failed to execute uuidgen")?;
    let raw = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let uuid_lower = raw.to_lowercase();

    crate::mac::copy_to_clipboard(&uuid_lower)?;

    ui::print_banner();
    ui::render_breadcrumbs(&["run", "UUID Generator"]);
    ui::print_card(
        "Generated UUID v4",
        &[
            ("UUID (Lowercase)", uuid_lower.green().bold().to_string()),
            ("UUID (Uppercase)", raw),
            ("Clipboard", "Copied to clipboard automatically 📋".cyan().to_string()),
        ],
    );
    Ok(())
}

// ============================================================================
// Secure Password / Token Generator (`run pass`, `run pwd`)
// ============================================================================

pub fn handle_pass(theme: &ColorfulTheme, length: Option<usize>) -> Result<()> {
    let len = match length {
        Some(l) if l >= 4 => l,
        _ => {
            if !std::io::stdin().is_terminal() {
                20
            } else {
                ui::maybe_auto_clear();
                ui::print_banner();
                ui::render_breadcrumbs(&["run", "Password Generator"]);

                let cancel_btn = ui::cancel_option();
                let options = [
                    "🔐 1. Standard Strong Password (16 chars)",
                    "🛡️  2. Extra Secure Password (24 chars)",
                    "🗝️  3. High Entropy API Token / Secret (32 chars)",
                    "⚡ 4. Alphanumeric Only (No special symbols, 20 chars)",
                    "🔢 5. Custom Length...",
                    &cancel_btn,
                ];

                let sel = Select::with_theme(theme)
                    .with_prompt("Select password type")
                    .items(&options)
                    .default(0)
                    .interact()?;

                if sel >= options.len() - 1 {
                    println!("Cancelled.");
                    return Ok(());
                }

                match sel {
                    0 => 16,
                    1 => 24,
                    2 => 32,
                    3 => 20,
                    4 => {
                        let l: usize = Input::with_theme(theme)
                            .with_prompt("Enter desired length (8-128)")
                            .default(20)
                            .interact_text()?;
                        l.clamp(4, 128)
                    }
                    _ => return Ok(()),
                }
            }
        }
    };

    let pass = generate_secure_password(len);
    crate::mac::copy_to_clipboard(&pass)?;

    ui::print_banner();
    ui::render_breadcrumbs(&["run", "Password Generator"]);
    ui::print_card(
        "Generated Secure Password",
        &[
            ("Password", pass.green().bold().to_string()),
            ("Length", len.to_string()),
            ("Entropy / Strength", "Very Strong 🔒".green().to_string()),
            ("Clipboard", "Copied to clipboard automatically 📋".cyan().to_string()),
        ],
    );
    Ok(())
}

fn generate_secure_password(len: usize) -> String {
    use std::fs::File;
    use std::io::Read;

    const CHARSET: &[u8] = b"abcdefghjkmnpqrstuvwxyzABCDEFGHJKLMNPQRSTUVWXYZ23456789!@#$%&*+=-_";
    let mut random_bytes = vec![0u8; len];
    if let Ok(mut f) = File::open("/dev/urandom") {
        let _ = f.read_exact(&mut random_bytes);
    } else {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(42);
        for (i, b) in random_bytes.iter_mut().enumerate() {
            *b = ((nanos >> (i % 16)) ^ (i as u128 * 73)) as u8;
        }
    }

    random_bytes
        .into_iter()
        .map(|b| CHARSET[(b as usize) % CHARSET.len()] as char)
        .collect()
}

// ============================================================================
// Terminal QR Code Generator (`run qr`, `run qrc`)
// ============================================================================

pub fn handle_qr(theme: &ColorfulTheme, content: Option<&str>) -> Result<()> {
    let text = match content {
        Some(c) if !c.trim().is_empty() => c.to_string(),
        _ => {
            let clip = crate::mac::read_from_clipboard().unwrap_or_default();
            let clip_trimmed = clip.trim().to_string();

            if !clip_trimmed.is_empty() && (clip_trimmed.starts_with("http://") || clip_trimmed.starts_with("https://")) {
                println!("{} Using URL from clipboard: {}", "📋".bold(), clip_trimmed.cyan());
                clip_trimmed
            } else if !std::io::stdin().is_terminal() {
                if !clip_trimmed.is_empty() {
                    clip_trimmed
                } else {
                    bail!("provide text or URL to generate QR code: run qr <text>");
                }
            } else {
                ui::maybe_auto_clear();
                ui::print_banner();
                ui::render_breadcrumbs(&["run", "Terminal QR Generator"]);

                let default_val = if !clip_trimmed.is_empty() {
                    clip_trimmed
                } else {
                    "https://github.com/naenmad/run-cli".to_string()
                };
                Input::with_theme(theme)
                    .with_prompt("Enter text or URL for QR Code")
                    .default(default_val)
                    .interact_text()?
            }
        }
    };

    use qrcode::QrCode;
    use qrcode::render::unicode;

    let code = QrCode::new(text.as_bytes()).context("failed to encode text into QR code")?;
    let image = code
        .render::<unicode::Dense1x2>()
        .dark_color(unicode::Dense1x2::Light)
        .light_color(unicode::Dense1x2::Dark)
        .build();

    println!();
    println!("{}", image);
    println!("  📱 Content: {}", text.cyan().bold());
    println!("  Scan with phone camera or QR reader.");
    println!();
    Ok(())
}

// ============================================================================
// System & Toolchain Updater (`run update`, `run upd`)
// ============================================================================

pub fn handle_update(_theme: &ColorfulTheme) -> Result<()> {
    ui::maybe_auto_clear();
    ui::print_banner();
    ui::render_breadcrumbs(&["run", "System & Toolchain Updater"]);

    println!("{}", "Scanning installed developer toolchains...".dimmed());
    println!();

    // 1. Homebrew
    let has_brew = Command::new("which").arg("brew").output().map(|o| o.status.success()).unwrap_or(false);
    if has_brew {
        println!("{} Updating Homebrew formulas and casks...", "🍺".bold());
        let _ = Command::new("brew").arg("update").status();
        println!("{} Homebrew updated.", "✔".green().bold());
        println!();
    }

    // 2. Rustup
    let has_rustup = Command::new("which").arg("rustup").output().map(|o| o.status.success()).unwrap_or(false);
    if has_rustup {
        println!("{} Updating Rust toolchain...", "🦀".bold());
        let _ = Command::new("rustup").arg("update").status();
        println!("{} Rust toolchain updated.", "✔".green().bold());
        println!();
    }

    // 3. Node package managers (npm / pnpm)
    let has_pnpm = Command::new("which").arg("pnpm").output().map(|o| o.status.success()).unwrap_or(false);
    let has_npm = Command::new("which").arg("npm").output().map(|o| o.status.success()).unwrap_or(false);

    if has_pnpm {
        println!("{} Updating global pnpm packages...", "📦".bold());
        let _ = Command::new("pnpm").args(["update", "-g"]).status();
        println!("{} pnpm global packages updated.", "✔".green().bold());
        println!();
    } else if has_npm {
        println!("{} Updating global npm packages...", "📦".bold());
        let _ = Command::new("npm").args(["update", "-g"]).status();
        println!("{} npm global packages updated.", "✔".green().bold());
        println!();
    }

    println!("{} All detected developer toolchains are up to date! 🚀", "✔".green().bold());
    Ok(())
}

// ============================================================================
// Multi-Stack Dependency Installer & Package Adder (`run install`, `run ins`)
// ============================================================================

#[derive(Debug, Clone)]
enum ProjectStack {
    Node { runner: String, lockfile: Option<String> },
    Python { runner: String, manifest: String, has_venv: bool },
    Rust,
    Flutter { runner: String },
    Go,
    Php,
    Ruby,
}

impl ProjectStack {
    fn name(&self) -> &str {
        match self {
            ProjectStack::Node { .. } => "Node.js",
            ProjectStack::Python { .. } => "Python",
            ProjectStack::Rust => "Rust",
            ProjectStack::Flutter { .. } => "Flutter / Dart",
            ProjectStack::Go => "Go",
            ProjectStack::Php => "PHP",
            ProjectStack::Ruby => "Ruby",
        }
    }

    fn detail(&self) -> String {
        match self {
            ProjectStack::Node { runner, lockfile } => {
                if let Some(lock) = lockfile {
                    format!("{runner} ({lock})")
                } else {
                    format!("{runner} (package.json)")
                }
            }
            ProjectStack::Python { runner, manifest, has_venv } => {
                let venv_status = if *has_venv { "in .venv" } else { "no venv" };
                format!("{runner} ({manifest}, {venv_status})")
            }
            ProjectStack::Rust => "cargo (Cargo.toml)".to_string(),
            ProjectStack::Flutter { runner } => format!("{runner} (pubspec.yaml)"),
            ProjectStack::Go => "go (go.mod)".to_string(),
            ProjectStack::Php => "composer (composer.json)".to_string(),
            ProjectStack::Ruby => "bundle (Gemfile)".to_string(),
        }
    }
}

fn detect_project_stacks(dir: &std::path::Path) -> Vec<ProjectStack> {
    let mut stacks = Vec::new();

    // 1. Node.js
    if dir.join("package.json").exists() {
        let (runner, lockfile) = if dir.join("pnpm-lock.yaml").exists() {
            ("pnpm".to_string(), Some("pnpm-lock.yaml".to_string()))
        } else if dir.join("yarn.lock").exists() {
            ("yarn".to_string(), Some("yarn.lock".to_string()))
        } else if dir.join("bun.lockb").exists() || dir.join("bun.lock").exists() {
            ("bun".to_string(), Some("bun.lock".to_string()))
        } else if dir.join("package-lock.json").exists() {
            ("npm".to_string(), Some("package-lock.json".to_string()))
        } else {
            ("npm".to_string(), None)
        };
        stacks.push(ProjectStack::Node { runner, lockfile });
    }

    // 2. Python
    let py_has_poetry = dir.join("poetry.lock").exists()
        || (dir.join("pyproject.toml").exists()
            && fs::read_to_string(dir.join("pyproject.toml"))
                .map(|s| s.contains("[tool.poetry]"))
                .unwrap_or(false));
    let py_has_pipenv = dir.join("Pipfile").exists();
    let py_has_reqs = dir.join("requirements.txt").exists() || dir.join("requirements-dev.txt").exists();
    let py_has_pyproject = dir.join("pyproject.toml").exists();
    let py_has_venv = dir.join(".venv").exists() || dir.join("venv").exists();

    if py_has_poetry {
        stacks.push(ProjectStack::Python {
            runner: "poetry".to_string(),
            manifest: "poetry.lock".to_string(),
            has_venv: py_has_venv,
        });
    } else if py_has_pipenv {
        stacks.push(ProjectStack::Python {
            runner: "pipenv".to_string(),
            manifest: "Pipfile".to_string(),
            has_venv: py_has_venv,
        });
    } else if py_has_reqs || py_has_pyproject {
        let manifest = if py_has_reqs { "requirements.txt" } else { "pyproject.toml" };
        let has_uv = Command::new("which").arg("uv").output().map(|o| o.status.success()).unwrap_or(false);
        let runner = if has_uv { "uv" } else { "pip" }.to_string();
        stacks.push(ProjectStack::Python {
            runner,
            manifest: manifest.to_string(),
            has_venv: py_has_venv,
        });
    }

    // 3. Rust
    if dir.join("Cargo.toml").exists() {
        stacks.push(ProjectStack::Rust);
    }

    // 4. Flutter / Dart
    if dir.join("pubspec.yaml").exists() {
        let has_flutter = Command::new("which").arg("flutter").output().map(|o| o.status.success()).unwrap_or(false);
        let runner = if has_flutter { "flutter" } else { "dart" }.to_string();
        stacks.push(ProjectStack::Flutter { runner });
    }

    // 5. Go
    if dir.join("go.mod").exists() {
        stacks.push(ProjectStack::Go);
    }

    // 6. PHP
    if dir.join("composer.json").exists() {
        stacks.push(ProjectStack::Php);
    }

    // 7. Ruby
    if dir.join("Gemfile").exists() {
        stacks.push(ProjectStack::Ruby);
    }

    stacks
}

fn execute_stack_install(theme: &ColorfulTheme, stack: &ProjectStack, dir: &std::path::Path) -> Result<()> {
    match stack {
        ProjectStack::Node { runner, lockfile } => {
            let desc = lockfile.as_deref().unwrap_or("package.json");
            println!("{} Installing Node.js dependencies using {} ({desc})...", "📦".bold(), runner.cyan().bold());
            let status = Command::new(runner).arg("install").current_dir(dir).status()
                .with_context(|| format!("failed to execute '{runner} install'"))?;
            if !status.success() {
                bail!("'{runner} install' exited with non-zero code");
            }
            println!("{} Node.js dependencies installed successfully!", "✔".green().bold());
        }
        ProjectStack::Python { runner, manifest, has_venv } => {
            let mut venv_created = *has_venv;
            if !*has_venv && runner != "poetry" && runner != "pipenv" && std::io::stdin().is_terminal() {
                let confirm = Confirm::with_theme(theme)
                    .with_prompt("No Python virtual environment (.venv) found. Create one now?")
                    .default(true)
                    .interact()?;
                if confirm {
                    println!("{} Creating virtual environment in .venv...", "🐍".bold());
                    let _ = Command::new("python3").args(["-m", "venv", ".venv"]).current_dir(dir).status();
                    println!("{} Created .venv.", "✔".green().bold());
                    venv_created = true;
                }
            }

            println!("{} Installing Python dependencies using {} ({manifest})...", "🐍".bold(), runner.cyan().bold());

            let status = if runner == "poetry" {
                Command::new("poetry").arg("install").current_dir(dir).status()?
            } else if runner == "pipenv" {
                Command::new("pipenv").arg("install").current_dir(dir).status()?
            } else if runner == "uv" {
                if dir.join("requirements.txt").exists() {
                    if venv_created && dir.join(".venv/bin/python").exists() {
                        Command::new("uv").args(["pip", "install", "-r", "requirements.txt", "--python", ".venv/bin/python"]).current_dir(dir).status()?
                    } else {
                        Command::new("uv").args(["pip", "install", "-r", "requirements.txt"]).current_dir(dir).status()?
                    }
                } else {
                    Command::new("uv").arg("sync").current_dir(dir).status()?
                }
            } else {
                let pip_bin = if venv_created && dir.join(".venv/bin/pip").exists() {
                    ".venv/bin/pip"
                } else if dir.join("venv/bin/pip").exists() {
                    "venv/bin/pip"
                } else {
                    "pip3"
                };
                if dir.join("requirements.txt").exists() {
                    Command::new(pip_bin).args(["install", "-r", "requirements.txt"]).current_dir(dir).status()?
                } else if dir.join("requirements-dev.txt").exists() {
                    Command::new(pip_bin).args(["install", "-r", "requirements-dev.txt"]).current_dir(dir).status()?
                } else {
                    Command::new(pip_bin).args(["install", "-e", "."]).current_dir(dir).status()?
                }
            };

            if !status.success() {
                bail!("Python dependency installation exited with error");
            }
            println!("{} Python dependencies installed successfully!", "✔".green().bold());
        }
        ProjectStack::Rust => {
            println!("{} Fetching and checking Rust crate dependencies...", "🦀".bold());
            let status = Command::new("cargo").arg("check").current_dir(dir).status()?;
            if !status.success() {
                bail!("'cargo check' exited with non-zero code");
            }
            println!("{} Rust crate dependencies resolved and checked!", "✔".green().bold());
        }
        ProjectStack::Flutter { runner } => {
            println!("{} Getting Flutter / Dart packages...", "📱".bold());
            let status = Command::new(runner).args(["pub", "get"]).current_dir(dir).status()?;
            if !status.success() {
                bail!("'{runner} pub get' exited with non-zero code");
            }
            println!("{} Flutter / Dart packages downloaded successfully!", "✔".green().bold());
        }
        ProjectStack::Go => {
            println!("{} Downloading and tidying Go modules...", "🐹".bold());
            let _ = Command::new("go").args(["mod", "download"]).current_dir(dir).status();
            let status = Command::new("go").args(["mod", "tidy"]).current_dir(dir).status()?;
            if !status.success() {
                bail!("'go mod tidy' exited with non-zero code");
            }
            println!("{} Go modules downloaded and tidied!", "✔".green().bold());
        }
        ProjectStack::Php => {
            println!("{} Installing Composer dependencies...", "🐘".bold());
            let status = Command::new("composer").arg("install").current_dir(dir).status()?;
            if !status.success() {
                bail!("'composer install' exited with non-zero code");
            }
            println!("{} Composer packages installed!", "✔".green().bold());
        }
        ProjectStack::Ruby => {
            println!("{} Installing Bundler gems...", "💎".bold());
            let status = Command::new("bundle").arg("install").current_dir(dir).status()?;
            if !status.success() {
                bail!("'bundle install' exited with non-zero code");
            }
            println!("{} Ruby gems installed!", "✔".green().bold());
        }
    }
    Ok(())
}

fn execute_stack_add(stack: &ProjectStack, pkg: &str, is_dev: bool, dir: &std::path::Path) -> Result<()> {
    match stack {
        ProjectStack::Node { runner, .. } => {
            println!("{} Adding '{pkg}' via {runner}...", "📦".bold(), runner = runner.cyan().bold());
            let mut args = vec!["add"];
            if is_dev {
                if runner == "bun" {
                    args.push("-d");
                } else if runner == "npm" {
                    args[0] = "install";
                    args.push("--save-dev");
                } else {
                    args.push("-D");
                }
            }
            args.push(pkg);
            let status = Command::new(runner).args(&args).current_dir(dir).status()?;
            if !status.success() {
                bail!("failed to add package '{pkg}' using {runner}");
            }
            println!("{} Successfully added '{pkg}' to Node dependencies!", "✔".green().bold());
        }
        ProjectStack::Python { runner, has_venv, .. } => {
            println!("{} Adding '{pkg}' via Python package manager...", "🐍".bold());
            let status = if runner == "poetry" {
                let mut args = vec!["add"];
                if is_dev {
                    args.push("--group");
                    args.push("dev");
                }
                args.push(pkg);
                Command::new("poetry").args(&args).current_dir(dir).status()?
            } else if runner == "pipenv" {
                let mut args = vec!["install"];
                if is_dev {
                    args.push("--dev");
                }
                args.push(pkg);
                Command::new("pipenv").args(&args).current_dir(dir).status()?
            } else if runner == "uv" {
                let mut args = vec!["add"];
                if is_dev {
                    args.push("--dev");
                }
                args.push(pkg);
                Command::new("uv").args(&args).current_dir(dir).status()?
            } else {
                let pip_bin = if *has_venv && dir.join(".venv/bin/pip").exists() {
                    ".venv/bin/pip"
                } else if dir.join("venv/bin/pip").exists() {
                    "venv/bin/pip"
                } else {
                    "pip3"
                };
                Command::new(pip_bin).args(["install", pkg]).current_dir(dir).status()?
            };

            if !status.success() {
                bail!("failed to add Python package '{pkg}'");
            }
            println!("{} Successfully added '{pkg}' to Python dependencies!", "✔".green().bold());
        }
        ProjectStack::Rust => {
            println!("{} Adding '{pkg}' via cargo...", "🦀".bold());
            let mut args = vec!["add"];
            if is_dev {
                args.push("--dev");
            }
            args.push(pkg);
            let status = Command::new("cargo").args(&args).current_dir(dir).status()?;
            if !status.success() {
                bail!("'cargo add {pkg}' failed");
            }
            println!("{} Successfully added '{pkg}' to Cargo.toml!", "✔".green().bold());
        }
        ProjectStack::Flutter { runner } => {
            println!("{} Adding '{pkg}' via {runner}...", "📱".bold());
            let mut args = vec!["pub", "add"];
            if is_dev {
                args.push("--dev");
            }
            args.push(pkg);
            let status = Command::new(runner).args(&args).current_dir(dir).status()?;
            if !status.success() {
                bail!("'{runner} pub add {pkg}' failed");
            }
            println!("{} Successfully added '{pkg}' to pubspec.yaml!", "✔".green().bold());
        }
        ProjectStack::Go => {
            println!("{} Adding '{pkg}' via go get...", "🐹".bold());
            let status = Command::new("go").args(["get", pkg]).current_dir(dir).status()?;
            if !status.success() {
                bail!("'go get {pkg}' failed");
            }
            let _ = Command::new("go").args(["mod", "tidy"]).current_dir(dir).status();
            println!("{} Successfully added '{pkg}' to go.mod!", "✔".green().bold());
        }
        ProjectStack::Php => {
            println!("{} Adding '{pkg}' via composer require...", "🐘".bold());
            let mut args = vec!["require"];
            if is_dev {
                args.push("--dev");
            }
            args.push(pkg);
            let status = Command::new("composer").args(&args).current_dir(dir).status()?;
            if !status.success() {
                bail!("'composer require {pkg}' failed");
            }
            println!("{} Successfully added '{pkg}' to composer.json!", "✔".green().bold());
        }
        ProjectStack::Ruby => {
            println!("{} Adding '{pkg}' via bundle add...", "💎".bold());
            let mut args = vec!["add"];
            if is_dev {
                args.push("--group");
                args.push("development");
            }
            args.push(pkg);
            let status = Command::new("bundle").args(&args).current_dir(dir).status()?;
            if !status.success() {
                bail!("'bundle add {pkg}' failed");
            }
            println!("{} Successfully added '{pkg}' to Gemfile!", "✔".green().bold());
        }
    }
    Ok(())
}

pub fn handle_install(
    theme: &ColorfulTheme,
    package: Option<&str>,
    is_dev: bool,
) -> Result<()> {
    ui::maybe_auto_clear();
    let current_dir = std::env::current_dir().context("failed to read current working directory")?;
    let stacks = detect_project_stacks(&current_dir);

    if stacks.is_empty() {
        bail!("no recognized project configuration found in current directory (e.g. package.json, requirements.txt, Cargo.toml, pubspec.yaml, go.mod)");
    }

    if let Some(pkg) = package {
        let target_stack = if stacks.len() == 1 {
            &stacks[0]
        } else if std::io::stdin().is_terminal() {
            let cancel_btn = ui::cancel_option();
            let mut options: Vec<String> = stacks.iter().map(|s| format!("{:<14} ➔  {}", s.name().bold(), s.detail().dimmed())).collect();
            options.push(cancel_btn);

            let sel = Select::with_theme(theme)
                .with_prompt(format!("Add '{pkg}' to which stack?"))
                .items(&options)
                .default(0)
                .interact()?;

            if sel >= stacks.len() {
                println!("Cancelled.");
                return Ok(());
            }
            &stacks[sel]
        } else {
            &stacks[0]
        };

        return execute_stack_add(target_stack, pkg, is_dev, &current_dir);
    }

    if stacks.len() == 1 {
        return execute_stack_install(theme, &stacks[0], &current_dir);
    }

    if std::io::stdin().is_terminal() {
        ui::print_banner();
        ui::render_breadcrumbs(&["run", "Dependency Installer"]);

        let cancel_btn = ui::cancel_option();
        let mut options: Vec<String> = stacks
            .iter()
            .map(|s| format!("{:<14} ➔  {}", s.name().bold(), s.detail().dimmed()))
            .collect();
        options.push("🚀 Install All Detected Stacks".bold().to_string());
        options.push(cancel_btn);

        let sel = Select::with_theme(theme)
            .with_prompt("Multiple stacks detected in workspace. Install which one?")
            .items(&options)
            .default(0)
            .interact()?;

        if sel < stacks.len() {
            execute_stack_install(theme, &stacks[sel], &current_dir)?;
        } else if sel == stacks.len() {
            for stack in &stacks {
                println!();
                execute_stack_install(theme, stack, &current_dir)?;
            }
        } else {
            println!("Cancelled.");
        }
    } else {
        for stack in &stacks {
            execute_stack_install(theme, stack, &current_dir)?;
        }
    }

    Ok(())
}

// ============================================================================
// 27. Universal Web Browser & Developer Search (`run browse`, `run web`, `run brw`)
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchTarget {
    Default,
    Google,
    DuckDuckGo,
    GitHub,
    StackOverflow,
    Crates,
    Npm,
    Mdn,
    Ai,
}

impl SearchTarget {
    pub fn name(&self) -> &'static str {
        match self {
            SearchTarget::Default => "Web",
            SearchTarget::Google => "Google",
            SearchTarget::DuckDuckGo => "DuckDuckGo",
            SearchTarget::GitHub => "GitHub",
            SearchTarget::StackOverflow => "StackOverflow",
            SearchTarget::Crates => "Crates.io",
            SearchTarget::Npm => "npm",
            SearchTarget::Mdn => "MDN Web Docs",
            SearchTarget::Ai => "AI (Perplexity)",
        }
    }
}

pub fn url_encode(input: &str) -> String {
    let mut encoded = String::with_capacity(input.len());
    for b in input.bytes() {
        match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(b as char);
            }
            b' ' => encoded.push('+'),
            _ => {
                use std::fmt::Write;
                let _ = write!(encoded, "%{:02X}", b);
            }
        }
    }
    encoded
}

pub fn is_likely_url(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.contains(' ') {
        return None;
    }
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        return Some(trimmed.to_string());
    }
    if trimmed.starts_with("localhost") || trimmed.starts_with("127.0.0.1") {
        return Some(format!("http://{trimmed}"));
    }
    let common_tlds = [
        ".com", ".org", ".net", ".io", ".dev", ".app", ".co", ".id", ".ai",
        ".me", ".xyz", ".cc", ".tv", ".sh", ".rs", ".so", ".to", ".info", ".edu", ".gov",
    ];
    let lower = trimmed.to_lowercase();
    for tld in common_tlds {
        if let Some(pos) = lower.find(tld) {
            let after = &lower[pos + tld.len()..];
            if after.is_empty()
                || after.starts_with('/')
                || after.starts_with('?')
                || after.starts_with(':')
            {
                return Some(format!("https://{trimmed}"));
            }
        }
    }
    None
}

pub fn build_target_url(target: SearchTarget, query: &str) -> String {
    let encoded = url_encode(query);
    match target {
        SearchTarget::Default => {
            let cfg = config::load_config();
            let engine = cfg
                .search_engine
                .as_deref()
                .unwrap_or("google")
                .to_lowercase();
            match engine.as_str() {
                "duckduckgo" | "ddg" => format!("https://duckduckgo.com/?q={encoded}"),
                "brave" => format!("https://search.brave.com/search?q={encoded}"),
                "bing" => format!("https://www.bing.com/search?q={encoded}"),
                "kagi" => format!("https://kagi.com/search?q={encoded}"),
                "perplexity" => format!("https://www.perplexity.ai/search?q={encoded}"),
                _ => format!("https://www.google.com/search?q={encoded}"),
            }
        }
        SearchTarget::Google => format!("https://www.google.com/search?q={encoded}"),
        SearchTarget::DuckDuckGo => format!("https://duckduckgo.com/?q={encoded}"),
        SearchTarget::GitHub => {
            if query.is_empty() {
                "https://github.com".to_string()
            } else {
                format!("https://github.com/search?q={encoded}")
            }
        }
        SearchTarget::StackOverflow => {
            if query.is_empty() {
                "https://stackoverflow.com".to_string()
            } else {
                format!("https://stackoverflow.com/search?q={encoded}")
            }
        }
        SearchTarget::Crates => {
            if query.is_empty() {
                "https://crates.io".to_string()
            } else {
                format!("https://crates.io/search?q={encoded}")
            }
        }
        SearchTarget::Npm => {
            if query.is_empty() {
                "https://www.npmjs.com".to_string()
            } else {
                format!("https://www.npmjs.com/search?q={encoded}")
            }
        }
        SearchTarget::Mdn => {
            if query.is_empty() {
                "https://developer.mozilla.org".to_string()
            } else {
                format!("https://developer.mozilla.org/en-US/search?q={encoded}")
            }
        }
        SearchTarget::Ai => {
            if query.is_empty() {
                "https://www.perplexity.ai".to_string()
            } else {
                format!("https://www.perplexity.ai/search?q={encoded}")
            }
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct BrowseFlags {
    pub github: bool,
    pub so: bool,
    pub crates: bool,
    pub npm: bool,
    pub mdn: bool,
    pub ai: bool,
}

pub fn handle_browse(
    theme: &ColorfulTheme,
    query_args: &[String],
    flags: BrowseFlags,
) -> Result<()> {
    ui::maybe_auto_clear();

    // 1. Determine target and query
    let (mut target, mut raw_query) = if flags.github {
        (SearchTarget::GitHub, query_args.join(" "))
    } else if flags.so {
        (SearchTarget::StackOverflow, query_args.join(" "))
    } else if flags.crates {
        (SearchTarget::Crates, query_args.join(" "))
    } else if flags.npm {
        (SearchTarget::Npm, query_args.join(" "))
    } else if flags.mdn {
        (SearchTarget::Mdn, query_args.join(" "))
    } else if flags.ai {
        (SearchTarget::Ai, query_args.join(" "))
    } else if let Some(first) = query_args.first() {
        let lower = first.to_lowercase();
        match lower.as_str() {
            "!gh" | "gh" | "github" => (SearchTarget::GitHub, query_args[1..].join(" ")),
            "!so" | "so" | "stackoverflow" => {
                (SearchTarget::StackOverflow, query_args[1..].join(" "))
            }
            "!crate" | "!c" | "crate" | "crates" => {
                (SearchTarget::Crates, query_args[1..].join(" "))
            }
            "!npm" | "npm" => (SearchTarget::Npm, query_args[1..].join(" ")),
            "!mdn" | "mdn" => (SearchTarget::Mdn, query_args[1..].join(" ")),
            "!ai" | "ai" | "perplexity" => (SearchTarget::Ai, query_args[1..].join(" ")),
            "!ddg" | "ddg" | "duckduckgo" => {
                (SearchTarget::DuckDuckGo, query_args[1..].join(" "))
            }
            "!g" | "google" => (SearchTarget::Google, query_args[1..].join(" ")),
            _ => (SearchTarget::Default, query_args.join(" ")),
        }
    } else {
        (SearchTarget::Default, String::new())
    };

    // 2. Handle zero arguments or empty query
    if raw_query.trim().is_empty() {
        let clip_text = crate::mac::read_from_clipboard()
            .unwrap_or_default()
            .trim()
            .to_string();

        if std::io::stdin().is_terminal() {
            ui::print_banner();
            ui::render_breadcrumbs(&["run", "Browse & Web Search"]);

            let cancel_btn = ui::cancel_option();
            let mut options = Vec::new();

            let clip_preview = if !clip_text.is_empty() {
                let first_line = clip_text.lines().next().unwrap_or("").trim();
                let truncated = if first_line.chars().count() > 50 {
                    format!("{}...", first_line.chars().take(50).collect::<String>())
                } else {
                    first_line.to_string()
                };
                Some(truncated)
            } else {
                None
            };

            if let Some(ref prev) = clip_preview {
                options.push(format!("📋 Search clipboard: \"{}\"", prev.bold()));
            }
            options.push("⌨️  Enter search query or URL...".to_string());
            options.push("🐙 Search on GitHub".to_string());
            options.push("📚 Search on StackOverflow".to_string());
            options.push("🦀 Search crates.io (Rust)".to_string());
            options.push("📦 Search npmjs.com (Node)".to_string());
            options.push("📖 Search MDN Web Docs".to_string());
            options.push("🤖 Ask AI (Perplexity)".to_string());
            options.push(cancel_btn);

            let sel = Select::with_theme(theme)
                .with_prompt("Choose search action")
                .items(&options)
                .default(0)
                .interact()?;

            let offset = if clip_preview.is_some() { 1 } else { 0 };

            if clip_preview.is_some() && sel == 0 {
                raw_query = clip_text;
            } else if sel == offset {
                let input: String = Input::with_theme(theme)
                    .with_prompt("Search query or URL (leave blank to cancel)")
                    .allow_empty(true)
                    .interact_text()?;
                let trimmed = input.trim().to_string();
                if trimmed.is_empty() {
                    println!("Cancelled.");
                    return Ok(());
                }
                raw_query = trimmed;
            } else if sel == offset + 1 {
                target = SearchTarget::GitHub;
                let input: String = Input::with_theme(theme)
                    .with_prompt("GitHub search query")
                    .allow_empty(true)
                    .interact_text()?;
                raw_query = input.trim().to_string();
            } else if sel == offset + 2 {
                target = SearchTarget::StackOverflow;
                let input: String = Input::with_theme(theme)
                    .with_prompt("StackOverflow search query")
                    .allow_empty(true)
                    .interact_text()?;
                raw_query = input.trim().to_string();
            } else if sel == offset + 3 {
                target = SearchTarget::Crates;
                let input: String = Input::with_theme(theme)
                    .with_prompt("Crates.io search query")
                    .allow_empty(true)
                    .interact_text()?;
                raw_query = input.trim().to_string();
            } else if sel == offset + 4 {
                target = SearchTarget::Npm;
                let input: String = Input::with_theme(theme)
                    .with_prompt("npm search query")
                    .allow_empty(true)
                    .interact_text()?;
                raw_query = input.trim().to_string();
            } else if sel == offset + 5 {
                target = SearchTarget::Mdn;
                let input: String = Input::with_theme(theme)
                    .with_prompt("MDN search query")
                    .allow_empty(true)
                    .interact_text()?;
                raw_query = input.trim().to_string();
            } else if sel == offset + 6 {
                target = SearchTarget::Ai;
                let input: String = Input::with_theme(theme)
                    .with_prompt("AI search query or question")
                    .allow_empty(true)
                    .interact_text()?;
                raw_query = input.trim().to_string();
            } else {
                println!("Cancelled.");
                return Ok(());
            }
        } else if !clip_text.is_empty() {
            raw_query = clip_text;
        } else {
            bail!("no search query or URL provided (usage: run browse <query> or run web <query>)");
        }
    }

    // 3. Direct URL handling
    if target == SearchTarget::Default
        && let Some(direct_url) = is_likely_url(&raw_query)
    {
        println!(
            "{} Opening URL in default browser: {}",
            "🌐".bold(),
            direct_url.cyan()
        );
        let status = Command::new("open").arg(&direct_url).status()?;
        if !status.success() {
            bail!("failed to open URL '{direct_url}'");
        }
        return Ok(());
    }

    // 4. Build search engine URL and open
    let final_url = build_target_url(target, &raw_query);
    println!(
        "{} Searching {} for: {}",
        "🔍".bold(),
        target.name().green().bold(),
        raw_query.cyan()
    );

    let status = Command::new("open").arg(&final_url).status()?;
    if !status.success() {
        bail!("failed to launch browser for '{final_url}'");
    }

    Ok(())
}

// ============================================================================
// 28. Raycast Script Commands Integration (`run raycast`, `run ray`)
// ============================================================================

pub struct RaycastScript {
    pub filename: &'static str,
    pub title: &'static str,
    pub icon: &'static str,
    pub description: &'static str,
    pub content: &'static str,
}

pub const RAYCAST_SCRIPTS: &[RaycastScript] = &[
    RaycastScript {
        filename: "run.sh",
        title: "Run CLI Command",
        icon: "⚡",
        description: "Universal run-cli runner (e.g. web, pass, uuid, pods, dns)",
        content: r#"#!/bin/bash

# Required parameters:
# @raycast.schemaVersion 1
# @raycast.title Run CLI Command
# @raycast.mode compact

# Optional parameters:
# @raycast.icon ⚡
# @raycast.packageName run-cli
# @raycast.argument1 { "type": "text", "placeholder": "Command (e.g. web rust, pass 32, pods)", "percentEncoded": false }

export PATH="$HOME/.cargo/bin:$PATH"
run $1
"#,
    },
    RaycastScript {
        filename: "run-web.sh",
        title: "Web & Dev Search",
        icon: "🌐",
        description: "Search Google/DDG, GitHub (gh), StackOverflow (so), crates, npm, AI",
        content: r#"#!/bin/bash

# Required parameters:
# @raycast.schemaVersion 1
# @raycast.title Web & Dev Search
# @raycast.mode silent

# Optional parameters:
# @raycast.icon 🌐
# @raycast.packageName run-cli
# @raycast.argument1 { "type": "text", "placeholder": "Query (e.g. gh nextjs, so rust, localhost:3000)", "percentEncoded": false }

export PATH="$HOME/.cargo/bin:$PATH"
run web "$1"
"#,
    },
    RaycastScript {
        filename: "run-uuid.sh",
        title: "Generate UUID",
        icon: "🆔",
        description: "Generate UUID v4 and copy directly to clipboard",
        content: r#"#!/bin/bash

# Required parameters:
# @raycast.schemaVersion 1
# @raycast.title Generate UUID
# @raycast.mode compact

# Optional parameters:
# @raycast.icon 🆔
# @raycast.packageName run-cli

export PATH="$HOME/.cargo/bin:$PATH"
run uuid
"#,
    },
    RaycastScript {
        filename: "run-pass.sh",
        title: "Generate Password",
        icon: "🔑",
        description: "Generate cryptographically secure password to clipboard",
        content: r#"#!/bin/bash

# Required parameters:
# @raycast.schemaVersion 1
# @raycast.title Generate Password
# @raycast.mode compact

# Optional parameters:
# @raycast.icon 🔑
# @raycast.packageName run-cli
# @raycast.argument1 { "type": "text", "placeholder": "Length (default 24)", "optional": true, "percentEncoded": false }

export PATH="$HOME/.cargo/bin:$PATH"
if [ -n "$1" ]; then
  run pass "$1"
else
  run pass 24
fi
"#,
    },
    RaycastScript {
        filename: "run-pods.sh",
        title: "Connect AirPods",
        icon: "🎧",
        description: "Instantly connect paired AirPods or Bluetooth headphones",
        content: r#"#!/bin/bash

# Required parameters:
# @raycast.schemaVersion 1
# @raycast.title Connect AirPods
# @raycast.mode compact

# Optional parameters:
# @raycast.icon 🎧
# @raycast.packageName run-cli

export PATH="$HOME/.cargo/bin:$PATH"
run pods
"#,
    },
    RaycastScript {
        filename: "run-dns.sh",
        title: "Flush macOS DNS",
        icon: "🌐",
        description: "Flush DNS cache in 1 click (dscacheutil + mDNSResponder)",
        content: r#"#!/bin/bash

# Required parameters:
# @raycast.schemaVersion 1
# @raycast.title Flush macOS DNS
# @raycast.mode compact

# Optional parameters:
# @raycast.icon 🌐
# @raycast.packageName run-cli

export PATH="$HOME/.cargo/bin:$PATH"
run dns
"#,
    },
    RaycastScript {
        filename: "run-timer.sh",
        title: "Focus / Pomodoro Timer",
        icon: "⏰",
        description: "Start focus timer with native completion chime and alert",
        content: r#"#!/bin/bash

# Required parameters:
# @raycast.schemaVersion 1
# @raycast.title Focus Timer
# @raycast.mode compact

# Optional parameters:
# @raycast.icon ⏰
# @raycast.packageName run-cli
# @raycast.argument1 { "type": "text", "placeholder": "Minutes (default 25)", "optional": true, "percentEncoded": false }

export PATH="$HOME/.cargo/bin:$PATH"
if [ -n "$1" ]; then
  run timer "$1"
else
  run timer 25
fi
"#,
    },
    RaycastScript {
        filename: "run-fixapp.sh",
        title: "Fix Gatekeeper Quarantine",
        icon: "🛠️",
        description: "Fix 'App is damaged and can't be opened' error",
        content: r#"#!/bin/bash

# Required parameters:
# @raycast.schemaVersion 1
# @raycast.title Fix App Quarantine
# @raycast.mode compact

# Optional parameters:
# @raycast.icon 🛠️
# @raycast.packageName run-cli
# @raycast.argument1 { "type": "text", "placeholder": "App Name (e.g. Figma)", "percentEncoded": false }

export PATH="$HOME/.cargo/bin:$PATH"
run fixapp "$1"
"#,
    },
    RaycastScript {
        filename: "run-awake.sh",
        title: "Keep Mac Awake",
        icon: "☕",
        description: "Prevent display and system sleep (caffeinate)",
        content: r#"#!/bin/bash

# Required parameters:
# @raycast.schemaVersion 1
# @raycast.title Keep Mac Awake
# @raycast.mode compact

# Optional parameters:
# @raycast.icon ☕
# @raycast.packageName run-cli
# @raycast.argument1 { "type": "text", "placeholder": "Minutes (default 60)", "optional": true, "percentEncoded": false }

export PATH="$HOME/.cargo/bin:$PATH"
if [ -n "$1" ]; then
  run awake "$1"
else
  run awake 60
fi
"#,
    },
    RaycastScript {
        filename: "run-clean.sh",
        title: "Clean Build Artifacts",
        icon: "🧹",
        description: "Scan and clean disposable target, node_modules caches",
        content: r#"#!/bin/bash

# Required parameters:
# @raycast.schemaVersion 1
# @raycast.title Clean Build Artifacts
# @raycast.mode compact

# Optional parameters:
# @raycast.icon 🧹
# @raycast.packageName run-cli

export PATH="$HOME/.cargo/bin:$PATH"
run clean
"#,
    },
    RaycastScript {
        filename: "run-update.sh",
        title: "Update System & Toolchains",
        icon: "🔄",
        description: "1-step updater for Homebrew, Rust, and global Node packages",
        content: r#"#!/bin/bash

# Required parameters:
# @raycast.schemaVersion 1
# @raycast.title Update System Toolchains
# @raycast.mode compact

# Optional parameters:
# @raycast.icon 🔄
# @raycast.packageName run-cli

export PATH="$HOME/.cargo/bin:$PATH"
run update
"#,
    },
    RaycastScript {
        filename: "run-voice.sh",
        title: "Speak Text (Voice)",
        icon: "🗣️",
        description: "Native macOS text-to-speech synthesis",
        content: r#"#!/bin/bash

# Required parameters:
# @raycast.schemaVersion 1
# @raycast.title Speak Text
# @raycast.mode compact

# Optional parameters:
# @raycast.icon 🗣️
# @raycast.packageName run-cli
# @raycast.argument1 { "type": "text", "placeholder": "Text to speak", "percentEncoded": false }

export PATH="$HOME/.cargo/bin:$PATH"
run voice "$1"
"#,
    },
];

pub fn handle_raycast(theme: &ColorfulTheme, action: Option<&str>) -> Result<()> {
    ui::maybe_auto_clear();

    let raycast_dir = config::config_dir()?.join("raycast");
    fs::create_dir_all(&raycast_dir)?;

    match action {
        Some("list") => {
            ui::print_banner();
            ui::render_breadcrumbs(&["run", "Raycast Commands"]);
            println!("{}", "Available Raycast Script Commands:\n".bold());
            for script in RAYCAST_SCRIPTS {
                println!(
                    "  {} {:<28} {}",
                    script.icon,
                    script.title.bold(),
                    script.description.dimmed()
                );
            }
            println!("\nDirectory: {}", raycast_dir.display().to_string().cyan());
            return Ok(());
        }
        Some("open") => {
            let _ = Command::new("open").arg(&raycast_dir).status();
            println!(
                "Opened Raycast scripts folder in Finder: {}",
                raycast_dir.display().to_string().cyan()
            );
            return Ok(());
        }
        _ => {}
    }

    // Default or "install" / "setup"
    ui::print_banner();
    ui::render_breadcrumbs(&["run", "Raycast Integration"]);

    let mut installed_count = 0;
    for script in RAYCAST_SCRIPTS {
        let script_path = raycast_dir.join(script.filename);
        fs::write(&script_path, script.content)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = fs::metadata(&script_path) {
                let mut perms = metadata.permissions();
                perms.set_mode(0o755);
                let _ = fs::set_permissions(&script_path, perms);
            }
        }
        installed_count += 1;
    }

    println!(
        "{} Successfully installed {} Raycast Script Commands to:",
        "✔".green().bold(),
        installed_count.to_string().bold()
    );
    println!("   {}\n", raycast_dir.display().to_string().cyan().bold());

    ui::print_card(
        "Raycast Cmd+Space Setup Instructions",
        &[
            ("1. Open Raycast", "Press Cmd + Space".to_string()),
            ("2. Open Settings", "Press Cmd + , (or search 'Settings')".to_string()),
            ("3. Go to Extensions", "Click 'Extensions' tab at the top".to_string()),
            ("4. Add Directory", "Click '+' icon at top right -> 'Add Script Directory'".to_string()),
            ("5. Select Folder", "~/.config/run/raycast".to_string()),
        ],
    );

    if std::io::stdin().is_terminal() {
        let open_finder = Confirm::with_theme(theme)
            .with_prompt("Open Raycast scripts directory in Finder now?")
            .default(true)
            .interact()?;

        if open_finder {
            let _ = Command::new("open").arg(&raycast_dir).status();
            println!("{} Opened scripts folder in Finder! 🎉", "✔".green());
        }
    }

    Ok(())
}

