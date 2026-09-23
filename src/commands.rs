use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, Result, bail};
use colored::Colorize;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Confirm, Input, Select};

use crate::ui;

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

/// port (alias: prt, lsof) - Inspect active network ports and listening sockets
pub fn handle_port(theme: &ColorfulTheme, port: Option<&str>) -> Result<()> {
    let mut cmd = Command::new("lsof");

    match port {
        Some(p) => {
            let port_clean = p.trim_start_matches(':');
            cmd.arg(format!("-i:{port_clean}"));
            println!(
                "{}",
                format!("Searching processes using port :{port_clean}...").dimmed()
            );
        }
        None => {
            let input: String = Input::with_theme(theme)
                .with_prompt("Port to inspect (leave blank to list all listening TCP ports)")
                .allow_empty(true)
                .interact_text()?;

            let trimmed = input.trim();
            if trimmed.is_empty() {
                cmd.args(["-iTCP", "-sTCP:LISTEN", "-P", "-n"]);
                println!("{}", "Listing all active listening ports...".dimmed());
            } else {
                let port_clean = trimmed.trim_start_matches(':');
                cmd.arg(format!("-i:{port_clean}"));
                println!(
                    "{}",
                    format!("Searching processes on port :{port_clean}...").dimmed()
                );
            }
        }
    }

    let status = cmd.status().context("failed to run lsof")?;
    if !status.success() && status.code() != Some(1) {
        bail!("port inspection failed");
    }

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
