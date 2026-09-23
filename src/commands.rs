use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

use anyhow::{bail, Context, Result};
use colored::Colorize;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Confirm, Input, Select};

fn electric_blue(text: &str) -> colored::ColoredString {
    text.truecolor(0, 162, 255)
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

/// path (alias: pth, pwd) - Print current working directory
pub fn handle_path(theme: &ColorfulTheme, interactive: bool) -> Result<()> {
    let current_dir = std::env::current_dir().context("failed to get current working directory")?;
    let path_str = current_dir.to_string_lossy().to_string();

    if interactive {
        let options = ["Print path", "Copy path to clipboard (pbcopy)", "Cancel"];
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
            options.push("Cancel".to_string());

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

    let content = fs::read_to_string(&target)
        .with_context(|| format!("failed to read file '{}' (binary files cannot be read as text)", target.display()))?;

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
pub fn handle_kill(
    theme: &ColorfulTheme,
    target: Option<&str>,
    force: bool,
) -> Result<()> {
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

            proc_list.push("Cancel".to_string());

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
            println!("{}", format!("Searching processes using port :{port_clean}...").dimmed());
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
                println!("{}", format!("Searching processes on port :{port_clean}...").dimmed());
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
        cmd.args(["-L", "--progress-bar", "-o", &out.to_string_lossy(), &target_url]);
        println!("{}", format!("Downloading '{target_url}' to '{}'...", out.display()).dimmed());
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
                    let mut options: Vec<String> = dirs.iter().map(|(n, _)| format!("{n}/")).collect();
                    options.insert(0, ". (Current directory)".to_string());
                    options.push("Cancel".to_string());

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

        println!("{}", format!("Calculating disk usage for '{}'...", target.display()).dimmed());
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
                    if name.ends_with(".zip") || name.ends_with(".tar.gz") || name.ends_with(".tgz") || name.ends_with(".tar") {
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
                options.push("Cancel".to_string());

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

    println!("{}", format!("Extracted archive: {}", arc.display()).green());
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
            let presets = [
                "755 (rwxr-xr-x) - Executable script / application",
                "644 (rw-r--r--) - Standard readable file",
                "600 (rw-------) - Private / Secret credential",
                "+x  (Make executable)",
                "Cancel",
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

    println!("{}", format!("Updated permissions: chmod {} {}", resolved_mode, target.display()).green());
    Ok(())
}

/// ping (alias: png) - Check network latency to host
pub fn handle_ping(theme: &ColorfulTheme, host: Option<&str>) -> Result<()> {
    let target_host = match host {
        Some(h) => h.to_string(),
        None => {
            let hosts = [
                "1.1.1.1 (Cloudflare DNS)",
                "8.8.8.8 (Google DNS)",
                "google.com",
                "Custom host...",
                "Cancel",
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

    println!("{}", format!("Pinging {target_host} (4 packets)...").dimmed());
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
            .filter(|(k, v)| k.to_lowercase().contains(&q_lower) || v.to_lowercase().contains(&q_lower))
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
