use std::collections::HashSet;
use std::fs;
use std::io::{IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use colored::Colorize;
use dialoguer::{Confirm, FuzzySelect, Input, Select};

mod commands;
mod completion;
mod config;
mod ui;

use ui::RunTheme as ColorfulTheme;

#[derive(Parser)]
#[command(name = "run")]
#[command(
    about = "Productivity CLI utility for macOS with dual-mode interaction",
    version
)]
#[command(disable_help_subcommand = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new folder or file directly or through an interactive menu
    #[command(name = "make", alias = "mak", aliases = ["mkdir", "touch"])]
    Make {
        /// Target type ('folder'/'file') or path to create directly
        item: Option<String>,
        /// Name or path if target type was specified first
        name: Option<PathBuf>,
        /// Explicitly create as file
        #[arg(short = 'f', long)]
        file: bool,
        /// Explicitly create as folder
        #[arg(short = 'd', long)]
        folder: bool,
    },

    /// Remove a file or directory with safe confirmation
    #[command(name = "remove", alias = "rmv", aliases = ["del", "dlt", "rm", "delete"])]
    Remove {
        /// Path of the file or folder to delete
        target: Option<PathBuf>,
    },

    /// Open a macOS application using open -a
    #[command(name = "open", alias = "opn")]
    Open {
        /// Target application name
        target: Option<String>,
    },

    /// Copy a file or folder from source to destination
    #[command(name = "copy", alias = "cpy", alias = "cp")]
    Copy {
        /// Source file or folder
        source: Option<PathBuf>,
        /// Destination path
        destination: Option<PathBuf>,
    },

    /// Move or rename a file or folder
    #[command(name = "move", alias = "mov", alias = "mv")]
    Move {
        /// Source file or folder
        source: Option<PathBuf>,
        /// Destination path
        destination: Option<PathBuf>,
    },

    /// Clear the terminal screen
    #[command(name = "clear", alias = "clr")]
    Clear,

    /// Smart directory navigation (root, back, subfolder, or interactive menu)
    #[command(name = "go", alias = "jmp", aliases = ["nav", "cd", "g"])]
    Go {
        /// Target folder name, 'root', or 'back'
        target: Option<String>,
    },

    /// List directory contents
    #[command(name = "list", alias = "lst", alias = "ls")]
    List {
        /// Target directory path
        path: Option<PathBuf>,
        /// Show hidden entries
        #[arg(short = 'a', long)]
        all: bool,
        /// Long listing format
        #[arg(short = 'l', long)]
        long: bool,
    },

    /// Print current working directory path
    #[command(name = "path", alias = "pth", alias = "pwd")]
    Path {
        /// Interactive mode with clipboard copy option
        #[arg(short, long)]
        interactive: bool,
    },

    /// Display file contents
    #[command(name = "read", alias = "red", alias = "cat")]
    Read {
        /// Path to file
        path: Option<PathBuf>,
    },

    /// Search pattern or text across files
    #[command(name = "find", alias = "fnd", aliases = ["grep", "search"])]
    Find {
        /// Search pattern
        pattern: Option<String>,
        /// Target file or directory
        path: Option<PathBuf>,
    },

    /// Inspect running processes or system resource snapshot
    #[command(name = "process", alias = "prc", aliases = ["proc", "ps", "top"])]
    Process {
        /// Filter by process name or user
        filter: Option<String>,
        /// Show resource usage snapshot
        #[arg(short, long)]
        snapshot: bool,
    },

    /// Terminate process by PID or name
    #[command(name = "kill", alias = "kil", aliases = ["stop", "stp"])]
    Kill {
        /// Process PID or name
        target: Option<String>,
        /// Force kill with SIGKILL (-9)
        #[arg(short = 'f', long)]
        force: bool,
    },

    /// Inspect active network ports and listening sockets
    #[command(name = "port", alias = "prt", alias = "lsof")]
    Port {
        /// Port to query (e.g. 3000, 8080)
        port: Option<String>,
    },

    /// Fetch URL response or download file locally
    #[command(name = "fetch", alias = "fch", aliases = ["get", "dwn", "curl", "wget"])]
    Fetch {
        /// URL to request or download
        url: Option<String>,
        /// Output file path for download
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Fetch headers only
        #[arg(short = 'i', long)]
        headers: bool,
    },

    /// Inspect disk free space or directory usage
    #[command(name = "disk", alias = "dsk", aliases = ["df", "du"])]
    Disk {
        /// Target directory to inspect usage
        path: Option<PathBuf>,
        /// Calculate usage instead of free space
        #[arg(short, long)]
        usage: bool,
    },

    /// Create compressed archive (.tar.gz or .zip)
    #[command(name = "pack", alias = "pck", aliases = ["tar", "zip"])]
    Pack {
        /// Archive name
        archive: Option<PathBuf>,
        /// Target folder or file
        target: Option<PathBuf>,
    },

    /// Extract compressed archive (.zip or .tar.gz)
    #[command(name = "unpack", alias = "upk", aliases = ["unzip", "untar"])]
    Unpack {
        /// Archive file path
        archive: Option<PathBuf>,
        /// Destination directory
        destination: Option<PathBuf>,
    },

    /// Change file or directory permissions
    #[command(name = "permit", alias = "prm", alias = "chmod")]
    Permit {
        /// Permission mode (e.g. 755, 644, +x)
        mode: Option<String>,
        /// Target file or folder
        path: Option<PathBuf>,
    },

    /// Test network host latency
    #[command(name = "ping", alias = "png")]
    Ping {
        /// Target host or IP
        host: Option<String>,
    },

    /// Print current username and system identity
    #[command(name = "whoami", alias = "who", aliases = ["user", "usr"])]
    Whoami,

    /// Display formatted current date and time
    #[command(name = "time", alias = "tim", aliases = ["date", "dat"])]
    Time {
        /// Optional date format
        format: Option<String>,
    },

    /// Display recent shell command history
    #[command(name = "history", alias = "his")]
    History {
        /// Number of recent commands to display
        limit: Option<usize>,
    },

    /// Locate executable binary in PATH
    #[command(name = "which", alias = "whc", alias = "loc")]
    Which {
        /// Binary name to locate
        binary: Option<String>,
    },

    /// Inspect or search environment variables
    #[command(name = "environment", alias = "env")]
    Env {
        /// Variable name or filter
        key: Option<String>,
    },

    /// Smart project management, workspace scanner, and IDE selector
    #[command(name = "project", alias = "prj", alias = "pro")]
    Project {
        /// Target project path or keyword ('.' for current directory)
        target: Option<String>,
    },

    /// Run development server for the current active project
    #[command(name = "develop", alias = "dev")]
    Dev,

    /// Build or compile the current active project
    #[command(name = "build", alias = "bld")]
    Build,

    /// Generate shell integration script for automatic directory switching
    #[command(name = "init", alias = "ini")]
    Init,

    /// Run automated test runner for the active project
    #[command(name = "test", alias = "tst")]
    Test,

    /// Clean disposable build artifacts and caches (target, node_modules, etc.)
    #[command(name = "clean", alias = "cln")]
    Clean {
        /// Target directory to scan and clean
        path: Option<PathBuf>,
    },

    /// Git status, pull, commit, and push sync
    #[command(name = "sync", alias = "snc", alias = "git")]
    Sync {
        /// Commit message
        #[arg(short, long)]
        message: Option<String>,
        /// Switch branch interactively
        #[arg(short = 'b', long)]
        branch: bool,
    },

    /// Inspect local LAN and public IP addresses
    #[command(name = "network", alias = "net", alias = "ip")]
    Network,

    /// Instant local HTTP file server on local network
    #[command(name = "share", alias = "shr")]
    Share {
        /// Port to serve on
        #[arg(short, long)]
        port: Option<u16>,
        /// Directory to share
        #[arg(short = 'd', long)]
        path: Option<PathBuf>,
    },

    /// Benchmark command execution time
    #[command(name = "bench", alias = "bnc")]
    Bench {
        /// Command and arguments to benchmark
        #[arg(trailing_var_arg = true)]
        command: Vec<String>,
    },

    /// Test network internet speed and responsiveness
    #[command(name = "speedtest", alias = "spd", alias = "speed")]
    Speedtest {
        /// Run tests sequentially instead of parallel
        #[arg(short, long)]
        sequential: bool,
    },

    /// Manage Docker and OrbStack containers, inspect status, view logs, start or stop
    #[command(name = "docker", alias = "dck")]
    Docker,

    /// Audit local .env files against .env.example, find missing secrets, or generate templates
    #[command(name = "secret", alias = "sec", alias = "dotenv")]
    Secret {
        /// Automatically generate .env.example template from .env
        #[arg(long)]
        fix: bool,
    },

    /// Developer quick scratchpad & snippet clipboard manager
    #[command(name = "memo", alias = "mem", alias = "clip")]
    Memo {
        /// Subaction: add, copy, rm, clear, or view
        action: Option<String>,
        /// Snippet title (for add, copy, or rm)
        arg1: Option<String>,
        /// Snippet content (for add)
        arg2: Option<String>,
    },

    /// Generate native shell auto-completion script (zsh, bash, fish)
    #[command(name = "completion", alias = "cmp")]
    Completion {
        /// Target shell (zsh, bash, fish, powershell, elvish)
        shell: String,
    },

    /// Manage run-cli configuration, colors, default IDE, and auto-clear
    #[command(name = "config", alias = "cfg")]
    Config {
        /// Subaction: get, set, path, edit, or reset
        action: Option<String>,
        /// Configuration key (primary_color, default_ide, auto_clear, custom_hubs)
        key: Option<String>,
        /// Configuration value to set
        val: Option<String>,
    },

    /// Display complete command reference and usage tutorial
    #[command(name = "help", alias = "doc", alias = "guide")]
    Help {
        /// Optional command name or alias to inspect
        command: Option<String>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MakeTargetType {
    Folder,
    File,
}

fn cancel_option() -> String {
    ui::cancel_option()
}

fn main() {
    if let Err(err) = run_app() {
        eprintln!("{} {err:#}", "error:".bold().red());
        std::process::exit(1);
    }
}

#[derive(Clone, Copy)]
struct CommandInfo {
    name: &'static str,
    alias_3: &'static str,
    aliases: &'static [&'static str],
    description: &'static str,
}

const ALL_COMMANDS: &[CommandInfo] = &[
    CommandInfo {
        name: "project",
        alias_3: "prj",
        aliases: &["prj", "pro"],
        description: "Scan workspace projects & open in IDE or terminal",
    },
    CommandInfo {
        name: "develop",
        alias_3: "dev",
        aliases: &["dev"],
        description: "Run active project development server",
    },
    CommandInfo {
        name: "build",
        alias_3: "bld",
        aliases: &["bld"],
        description: "Compile or build active project in release mode",
    },
    CommandInfo {
        name: "go",
        alias_3: "jmp",
        aliases: &["jmp", "nav", "cd", "g"],
        description: "Smart directory navigation, root/back, & folder picker",
    },
    CommandInfo {
        name: "make",
        alias_3: "mak",
        aliases: &["mak", "mkdir", "touch"],
        description: "Create folders or files directly or interactively",
    },
    CommandInfo {
        name: "remove",
        alias_3: "rmv",
        aliases: &["rmv", "del", "dlt", "rm", "delete"],
        description: "Safely delete files or directories with confirmation",
    },
    CommandInfo {
        name: "copy",
        alias_3: "cpy",
        aliases: &["cpy", "cp"],
        description: "Copy files or directory trees recursively",
    },
    CommandInfo {
        name: "move",
        alias_3: "mov",
        aliases: &["mov", "mv"],
        description: "Move or rename files and directories",
    },
    CommandInfo {
        name: "list",
        alias_3: "lst",
        aliases: &["lst", "ls"],
        description: "List directory contents with formatted sizes",
    },
    CommandInfo {
        name: "path",
        alias_3: "pth",
        aliases: &["pth", "pwd"],
        description: "Print or copy current working directory path",
    },
    CommandInfo {
        name: "read",
        alias_3: "red",
        aliases: &["red", "cat"],
        description: "Inspect file contents directly in terminal",
    },
    CommandInfo {
        name: "find",
        alias_3: "fnd",
        aliases: &["fnd", "grep", "search"],
        description: "Search pattern or text across files recursively",
    },
    CommandInfo {
        name: "process",
        alias_3: "prc",
        aliases: &["prc", "proc", "ps", "top"],
        description: "Inspect active processes or resource usage snapshot",
    },
    CommandInfo {
        name: "kill",
        alias_3: "kil",
        aliases: &["kil", "stop", "stp"],
        description: "Terminate process by PID or name with confirmation",
    },
    CommandInfo {
        name: "port",
        alias_3: "prt",
        aliases: &["prt", "lsof"],
        description: "Check active listening TCP ports and sockets",
    },
    CommandInfo {
        name: "fetch",
        alias_3: "fch",
        aliases: &["fch", "get", "dwn", "curl", "wget"],
        description: "Fetch HTTP response or download file with progress",
    },
    CommandInfo {
        name: "disk",
        alias_3: "dsk",
        aliases: &["dsk", "df", "du"],
        description: "Inspect disk free space or directory storage usage",
    },
    CommandInfo {
        name: "pack",
        alias_3: "pck",
        aliases: &["pck", "tar", "zip"],
        description: "Create compressed archive (.tar.gz or .zip)",
    },
    CommandInfo {
        name: "unpack",
        alias_3: "upk",
        aliases: &["upk", "unzip", "untar"],
        description: "Extract compressed archive (.zip or .tar.gz)",
    },
    CommandInfo {
        name: "permit",
        alias_3: "prm",
        aliases: &["prm", "chmod"],
        description: "Change file permissions with presets (755, 644, +x)",
    },
    CommandInfo {
        name: "ping",
        alias_3: "png",
        aliases: &["png"],
        description: "Test network host latency and connectivity",
    },
    CommandInfo {
        name: "whoami",
        alias_3: "who",
        aliases: &["who", "user", "usr"],
        description: "Display user identity, UID, GID, and hostname",
    },
    CommandInfo {
        name: "time",
        alias_3: "tim",
        aliases: &["tim", "date", "dat"],
        description: "Display current formatted date and time",
    },
    CommandInfo {
        name: "history",
        alias_3: "his",
        aliases: &["his"],
        description: "Display recent shell command history",
    },
    CommandInfo {
        name: "which",
        alias_3: "whc",
        aliases: &["whc", "loc"],
        description: "Locate executable binary in system PATH",
    },
    CommandInfo {
        name: "environment",
        alias_3: "env",
        aliases: &["env"],
        description: "Inspect or search environment variables",
    },
    CommandInfo {
        name: "open",
        alias_3: "opn",
        aliases: &["opn"],
        description: "Launch macOS application via open -a",
    },
    CommandInfo {
        name: "clear",
        alias_3: "clr",
        aliases: &["clr"],
        description: "Clear terminal screen viewport",
    },
    CommandInfo {
        name: "init",
        alias_3: "ini",
        aliases: &["ini"],
        description: "Generate shell integration wrapper for ~/.zshrc",
    },
    CommandInfo {
        name: "test",
        alias_3: "tst",
        aliases: &["tst"],
        description: "Run automated tests for the active project",
    },
    CommandInfo {
        name: "clean",
        alias_3: "cln",
        aliases: &["cln"],
        description: "Clean disposable build caches & artifacts",
    },
    CommandInfo {
        name: "sync",
        alias_3: "snc",
        aliases: &["snc", "git"],
        description: "One-step git pull, commit, and push sync",
    },
    CommandInfo {
        name: "network",
        alias_3: "net",
        aliases: &["net", "ip"],
        description: "Inspect local LAN and public IP addresses",
    },
    CommandInfo {
        name: "share",
        alias_3: "shr",
        aliases: &["shr"],
        description: "Instant local HTTP file server on LAN",
    },
    CommandInfo {
        name: "bench",
        alias_3: "bnc",
        aliases: &["bnc"],
        description: "High-resolution command execution benchmark",
    },
    CommandInfo {
        name: "speedtest",
        alias_3: "spd",
        aliases: &["spd", "speed"],
        description: "Measure internet download, upload and latency",
    },
    CommandInfo {
        name: "docker",
        alias_3: "dck",
        aliases: &["dck"],
        description: "Inspect & manage Docker/OrbStack containers & logs",
    },
    CommandInfo {
        name: "secret",
        alias_3: "sec",
        aliases: &["sec", "dotenv"],
        description: "Audit & sync .env and .env.example environment variables",
    },
    CommandInfo {
        name: "memo",
        alias_3: "mem",
        aliases: &["mem", "clip"],
        description: "Quick developer scratchpad & snippet clipboard manager",
    },
    CommandInfo {
        name: "completion",
        alias_3: "cmp",
        aliases: &["cmp"],
        description: "Generate native shell tab-completion (zsh, bash, fish)",
    },
    CommandInfo {
        name: "config",
        alias_3: "cfg",
        aliases: &["cfg"],
        description: "Manage CLI settings, primary color, editor & auto-clear",
    },
    CommandInfo {
        name: "help",
        alias_3: "doc",
        aliases: &["doc", "guide"],
        description: "Display complete command reference & tutorial",
    },
];

fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let m = a_chars.len();
    let n = b_chars.len();

    let mut dp = vec![vec![0; n + 1]; m + 1];

    for (i, row) in dp.iter_mut().enumerate().take(m + 1) {
        row[0] = i;
    }
    for (j, item) in dp[0].iter_mut().enumerate().take(n + 1) {
        *item = j;
    }

    for i in 1..=m {
        for j in 1..=n {
            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                0
            } else {
                1
            };
            dp[i][j] = (dp[i - 1][j] + 1)
                .min(dp[i][j - 1] + 1)
                .min(dp[i - 1][j - 1] + cost);
        }
    }

    dp[m][n]
}

fn resolve_command_args(theme: &ColorfulTheme, args: &[String]) -> Result<Option<Vec<String>>> {
    let interactive =
        std::io::stdin().is_terminal() && std::env::var("RUN_TEST_NON_INTERACTIVE").is_err();
    resolve_command_args_internal(theme, args, interactive)
}

fn resolve_command_args_internal(
    theme: &ColorfulTheme,
    args: &[String],
    interactive: bool,
) -> Result<Option<Vec<String>>> {
    if args.len() <= 1 {
        return Ok(Some(args.to_vec()));
    }

    let first = &args[1];
    if first.starts_with('-') {
        return Ok(Some(args.to_vec()));
    }

    let query = first.to_lowercase();

    for cmd in ALL_COMMANDS {
        if cmd.name == query || cmd.alias_3 == query || cmd.aliases.contains(&query.as_str()) {
            let mut new_args = args.to_vec();
            new_args[1] = cmd.name.to_string();
            return Ok(Some(new_args));
        }
    }

    if Cli::try_parse_from(args).is_ok() {
        return Ok(Some(args.to_vec()));
    }

    let mut prefix_matches: Vec<&'static CommandInfo> = Vec::new();
    for cmd in ALL_COMMANDS {
        let matches_name = cmd.name.starts_with(&query);
        let matches_alias =
            cmd.alias_3.starts_with(&query) || cmd.aliases.iter().any(|a| a.starts_with(&query));
        if (matches_name || matches_alias) && !prefix_matches.iter().any(|m| m.name == cmd.name) {
            prefix_matches.push(cmd);
        }
    }

    if prefix_matches.len() == 1 {
        let matched = prefix_matches[0];
        let mut new_args = args.to_vec();
        new_args[1] = matched.name.to_string();
        return Ok(Some(new_args));
    }

    if prefix_matches.len() > 1 {
        if !interactive {
            let mut new_args = args.to_vec();
            new_args[1] = prefix_matches[0].name.to_string();
            return Ok(Some(new_args));
        }

        let mut menu_items: Vec<String> = prefix_matches
            .iter()
            .map(|cmd| {
                let alias_display = if cmd.alias_3 != cmd.name {
                    format!(" ({})", cmd.alias_3)
                } else {
                    String::new()
                };
                format!("{:<14}{:<8} {}", cmd.name, alias_display, cmd.description)
            })
            .collect();
        menu_items.push(cancel_option());

        let prompt = format!("Multiple commands match '{}':", query);
        let selection = Select::with_theme(theme)
            .with_prompt(prompt)
            .items(&menu_items)
            .default(0)
            .interact()?;

        if selection >= prefix_matches.len() {
            println!("Cancelled.");
            return Ok(None);
        }

        let selected = prefix_matches[selection];
        let mut new_args = args.to_vec();
        new_args[1] = selected.name.to_string();
        return Ok(Some(new_args));
    }

    let mut scored: Vec<(&'static CommandInfo, usize, bool)> = ALL_COMMANDS
        .iter()
        .map(|cmd| {
            let dist_name = levenshtein_distance(&query, cmd.name);
            let mut min_dist = dist_name;
            let mut is_primary = true;

            let dist_alias3 = levenshtein_distance(&query, cmd.alias_3);
            if dist_alias3 < min_dist {
                min_dist = dist_alias3;
                is_primary = false;
            }
            for alias in cmd.aliases {
                let d = levenshtein_distance(&query, alias);
                if d < min_dist {
                    min_dist = d;
                    is_primary = false;
                }
            }
            (cmd, min_dist, is_primary)
        })
        .collect();

    // Sort by distance ascending, then prefer primary command names over aliases on ties
    scored.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| b.2.cmp(&a.2)));

    let max_allowed = if query.len() >= 6 { 3 } else { 2 };
    let candidates: Vec<&'static CommandInfo> = scored
        .into_iter()
        .filter(|(_, dist, _)| *dist <= max_allowed)
        .take(5)
        .map(|(cmd, _, _)| cmd)
        .collect();

    if !candidates.is_empty() {
        if !interactive {
            let mut new_args = args.to_vec();
            new_args[1] = candidates[0].name.to_string();
            return Ok(Some(new_args));
        }

        let mut menu_items: Vec<String> = candidates
            .iter()
            .map(|cmd| {
                let alias_display = if cmd.alias_3 != cmd.name {
                    format!(" ({})", cmd.alias_3)
                } else {
                    String::new()
                };
                format!("{:<14}{:<8} {}", cmd.name, alias_display, cmd.description)
            })
            .collect();
        menu_items.push(cancel_option());

        let prompt = format!("Unknown command '{}'. Did you mean:", query);
        let selection = Select::with_theme(theme)
            .with_prompt(prompt)
            .items(&menu_items)
            .default(0)
            .interact()?;

        if selection >= candidates.len() {
            println!("Cancelled.");
            return Ok(None);
        }

        let selected = candidates[selection];
        let mut new_args = args.to_vec();
        new_args[1] = selected.name.to_string();
        return Ok(Some(new_args));
    }

    Ok(Some(args.to_vec()))
}

fn handle_all_commands_menu(theme: &ColorfulTheme) -> Result<()> {
    ui::maybe_auto_clear();
    ui::print_banner();
    ui::render_breadcrumbs(&["run", "Launcher Dashboard"]);

    let categories = [
        "🛠️   Developer & Workspace  (project, dev, build, docker, config...)",
        "📂  Filesystem & Navigation (go, list, make, memo, read, find...)",
        "⚙️   System & Monitoring     (process, kill, disk, whoami, time...)",
        "🌐  Network & Utilities     (port, fetch, speedtest, completion...)",
        "🔍  Search All 41 Commands... (search as you type)",
    ];
    let cancel_btn = cancel_option();
    let mut dashboard_options: Vec<String> = categories.iter().map(|s| s.to_string()).collect();
    dashboard_options.push(cancel_btn);

    ui::print_key_hints();
    let cat_selection = Select::with_theme(theme)
        .with_prompt("Select workspace category or search")
        .items(&dashboard_options)
        .default(0)
        .interact()?;

    if cat_selection >= categories.len() {
        println!("Cancelled.");
        return Ok(());
    }

    let target_commands: Vec<CommandInfo> = match cat_selection {
        0 => ALL_COMMANDS
            .iter()
            .copied()
            .filter(|c| {
                [
                    "project", "dev", "build", "test", "clean", "sync", "network", "share",
                    "bench", "docker", "secret", "config",
                ]
                .contains(&c.name)
            })
            .collect(),
        1 => ALL_COMMANDS
            .iter()
            .copied()
            .filter(|c| {
                [
                    "go", "path", "list", "make", "remove", "copy", "move", "read", "find",
                    "permit", "memo",
                ]
                .contains(&c.name)
            })
            .collect(),
        2 => ALL_COMMANDS
            .iter()
            .copied()
            .filter(|c| {
                [
                    "process", "kill", "disk", "whoami", "time", "history", "which", "env",
                ]
                .contains(&c.name)
            })
            .collect(),
        3 => ALL_COMMANDS
            .iter()
            .copied()
            .filter(|c| {
                [
                    "port",
                    "fetch",
                    "ping",
                    "pack",
                    "unpack",
                    "speedtest",
                    "completion",
                ]
                .contains(&c.name)
            })
            .collect(),
        4 => ALL_COMMANDS.to_vec(),
        _ => return Ok(()),
    };

    let mut menu_items: Vec<String> = target_commands
        .iter()
        .map(|cmd| {
            let alias_display = if cmd.alias_3 != cmd.name {
                format!(" ({})", cmd.alias_3)
            } else {
                String::new()
            };
            format!("{:<14}{:<8} {}", cmd.name, alias_display, cmd.description)
        })
        .collect();
    menu_items.push(cancel_option());

    let selection = if cat_selection == 4 {
        FuzzySelect::with_theme(theme)
            .with_prompt("Type to filter commands")
            .items(&menu_items)
            .default(0)
            .interact()?
    } else {
        Select::with_theme(theme)
            .with_prompt("Select command to execute")
            .items(&menu_items)
            .default(0)
            .interact()?
    };

    if selection >= target_commands.len() {
        println!("Cancelled.");
        return Ok(());
    }

    let selected = target_commands[selection];
    let new_args = vec!["run".to_string(), selected.name.to_string()];
    let cli = Cli::try_parse_from(&new_args)?;
    if let Some(cmd) = cli.command {
        dispatch_command(theme, cmd)?;
    }
    Ok(())
}

fn dispatch_command(theme: &ColorfulTheme, command: Commands) -> Result<()> {
    match command {
        Commands::Make {
            item,
            name,
            file,
            folder,
        } => handle_make(theme, item, name, file, folder),
        Commands::Remove { target } => handle_del(theme, target),
        Commands::Open { target } => handle_open(theme, target),
        Commands::Copy {
            source,
            destination,
        } => handle_copy(theme, source, destination),
        Commands::Move {
            source,
            destination,
        } => handle_move(theme, source, destination),
        Commands::Clear => clear_terminal(),
        Commands::Go { target } => handle_go(theme, target.as_deref()),
        Commands::List { path, all, long } => commands::handle_list(theme, path, all, long),
        Commands::Path { interactive } => commands::handle_path(theme, interactive),
        Commands::Read { path } => commands::handle_read(theme, path),
        Commands::Find { pattern, path } => commands::handle_find(theme, pattern, path),
        Commands::Process { filter, snapshot } => {
            commands::handle_process(filter.as_deref(), snapshot)
        }
        Commands::Kill { target, force } => commands::handle_kill(theme, target.as_deref(), force),
        Commands::Port { port } => commands::handle_port(theme, port.as_deref()),
        Commands::Fetch {
            url,
            output,
            headers,
        } => commands::handle_fetch(theme, url.as_deref(), output, headers),
        Commands::Disk { path, usage } => commands::handle_disk(theme, path, usage),
        Commands::Pack { archive, target } => commands::handle_pack(theme, archive, target),
        Commands::Unpack {
            archive,
            destination,
        } => commands::handle_unpack(theme, archive, destination),
        Commands::Permit { mode, path } => commands::handle_permit(theme, mode.as_deref(), path),
        Commands::Ping { host } => commands::handle_ping(theme, host.as_deref()),
        Commands::Whoami => commands::handle_whoami(),
        Commands::Time { format } => commands::handle_time(format.as_deref()),
        Commands::History { limit } => commands::handle_history(limit),
        Commands::Which { binary } => commands::handle_which(theme, binary.as_deref()),
        Commands::Env { key } => commands::handle_env(key.as_deref()),
        Commands::Project { target } => handle_project(theme, target.as_deref()),
        Commands::Dev => handle_dev(),
        Commands::Build => handle_build(),
        Commands::Test => commands::handle_test(),
        Commands::Clean { path } => commands::handle_clean(theme, path),
        Commands::Sync { message, branch } => commands::handle_sync(theme, message, branch),
        Commands::Network => commands::handle_net(theme),
        Commands::Share { port, path } => commands::handle_share(theme, port, path),
        Commands::Bench { command } => commands::handle_bench(theme, &command),
        Commands::Speedtest { sequential } => commands::handle_speedtest(theme, sequential),
        Commands::Docker => commands::handle_docker(theme),
        Commands::Secret { fix } => commands::handle_secret(theme, fix),
        Commands::Memo { action, arg1, arg2 } => {
            commands::handle_memo(theme, action.as_deref(), arg1.as_deref(), arg2.as_deref())
        }
        Commands::Completion { shell } => completion::generate_completion(&shell),
        Commands::Config { action, key, val } => {
            commands::handle_config(theme, action.as_deref(), key.as_deref(), val.as_deref())
        }
        Commands::Init => handle_init(),
        Commands::Help { command } => handle_help(command.as_deref()),
    }
}

fn run_app() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let theme = custom_theme();

    ui::maybe_auto_clear();

    if args.len() <= 1 {
        if std::io::stdin().is_terminal() {
            return handle_all_commands_menu(&theme);
        } else {
            return handle_help(None);
        }
    }

    let resolved_args = resolve_command_args(&theme, &args)?;
    let Some(final_args) = resolved_args else {
        return Ok(());
    };

    let cli = match Cli::try_parse_from(&final_args) {
        Ok(c) => c,
        Err(err) => err.exit(),
    };
    match cli.command {
        Some(command) => dispatch_command(&theme, command),
        None => {
            if std::io::stdin().is_terminal() {
                handle_all_commands_menu(&theme)
            } else {
                handle_help(None)
            }
        }
    }
}

/// Handles folder or file creation in both direct and interactive modes.
fn handle_make(
    theme: &ColorfulTheme,
    item: Option<String>,
    name: Option<PathBuf>,
    force_file: bool,
    force_folder: bool,
) -> Result<()> {
    let (target_type, target_path) = match (item, name) {
        (None, None) => {
            let cancel_btn = cancel_option();
            let options = ["Folder", "File", cancel_btn.as_str()];
            let selection = Select::with_theme(theme)
                .with_prompt("Select item type to create")
                .items(&options)
                .default(0)
                .interact()?;

            let selected_type = match selection {
                0 => MakeTargetType::Folder,
                1 => MakeTargetType::File,
                _ => {
                    println!("Cancelled.");
                    return Ok(());
                }
            };

            let prompt_text = match selected_type {
                MakeTargetType::Folder => "Folder path (leave blank to cancel)",
                MakeTargetType::File => "File path (leave blank to cancel)",
            };
            let input: String = Input::with_theme(theme)
                .with_prompt(prompt_text)
                .allow_empty(true)
                .interact_text()?;
            let trimmed = input.trim();
            if trimmed.is_empty() {
                println!("Cancelled.");
                return Ok(());
            }
            (selected_type, PathBuf::from(trimmed))
        }
        (Some(first), Some(second)) => {
            let first_lower = first.to_lowercase();
            let selected_type =
                if first_lower == "folder" || first_lower == "dir" || first_lower == "directory" {
                    MakeTargetType::Folder
                } else if first_lower == "file" {
                    MakeTargetType::File
                } else if force_folder {
                    MakeTargetType::Folder
                } else if force_file {
                    MakeTargetType::File
                } else {
                    MakeTargetType::Folder
                };
            (selected_type, second)
        }
        (Some(first), None) => {
            let first_lower = first.to_lowercase();
            if first_lower == "folder" || first_lower == "dir" || first_lower == "directory" {
                let input: String = Input::with_theme(theme)
                    .with_prompt("Folder path (leave blank to cancel)")
                    .allow_empty(true)
                    .interact_text()?;
                let trimmed = input.trim();
                if trimmed.is_empty() {
                    println!("Cancelled.");
                    return Ok(());
                }
                (MakeTargetType::Folder, PathBuf::from(trimmed))
            } else if first_lower == "file" {
                let input: String = Input::with_theme(theme)
                    .with_prompt("File path (leave blank to cancel)")
                    .allow_empty(true)
                    .interact_text()?;
                let trimmed = input.trim();
                if trimmed.is_empty() {
                    println!("Cancelled.");
                    return Ok(());
                }
                (MakeTargetType::File, PathBuf::from(trimmed))
            } else {
                let path = PathBuf::from(&first);
                let selected_type = if force_folder {
                    MakeTargetType::Folder
                } else if force_file {
                    MakeTargetType::File
                } else if first.ends_with('/') || first.ends_with('\\') {
                    MakeTargetType::Folder
                } else {
                    let invoked_as = std::env::args().nth(1).unwrap_or_default();
                    if invoked_as == "touch" {
                        MakeTargetType::File
                    } else if invoked_as == "mkdir" {
                        MakeTargetType::Folder
                    } else if path.extension().is_some() {
                        MakeTargetType::File
                    } else {
                        let cancel_btn = cancel_option();
                        let options = ["Folder", "File", cancel_btn.as_str()];
                        let selection = Select::with_theme(theme)
                            .with_prompt(format!("Create '{}' as", first))
                            .items(&options)
                            .default(0)
                            .interact()?;
                        match selection {
                            0 => MakeTargetType::Folder,
                            1 => MakeTargetType::File,
                            _ => {
                                println!("Cancelled.");
                                return Ok(());
                            }
                        }
                    }
                };
                (selected_type, path)
            }
        }
        (None, Some(second)) => (MakeTargetType::Folder, second),
    };

    match target_type {
        MakeTargetType::Folder => make_directory(&target_path),
        MakeTargetType::File => create_empty_file(&target_path),
    }
}

/// Scans common macOS application directories for installed applications.
fn scan_installed_apps() -> Vec<String> {
    let mut apps = HashSet::new();
    let mut search_paths = vec![
        PathBuf::from("/Applications"),
        PathBuf::from("/System/Applications"),
        PathBuf::from("/System/Applications/Utilities"),
    ];

    if let Some(home) = std::env::var_os("HOME") {
        search_paths.push(PathBuf::from(home).join("Applications"));
    }

    for dir in search_paths {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("app")
                    && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
                {
                    apps.insert(stem.to_string());
                }
            }
        }
    }

    let mut list: Vec<String> = apps.into_iter().collect();
    list.sort_by_key(|a| a.to_lowercase());
    list
}

/// Find matching installed applications by exact, prefix, substring, or fuzzy typo matching.
fn find_matching_apps(query: &str, installed: &[String]) -> Vec<String> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return Vec::new();
    }

    // 1. Exact case-insensitive match
    if let Some(exact) = installed.iter().find(|a| a.to_lowercase() == q) {
        return vec![exact.clone()];
    }

    // 2. Prefix matches (e.g. "what" -> "WhatsApp")
    let prefix_matches: Vec<String> = installed
        .iter()
        .filter(|a| a.to_lowercase().starts_with(&q))
        .cloned()
        .collect();

    if !prefix_matches.is_empty() {
        return prefix_matches;
    }

    // 3. Substring matches (e.g. "code" -> "Visual Studio Code")
    let substr_matches: Vec<String> = installed
        .iter()
        .filter(|a| a.to_lowercase().contains(&q))
        .cloned()
        .collect();

    if !substr_matches.is_empty() {
        return substr_matches;
    }

    // 4. Fuzzy Levenshtein distance matches (e.g. "whasapp", "watsap")
    let mut fuzzy_candidates: Vec<(&String, usize)> = installed
        .iter()
        .map(|a| {
            let dist = levenshtein_distance(&q, &a.to_lowercase());
            (a, dist)
        })
        .filter(|(_, dist)| *dist <= 3)
        .collect();

    fuzzy_candidates.sort_by_key(|(_, dist)| *dist);
    fuzzy_candidates
        .into_iter()
        .take(5)
        .map(|(a, _)| a.clone())
        .collect()
}

/// Handles opening macOS applications with smart fuzzy matching and typo resolution.
fn handle_open(theme: &ColorfulTheme, target: Option<String>) -> Result<()> {
    let installed = scan_installed_apps();

    let app_name = match target {
        Some(name) => name,
        None => {
            if installed.is_empty() {
                let input: String = Input::with_theme(theme)
                    .with_prompt("Application name (leave blank to cancel)")
                    .allow_empty(true)
                    .interact_text()?;
                let trimmed = input.trim().to_string();
                if trimmed.is_empty() {
                    println!("Cancelled.");
                    return Ok(());
                }
                trimmed
            } else {
                let mut menu_items = installed.clone();
                menu_items.push(cancel_option());

                ui::print_key_hints();
                let selection = FuzzySelect::with_theme(theme)
                    .with_prompt("Select application to open (type to filter)")
                    .items(&menu_items)
                    .default(0)
                    .interact()?;

                if selection >= installed.len() {
                    println!("Cancelled.");
                    return Ok(());
                }
                menu_items[selection].clone()
            }
        }
    };

    // 1. Try launching with the exact name given
    let direct_status = Command::new("open").arg("-a").arg(&app_name).status();
    if direct_status.map(|s| s.success()).unwrap_or(false) {
        println!(
            "{}",
            format!("Opened application: {app_name}").green().bold()
        );
        return Ok(());
    }

    // 2. If direct launch fails, resolve via smart matching (exact, prefix, substring, Levenshtein)
    let matches = find_matching_apps(&app_name, &installed);

    if matches.is_empty() {
        bail!("application '{app_name}' was not found in /Applications");
    } else if matches.len() == 1 {
        let target_app = &matches[0];
        println!(
            "{}",
            format!("Matched application '{target_app}'. Launching...").dimmed()
        );
        let status = Command::new("open")
            .arg("-a")
            .arg(target_app)
            .status()
            .with_context(|| format!("failed to launch application '{target_app}'"))?;
        if !status.success() {
            bail!("failed to launch application '{target_app}'");
        }
        println!(
            "{}",
            format!("Opened application: {target_app}").green().bold()
        );
    } else {
        let mut menu_items = matches.clone();
        menu_items.push(cancel_option());

        let prompt = format!("Unknown application '{app_name}'. Did you mean:");
        let selection = Select::with_theme(theme)
            .with_prompt(prompt)
            .items(&menu_items)
            .default(0)
            .interact()?;

        if selection >= matches.len() {
            println!("Cancelled.");
            return Ok(());
        }

        let selected_app = &matches[selection];
        let status = Command::new("open")
            .arg("-a")
            .arg(selected_app)
            .status()
            .with_context(|| format!("failed to launch application '{selected_app}'"))?;
        if !status.success() {
            bail!("failed to launch application '{selected_app}'");
        }
        println!(
            "{}",
            format!("Opened application: {selected_app}").green().bold()
        );
    }

    Ok(())
}

/// Handles copying items with interactive prompts when arguments are missing.
fn handle_copy(
    theme: &ColorfulTheme,
    source: Option<PathBuf>,
    destination: Option<PathBuf>,
) -> Result<()> {
    let src = match source {
        Some(path) => path,
        None => {
            let input: String = Input::with_theme(theme)
                .with_prompt("Source path (leave blank to cancel)")
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

    let dst = match destination {
        Some(path) => path,
        None => {
            let input: String = Input::with_theme(theme)
                .with_prompt("Destination path (leave blank to cancel)")
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

    copy_item(&src, &dst)
}

/// Handles moving or renaming items with interactive prompts when arguments are missing.
fn handle_move(
    theme: &ColorfulTheme,
    source: Option<PathBuf>,
    destination: Option<PathBuf>,
) -> Result<()> {
    let src = match source {
        Some(path) => path,
        None => {
            let input: String = Input::with_theme(theme)
                .with_prompt("Source path (leave blank to cancel)")
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

    let dst = match destination {
        Some(path) => path,
        None => {
            let input: String = Input::with_theme(theme)
                .with_prompt("Destination path (leave blank to cancel)")
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

    move_item(&src, &dst)
}

/// Handles file or folder deletion with a safe confirmation dialog.
fn handle_del(theme: &ColorfulTheme, target: Option<PathBuf>) -> Result<()> {
    let path = match target {
        Some(p) => p,
        None => {
            let input: String = Input::with_theme(theme)
                .with_prompt("Path to delete (leave blank to cancel)")
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

    if !path.exists() {
        bail!("path does not exist: {}", path.display());
    }

    let item_kind = if path.is_dir() { "directory" } else { "file" };
    let prompt = format!("Delete {item_kind} '{}'?", path.display());

    let confirmation = Confirm::with_theme(theme)
        .with_prompt(prompt)
        .default(false)
        .interact()?;

    if !confirmation {
        println!("Cancelled.");
        return Ok(());
    }

    delete_item(&path)
}

/// Creates a nested directory tree using fs::create_dir_all.
fn make_directory(path: &Path) -> Result<()> {
    fs::create_dir_all(path)
        .with_context(|| format!("failed to create directory '{}'", path.display()))?;

    println!("Created directory: {}", path.display());
    Ok(())
}

/// Creates a new empty file and ensures the parent directory exists.
fn create_empty_file(path: &Path) -> Result<()> {
    ensure_parent_exists(path)?;

    if path.exists() {
        println!("File already exists: {}", path.display());
    } else {
        fs::File::create(path)
            .with_context(|| format!("failed to create file '{}'", path.display()))?;
        println!("Created file: {}", path.display());
    }
    Ok(())
}

/// Copies a single file or recursively copies an entire directory tree.
fn copy_item(source: &Path, destination: &Path) -> Result<()> {
    if !source.exists() {
        bail!("source does not exist: {}", source.display());
    }

    if source.is_dir() {
        copy_directory_recursive(source, destination)?;
    } else {
        ensure_parent_exists(destination)?;
        fs::copy(source, destination).with_context(|| {
            format!(
                "failed to copy file from '{}' to '{}'",
                source.display(),
                destination.display()
            )
        })?;
    }

    println!(
        "Copied '{}' to '{}'",
        source.display(),
        destination.display()
    );
    Ok(())
}

/// Recursively copies a directory tree to a target destination.
fn copy_directory_recursive(source: &Path, destination: &Path) -> Result<()> {
    fs::create_dir_all(destination).with_context(|| {
        format!(
            "failed to create destination directory '{}'",
            destination.display()
        )
    })?;

    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let entry_type = entry.file_type()?;
        let target_path = destination.join(entry.file_name());

        if entry_type.is_dir() {
            copy_directory_recursive(&entry.path(), &target_path)?;
        } else {
            fs::copy(entry.path(), &target_path)?;
        }
    }

    Ok(())
}

/// Moves or renames a file or directory to a target location.
fn move_item(source: &Path, destination: &Path) -> Result<()> {
    if !source.exists() {
        bail!("source does not exist: {}", source.display());
    }

    ensure_parent_exists(destination)?;

    fs::rename(source, destination).with_context(|| {
        format!(
            "failed to move '{}' to '{}'",
            source.display(),
            destination.display()
        )
    })?;

    println!(
        "Moved '{}' to '{}'",
        source.display(),
        destination.display()
    );
    Ok(())
}

/// Deletes a single file or an entire directory tree.
fn delete_item(path: &Path) -> Result<()> {
    let item_kind = if path.is_dir() {
        fs::remove_dir_all(path)
            .with_context(|| format!("failed to delete directory '{}'", path.display()))?;
        "directory"
    } else {
        fs::remove_file(path)
            .with_context(|| format!("failed to delete file '{}'", path.display()))?;
        "file"
    };

    println!("Deleted {item_kind}: {}", path.display());
    Ok(())
}

/// Ensures the parent directory of a path exists before file operations.
fn ensure_parent_exists(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
        && !parent.exists()
    {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create parent directory '{}'", parent.display()))?;
    }
    Ok(())
}

/// Clears the terminal screen.
fn clear_terminal() -> Result<()> {
    if Command::new("clear").status().is_ok() {
        return Ok(());
    }

    print!("\x1B[2J\x1B[1;1H");
    std::io::stdout()
        .flush()
        .context("failed to flush stdout")?;

    Ok(())
}

/// Handles smart directory navigation to root, back, fuzzy-searched subfolders, or via interactive menu.
fn handle_go(theme: &ColorfulTheme, target: Option<&str>) -> Result<()> {
    let current_dir =
        std::env::current_dir().context("failed to read current working directory")?;
    let home_dir = std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/"));

    let target_path = match target {
        Some("root") | Some("~") => home_dir,
        Some("back") | Some("..") => current_dir.parent().unwrap_or(&current_dir).to_path_buf(),
        Some(query) => {
            let candidates = find_matching_directories(&current_dir, &home_dir, query);

            if candidates.is_empty() {
                bail!("no directory matching '{query}' found");
            } else if candidates.len() == 1 {
                candidates.into_iter().next().unwrap()
            } else {
                let q_lower = query.to_lowercase();
                let top_name = candidates[0]
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                let second_name = candidates
                    .get(1)
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_lowercase();

                if top_name == q_lower && second_name != q_lower {
                    candidates.into_iter().next().unwrap()
                } else {
                    let term = dialoguer::console::Term::stderr();
                    let home_str = home_dir.to_string_lossy();

                    let mut display_items: Vec<String> = candidates
                        .iter()
                        .take(15)
                        .map(|p| {
                            let p_str = p.to_string_lossy();
                            if p_str.starts_with(home_str.as_ref()) {
                                format!("~{}", &p_str[home_str.len()..])
                            } else {
                                p_str.to_string()
                            }
                        })
                        .collect();
                    display_items.push(cancel_option());

                    let prompt = format!("Multiple folders match '{query}'. Select target:");
                    ui::print_key_hints();
                    let selection = FuzzySelect::with_theme(theme)
                        .with_prompt(prompt)
                        .items(&display_items)
                        .default(0)
                        .interact_on(&term)?;

                    if selection == display_items.len() - 1 {
                        println!("Cancelled.");
                        return Ok(());
                    }

                    candidates[selection].clone()
                }
            }
        }
        None => {
            let term = dialoguer::console::Term::stderr();
            let mut options = Vec::new();
            let mut paths = Vec::new();

            options.push("~ (Home directory)".to_string());
            paths.push(home_dir);

            if let Some(parent) = current_dir.parent() {
                options.push(".. (Parent directory)".to_string());
                paths.push(parent.to_path_buf());
            }

            if let Ok(entries) = fs::read_dir(&current_dir) {
                let mut dirs = Vec::new();
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir()
                        && let Some(name) = path.file_name().and_then(|n| n.to_str())
                        && !name.starts_with('.')
                    {
                        dirs.push((name.to_string(), path));
                    }
                }
                dirs.sort_by_key(|a| a.0.to_lowercase());
                for (name, path) in dirs {
                    options.push(format!("{name}/"));
                    paths.push(path);
                }
            }
            options.push(cancel_option());

            ui::print_key_hints();
            let selection = FuzzySelect::with_theme(theme)
                .with_prompt("Select target folder (type to filter)")
                .items(&options)
                .default(0)
                .interact_on(&term)?;

            if selection == options.len() - 1 {
                println!("Cancelled.");
                return Ok(());
            }

            paths[selection].clone()
        }
    };

    let is_shell_resolve = std::env::var("RUN_SHELL_RESOLVE")
        .map(|v| v == "1")
        .unwrap_or(false);

    if is_shell_resolve {
        println!("{}", target_path.display());
    } else {
        println!("Target directory: {}", target_path.display());
        eprintln!(
            "Notice: Run 'source ~/.zshrc' in this terminal tab to activate in-place directory switching."
        );
    }

    Ok(())
}

/// Searches for directories matching the query using fuzzy and substring scoring across current dir and system hubs.
fn find_matching_directories(current_dir: &Path, home_dir: &Path, query: &str) -> Vec<PathBuf> {
    let q_lower = query.trim().to_lowercase();
    let mut scored: Vec<(f64, PathBuf)> = Vec::new();
    let mut visited: HashSet<PathBuf> = HashSet::new();

    let mut search_roots: Vec<(PathBuf, usize)> = Vec::new();
    search_roots.push((current_dir.to_path_buf(), 2));

    let hubs = [
        "Developer",
        "Downloads",
        "Documents",
        "Desktop",
        "Projects",
        "Pictures",
        "Code",
    ];

    for hub in &hubs {
        let hub_path = home_dir.join(hub);
        if hub_path.is_dir() {
            if visited.insert(hub_path.clone()) {
                let score = score_name(hub, &q_lower, false, true, 0.0);
                if score > 0.0 {
                    scored.push((score, hub_path.clone()));
                }
            }
            search_roots.push((hub_path, 2));
        }
    }

    let ignored_names = [
        "Library",
        ".Trash",
        ".git",
        "node_modules",
        "target",
        ".cargo",
        ".rustup",
        ".gemini",
        ".vscode",
        ".npm",
        ".cache",
        ".local",
        "venv",
        ".venv",
    ];

    let mut ctx = SearchContext {
        q_lower: &q_lower,
        current_dir,
        ignored_names: &ignored_names,
        visited: &mut visited,
        scored: &mut scored,
    };

    for (root, max_depth) in search_roots {
        scan_for_query(&root, max_depth, 0, &mut ctx);
    }

    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    scored.into_iter().map(|(_, path)| path).collect()
}

fn score_name(
    name: &str,
    q_lower: &str,
    is_current_sub: bool,
    is_hub: bool,
    depth_penalty: f64,
) -> f64 {
    let name_lower = name.to_lowercase();
    let mut score = 0.0;

    if name_lower == q_lower {
        score = 1.0;
    } else if name_lower.starts_with(q_lower) {
        score = 0.90;
    } else if name_lower.contains(q_lower) {
        score = 0.80;
    } else if q_lower.len() >= 3 && name_lower.len() >= 3 {
        let similarity = strsim::jaro_winkler(&name_lower, q_lower);
        if similarity >= 0.82 {
            score = similarity * 0.75;
        }
    }

    if score > 0.0 {
        if is_hub {
            score += 0.08;
        }
        if is_current_sub {
            score += 0.05;
        }
        score -= depth_penalty;
    }

    score
}

struct SearchContext<'a> {
    q_lower: &'a str,
    current_dir: &'a Path,
    ignored_names: &'a [&'a str],
    visited: &'a mut HashSet<PathBuf>,
    scored: &'a mut Vec<(f64, PathBuf)>,
}

fn scan_for_query(dir: &Path, depth: usize, depth_from_root: usize, ctx: &mut SearchContext) {
    if depth == 0 {
        return;
    }

    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n,
            None => continue,
        };

        if name.starts_with('.') || ctx.ignored_names.contains(&name) {
            continue;
        }

        if ctx.visited.insert(path.clone()) {
            let is_current_sub = path.starts_with(ctx.current_dir);
            let depth_penalty = depth_from_root as f64 * 0.04;
            let score = score_name(name, ctx.q_lower, is_current_sub, false, depth_penalty);

            if score > 0.0 {
                ctx.scored.push((score, path.clone()));
            }

            scan_for_query(&path, depth - 1, depth_from_root + 1, ctx);
        }
    }
}

/// Generates the shell integration wrapper function for zsh and bash.
fn handle_init() -> Result<()> {
    println!(
        r#"run() {{
    if [ "$1" = "go" ] || [ "$1" = "jmp" ] || [ "$1" = "nav" ] || [ "$1" = "cd" ] || [ "$1" = "g" ] || [ "$1" = "project" ] || [ "$1" = "prj" ] || [ "$1" = "pro" ]; then
        local target
        target="$(RUN_SHELL_RESOLVE=1 command run "$@")" || return $?
        if [ -n "$target" ] && [ -d "$target" ]; then
            cd "$target"
        fi
    else
        command run "$@"
    fi
}}
if [ -n "$ZSH_VERSION" ]; then
    source <(command run completion zsh 2>/dev/null)
elif [ -n "$BASH_VERSION" ]; then
    source <(command run completion bash 2>/dev/null)
fi"#
    );
    Ok(())
}

/// Handles smart project scanning, contextual project selection, and interactive IDE launching.
fn handle_project(theme: &ColorfulTheme, target: Option<&str>) -> Result<()> {
    let current_dir =
        std::env::current_dir().context("failed to read current working directory")?;
    let home_dir = std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/"));

    let target_path = match target {
        Some(".") => current_dir,
        Some(t) => {
            let candidate_path = PathBuf::from(t);
            if candidate_path.exists() {
                if candidate_path.is_absolute() {
                    candidate_path
                } else {
                    current_dir.join(candidate_path)
                }
            } else {
                let scanned = scan_projects(&home_dir, &current_dir);
                let q_lower = t.to_lowercase();
                let matches: Vec<PathBuf> = scanned
                    .into_iter()
                    .filter(|p| {
                        let name = p
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("")
                            .to_lowercase();
                        name.contains(&q_lower)
                            || p.to_string_lossy().to_lowercase().contains(&q_lower)
                    })
                    .collect();

                if matches.is_empty() {
                    bail!("no project matching '{t}' found");
                } else if matches.len() == 1 {
                    matches.into_iter().next().unwrap()
                } else {
                    let term = dialoguer::console::Term::stderr();
                    let home_str = home_dir.to_string_lossy();
                    let mut display_items: Vec<String> = matches
                        .iter()
                        .take(20)
                        .map(|p| {
                            let p_str = p.to_string_lossy();
                            if p_str.starts_with(home_str.as_ref()) {
                                format!("~{}", &p_str[home_str.len()..])
                            } else {
                                p_str.to_string()
                            }
                        })
                        .collect();
                    display_items.push(cancel_option());

                    let prompt = format!("Multiple projects match '{t}'. Select target:");
                    let selection = Select::with_theme(theme)
                        .with_prompt(prompt)
                        .items(&display_items)
                        .default(0)
                        .interact_on(&term)?;

                    if selection == display_items.len() - 1 {
                        println!("Cancelled.");
                        return Ok(());
                    }

                    matches[selection].clone()
                }
            }
        }
        None => {
            let term = dialoguer::console::Term::stderr();
            let mut sp = ui::Spinner::start("Scanning development hubs for projects...");
            let scanned = scan_projects(&home_dir, &current_dir);
            sp.stop();
            if scanned.is_empty() {
                bail!("no projects found in common development hubs");
            }

            let home_str = home_dir.to_string_lossy();
            let mut display_items: Vec<String> = scanned
                .iter()
                .map(|p| {
                    let p_str = p.to_string_lossy();
                    let short_path = if p_str.starts_with(home_str.as_ref()) {
                        format!("~{}", &p_str[home_str.len()..])
                    } else {
                        p_str.to_string()
                    };
                    let badge = ui::detect_stack_badge(p);
                    format!("{:<38} {}", short_path, badge)
                })
                .collect();
            display_items.push(cancel_option());

            ui::print_key_hints();
            let selection = FuzzySelect::with_theme(theme)
                .with_prompt("Select project (type to filter)")
                .items(&display_items)
                .default(0)
                .interact_on(&term)?;

            if selection == display_items.len() - 1 {
                println!("Cancelled.");
                return Ok(());
            }

            scanned[selection].clone()
        }
    };

    let canonical = target_path.canonicalize().unwrap_or(target_path);

    let is_shell_resolve = std::env::var("RUN_SHELL_RESOLVE")
        .map(|v| v == "1")
        .unwrap_or(false);

    let cfg = config::load_config();
    if let Some(ref ide) = cfg.default_ide {
        let ide_clean = ide.trim().to_lowercase();
        if ide_clean != "ask" && !ide_clean.is_empty() {
            match ide_clean.as_str() {
                "antigravity" | "agy" => return open_in_antigravity(&canonical, is_shell_resolve),
                "cursor" => return open_in_cursor(&canonical, is_shell_resolve),
                "vscode" | "code" => return open_in_vscode(&canonical, is_shell_resolve),
                "xcode" => return open_in_xcode(&canonical, is_shell_resolve),
                "terminal" | "none" => {
                    if is_shell_resolve {
                        println!("{}", canonical.display());
                    } else {
                        println!(
                            "{}",
                            format!("Target directory: {}", canonical.display()).green()
                        );
                        eprintln!(
                            "Notice: Run 'source ~/.zshrc' in this terminal tab to activate in-place directory switching."
                        );
                    }
                    return Ok(());
                }
                _ => {}
            }
        }
    }

    let cancel_btn = cancel_option();
    let ide_options = [
        "Visual Studio Code (code)",
        "Cursor (cursor)",
        "Antigravity IDE (antigravity)",
        "Xcode (xcode)",
        "Switch Terminal Directory Only",
        cancel_btn.as_str(),
    ];

    let term = dialoguer::console::Term::stderr();
    let project_name = canonical
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project");

    ui::render_breadcrumbs(&["run", "project", project_name, "Select Action"]);
    let prompt = format!("Open '{project_name}' with:");

    let ide_selection = Select::with_theme(theme)
        .with_prompt(prompt)
        .items(&ide_options)
        .default(0)
        .interact_on(&term)?;

    match ide_selection {
        0 => open_in_vscode(&canonical, is_shell_resolve)?,
        1 => open_in_cursor(&canonical, is_shell_resolve)?,
        2 => open_in_antigravity(&canonical, is_shell_resolve)?,
        3 => open_in_xcode(&canonical, is_shell_resolve)?,
        4 => {
            if is_shell_resolve {
                println!("{}", canonical.display());
            } else {
                println!(
                    "{}",
                    format!("Target directory: {}", canonical.display()).green()
                );
                eprintln!(
                    "Notice: Run 'source ~/.zshrc' in this terminal tab to activate in-place directory switching."
                );
            }
        }
        _ => {
            println!("Cancelled.");
        }
    }

    Ok(())
}

/// Opens project directory in Google Antigravity IDE.
fn open_in_antigravity(path: &Path, is_shell_resolve: bool) -> Result<()> {
    let status = Command::new("antigravity").arg(path).status();
    let success = match status {
        Ok(s) => s.success(),
        Err(_) => false,
    };

    let success = if !success {
        match Command::new("agy").arg(path).status() {
            Ok(s) => s.success(),
            Err(_) => false,
        }
    } else {
        true
    };

    if !success {
        let fallback = Command::new("open")
            .arg("-a")
            .arg("Antigravity IDE")
            .arg(path)
            .status();
        let fallback_ok = match fallback {
            Ok(s) => s.success(),
            Err(_) => false,
        };

        if !fallback_ok {
            let alt_fallback = Command::new("open")
                .arg("-a")
                .arg("Antigravity")
                .arg(path)
                .status()
                .with_context(|| "failed to launch Antigravity IDE via open -a")?;
            if !alt_fallback.success() {
                bail!(
                    "failed to launch Antigravity IDE (ensure Antigravity IDE is installed in /Applications)"
                );
            }
        }
    }

    let msg = format!("Opened project in Antigravity IDE: {}", path.display());
    if is_shell_resolve {
        eprintln!("{}", msg.green());
    } else {
        println!("{}", msg.green());
    }
    Ok(())
}

/// Opens project directory in Visual Studio Code.
fn open_in_vscode(path: &Path, is_shell_resolve: bool) -> Result<()> {
    let status = Command::new("code").arg(path).status();
    let success = match status {
        Ok(s) => s.success(),
        Err(_) => false,
    };

    if !success {
        let fallback = Command::new("open")
            .arg("-a")
            .arg("Visual Studio Code")
            .arg(path)
            .status()
            .with_context(|| "failed to launch Visual Studio Code via open -a")?;
        if !fallback.success() {
            bail!("failed to launch Visual Studio Code");
        }
    }

    let msg = format!("Opened project in Visual Studio Code: {}", path.display());
    if is_shell_resolve {
        eprintln!("{}", msg.green());
    } else {
        println!("{}", msg.green());
    }
    Ok(())
}

/// Opens project directory in Cursor.
fn open_in_cursor(path: &Path, is_shell_resolve: bool) -> Result<()> {
    let status = Command::new("cursor").arg(path).status();
    let success = match status {
        Ok(s) => s.success(),
        Err(_) => false,
    };

    if !success {
        let fallback = Command::new("open")
            .arg("-a")
            .arg("Cursor")
            .arg(path)
            .status()
            .with_context(|| "failed to launch Cursor via open -a")?;
        if !fallback.success() {
            bail!("failed to launch Cursor (ensure Cursor is installed in /Applications)");
        }
    }

    let msg = format!("Opened project in Cursor: {}", path.display());
    if is_shell_resolve {
        eprintln!("{}", msg.green());
    } else {
        println!("{}", msg.green());
    }
    Ok(())
}

/// Opens project directory in Xcode (opens .xcworkspace or .xcodeproj if present).
fn open_in_xcode(path: &Path, is_shell_resolve: bool) -> Result<()> {
    let mut xcode_file = None;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
                if ext == "xcworkspace" {
                    xcode_file = Some(p);
                    break;
                } else if ext == "xcodeproj" && xcode_file.is_none() {
                    xcode_file = Some(p);
                }
            }
        }
    }

    let status = if let Some(target) = xcode_file {
        Command::new("open").arg(target).status()
    } else {
        Command::new("open")
            .arg("-a")
            .arg("Xcode")
            .arg(path)
            .status()
    }
    .with_context(|| "failed to launch Xcode")?;

    if !status.success() {
        bail!("failed to launch Xcode");
    }

    let msg = format!("Opened project in Xcode: {}", path.display());
    if is_shell_resolve {
        eprintln!("{}", msg.green());
    } else {
        println!("{}", msg.green());
    }
    Ok(())
}

/// Scans standard development directories for project roots.
fn scan_projects(home_dir: &Path, current_dir: &Path) -> Vec<PathBuf> {
    let mut projects = Vec::new();
    let mut visited = HashSet::new();

    let cfg = config::load_config();
    let mut candidate_hubs = vec![
        home_dir.join("Developer"),
        home_dir.join("Projects"),
        home_dir.join("Code"),
        home_dir.join("Documents"),
        home_dir.join("Desktop"),
    ];

    if let Some(custom) = cfg.custom_hubs {
        for hub in custom {
            let p = if hub.to_string_lossy().starts_with("~/") {
                home_dir.join(&hub.to_string_lossy()[2..])
            } else {
                hub
            };
            if !candidate_hubs.contains(&p) {
                candidate_hubs.push(p);
            }
        }
    }

    let ignored_names = [
        "Library",
        ".Trash",
        ".git",
        "node_modules",
        "target",
        ".cargo",
        ".rustup",
        ".gemini",
        ".vscode",
        ".npm",
        ".cache",
        ".local",
        "venv",
        ".venv",
        "Pods",
        "DerivedData",
        ".build",
        ".next",
        "dist",
        "build",
    ];

    for hub in &candidate_hubs {
        if hub.is_dir() {
            scan_dir_for_projects(hub, 3, &ignored_names, &mut visited, &mut projects);
        }
    }

    if visited.insert(current_dir.to_path_buf()) && is_project_directory(current_dir) {
        projects.push(current_dir.to_path_buf());
    }

    projects.sort_by(|a, b| {
        a.to_string_lossy()
            .to_lowercase()
            .cmp(&b.to_string_lossy().to_lowercase())
    });
    projects
}

fn scan_dir_for_projects(
    dir: &Path,
    depth: usize,
    ignored_names: &[&str],
    visited: &mut HashSet<PathBuf>,
    projects: &mut Vec<PathBuf>,
) {
    if depth == 0 {
        return;
    }

    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n,
            None => continue,
        };

        if name.starts_with('.') || ignored_names.contains(&name) {
            continue;
        }

        if visited.insert(path.clone()) {
            if is_project_directory(&path) {
                projects.push(path.clone());
            }
            scan_dir_for_projects(&path, depth - 1, ignored_names, visited, projects);
        }
    }
}

/// Checks if a directory contains project signature markers.
fn is_project_directory(path: &Path) -> bool {
    let markers = [
        ".git",
        "Cargo.toml",
        "package.json",
        "pubspec.yaml",
        "go.mod",
        "pom.xml",
        "build.gradle",
        "build.gradle.kts",
        "Makefile",
        "pyproject.toml",
        "requirements.txt",
        "Package.swift",
        "CMakeLists.txt",
    ];

    for marker in &markers {
        if path.join(marker).exists() {
            return true;
        }
    }

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Some(ext) = entry.path().extension().and_then(|s| s.to_str())
                && (ext == "xcodeproj" || ext == "xcworkspace")
            {
                return true;
            }
        }
    }

    false
}

/// Runs development server for current active project based on detected signature.
fn handle_dev() -> Result<()> {
    let current_dir =
        std::env::current_dir().context("failed to read current working directory")?;

    let (cmd, args, label) = if current_dir.join("Cargo.toml").exists() {
        ("cargo", vec!["run"], "cargo run")
    } else if current_dir.join("package.json").exists() {
        if current_dir.join("pnpm-lock.yaml").exists() {
            ("pnpm", vec!["run", "dev"], "pnpm run dev")
        } else if current_dir.join("yarn.lock").exists() {
            ("yarn", vec!["dev"], "yarn dev")
        } else if current_dir.join("bun.lockb").exists() || current_dir.join("bun.lock").exists() {
            ("bun", vec!["run", "dev"], "bun run dev")
        } else {
            ("npm", vec!["run", "dev"], "npm run dev")
        }
    } else if current_dir.join("pubspec.yaml").exists() {
        ("flutter", vec!["run"], "flutter run")
    } else if current_dir.join("go.mod").exists() {
        ("go", vec!["run", "."], "go run .")
    } else if current_dir.join("Makefile").exists() {
        ("make", vec!["dev"], "make dev")
    } else {
        bail!(
            "no recognized project configuration found in current directory (e.g., Cargo.toml, package.json)"
        );
    };

    println!(
        "{}",
        format!("Starting development server ({label})...")
            .truecolor(0, 162, 255)
            .bold()
    );

    let status = Command::new(cmd)
        .args(&args)
        .status()
        .with_context(|| format!("failed to start development server using '{label}'"))?;

    if !status.success() {
        let code = status.code().unwrap_or(1);
        bail!("development server exited with error code {code}");
    }

    Ok(())
}

/// Builds or compiles the current active project based on detected signature.
fn handle_build() -> Result<()> {
    let current_dir =
        std::env::current_dir().context("failed to read current working directory")?;

    let (cmd, args, label) = if current_dir.join("Cargo.toml").exists() {
        ("cargo", vec!["build", "--release"], "cargo build --release")
    } else if current_dir.join("package.json").exists() {
        if current_dir.join("pnpm-lock.yaml").exists() {
            ("pnpm", vec!["run", "build"], "pnpm run build")
        } else if current_dir.join("yarn.lock").exists() {
            ("yarn", vec!["build"], "yarn build")
        } else if current_dir.join("bun.lockb").exists() || current_dir.join("bun.lock").exists() {
            ("bun", vec!["run", "build"], "bun run build")
        } else {
            ("npm", vec!["run", "build"], "npm run build")
        }
    } else if current_dir.join("pubspec.yaml").exists() {
        ("flutter", vec!["build"], "flutter build")
    } else if current_dir.join("go.mod").exists() {
        ("go", vec!["build", "."], "go build .")
    } else if current_dir.join("Makefile").exists() {
        ("make", vec!["build"], "make build")
    } else {
        bail!(
            "no recognized project configuration found in current directory (e.g., Cargo.toml, package.json)"
        );
    };

    println!(
        "{}",
        format!("Building project ({label})...")
            .truecolor(0, 162, 255)
            .bold()
    );

    let status = Command::new(cmd)
        .args(&args)
        .status()
        .with_context(|| format!("failed to execute '{label}'"))?;

    if !status.success() {
        let code = status.code().unwrap_or(1);
        bail!("build failed with exit code {code}");
    }

    println!("{}", "Build completed successfully!".green().bold());
    Ok(())
}

fn electric_blue(text: &str) -> colored::ColoredString {
    ui::primary_colored(text)
}

/// Creates a customized dialoguer theme respecting user's primary color and red Cancel.
fn custom_theme() -> ColorfulTheme {
    ui::custom_theme()
}

/// Displays beautifully formatted terminal help reference and command details.
fn handle_help(target_command: Option<&str>) -> Result<()> {
    match target_command {
        None => print_main_help(),
        Some(cmd) => print_command_detail(cmd.trim()),
    }
    Ok(())
}

fn print_main_help() {
    println!();
    println!(
        "{} {}",
        "run".bold().white(),
        electric_blue("Productivity CLI utility for macOS").dimmed()
    );
    println!();
    println!("{}", electric_blue("USAGE:").bold());
    println!("  run <command> [arguments]");
    println!("  run <alias>   [arguments]");
    println!();
    println!("{}", electric_blue("WORKSPACE & DEV:").bold());
    print_cmd_summary("project", "prj", "Scan projects & open in IDE or terminal");
    print_cmd_summary("dev", "dev", "Run active project dev server");
    print_cmd_summary("build", "bld", "Build or compile active project");
    print_cmd_summary(
        "test",
        "tst",
        "Smart polyglot test runner (Rust, Node, Python, etc.)",
    );
    print_cmd_summary(
        "clean",
        "cln",
        "Clean disposable build artifacts and cache folders",
    );
    print_cmd_summary(
        "sync",
        "snc (git)",
        "Interactive 1-step Git pull, commit, and push",
    );
    print_cmd_summary(
        "docker",
        "dck",
        "Inspect & manage Docker/OrbStack containers & logs",
    );
    print_cmd_summary(
        "secret",
        "sec (dotenv)",
        "Audit & sync .env and .env.example environment variables",
    );
    print_cmd_summary(
        "network",
        "net (ip)",
        "Inspect local LAN and public IP with quick copy",
    );
    print_cmd_summary("share", "shr", "Instant local LAN HTTP file server");
    print_cmd_summary(
        "bench",
        "bnc",
        "Benchmark command execution duration & performance",
    );
    println!();
    println!("{}", electric_blue("FILESYSTEM & NAVIGATION:").bold());
    print_cmd_summary("go", "jmp (cd)", "Smart directory navigation & hub jump");
    print_cmd_summary(
        "path",
        "pth (pwd)",
        "Print or copy current working directory",
    );
    print_cmd_summary(
        "list",
        "lst (ls)",
        "List directory contents with formatted sizes",
    );
    print_cmd_summary(
        "make",
        "mak (touch, mkdir)",
        "Create folders or empty files",
    );
    print_cmd_summary(
        "remove",
        "rmv (rm, del)",
        "Safely delete files or directories",
    );
    print_cmd_summary("copy", "cpy (cp)", "Copy files or directories recursively");
    print_cmd_summary("move", "mov (mv)", "Move or rename files and directories");
    print_cmd_summary("read", "red (cat)", "Inspect file contents directly");
    print_cmd_summary("find", "fnd (grep)", "Search pattern or text in files");
    print_cmd_summary("permit", "prm (chmod)", "Change permissions with presets");
    print_cmd_summary(
        "memo",
        "mem (clip)",
        "Quick developer scratchpad & snippet clipboard manager",
    );
    println!();
    println!("{}", electric_blue("SYSTEM & PROCESS:").bold());
    print_cmd_summary(
        "process",
        "prc (ps, top)",
        "Inspect active processes or resource snapshot",
    );
    print_cmd_summary(
        "kill",
        "kil (stop)",
        "Terminate process with search & confirm",
    );
    print_cmd_summary(
        "disk",
        "dsk (df, du)",
        "Inspect disk space or directory usage",
    );
    print_cmd_summary("whoami", "who", "Display user identity and system details");
    print_cmd_summary("time", "tim (date)", "Display current date and time");
    print_cmd_summary("history", "his", "Display recent shell command history");
    print_cmd_summary("which", "whc", "Locate binary executable in PATH");
    print_cmd_summary("env", "env", "Inspect or search environment variables");
    println!();
    println!("{}", electric_blue("NETWORKING & ARCHIVE:").bold());
    print_cmd_summary(
        "port",
        "prt (lsof)",
        "Check active listening ports & sockets",
    );
    print_cmd_summary(
        "fetch",
        "fch (curl, wget)",
        "Fetch HTTP response or download file",
    );
    print_cmd_summary("ping", "png", "Test network host latency");
    print_cmd_summary(
        "pack",
        "pck (tar, zip)",
        "Create compressed archive (.tar.gz, .zip)",
    );
    print_cmd_summary("unpack", "upk (unzip)", "Extract compressed archive");
    print_cmd_summary(
        "speedtest",
        "spd (speed)",
        "Measure internet download, upload and latency",
    );
    println!();
    println!("{}", electric_blue("UTILITY:").bold());
    print_cmd_summary("open", "opn", "Launch macOS applications (open -a)");
    print_cmd_summary("clear", "clr", "Clear terminal screen");
    print_cmd_summary("init", "ini", "Generate shell integration wrapper");
    print_cmd_summary(
        "completion",
        "cmp",
        "Generate native shell completion (zsh, bash, fish)",
    );
    print_cmd_summary(
        "config",
        "cfg",
        "Manage CLI settings, primary color & auto-clear",
    );
    print_cmd_summary("help", "doc (guide)", "Display reference or detailed guide");
    println!();
    println!("{}", electric_blue("CANCELLATION:").bold());
    println!("  - Menus: Select 'Cancel' (last option) to abort.");
    println!("  - Prompts: Press Enter on blank input to abort.");
    println!("  - Deletion: Confirmation defaults to 'N' (Cancel).");
    println!();
    println!(
        "{}",
        electric_blue("Run 'run help <command>' (e.g. 'run help fetch') for detailed guide.")
            .dimmed()
    );
    println!();
}

fn print_cmd_summary(cmd: &str, alias: &str, desc: &str) {
    let name_col = if alias.is_empty() || alias == cmd {
        cmd.to_string()
    } else {
        format!("{cmd}, {alias}")
    };
    println!("  {: <24} {}", name_col.bold(), desc);
}

fn print_command_detail(cmd: &str) {
    let cmd_lower = cmd.to_lowercase();
    println!();

    match cmd_lower.as_str() {
        "make" | "mak" | "mkdir" | "touch" => {
            println!(
                "{} make (alias: mak, mkdir, touch)",
                electric_blue("COMMAND:").bold()
            );
            println!("Unified creation for folders and files directly or interactively.\n");
            println!("{}", electric_blue("USAGE:").bold());
            println!("  run make folder <path>    Create folder and required parents");
            println!("  run make file <path>      Create empty file with parent directories");
            println!("  run mak                   Interactive type selection and path prompt\n");
            println!("{}", electric_blue("EXAMPLES:").bold());
            println!("  run mak folder src/routes");
            println!("  run mak file src/routes/index.ts");
        }
        "remove" | "rmv" | "del" | "dlt" | "rm" | "delete" => {
            println!(
                "{} remove (alias: rmv, del, rm)",
                electric_blue("COMMAND:").bold()
            );
            println!("Safely remove files or directories with confirmation prompt.\n");
            println!("{}", electric_blue("USAGE:").bold());
            println!("  run remove <path>         Prompt confirmation [y/N] then delete");
            println!("  run rmv                   Interactive target prompt & confirmation\n");
            println!("{}", electric_blue("SAFETY:").bold());
            println!("  Defaults to 'No' (false). Pressing Enter cancels deletion immediately.");
        }
        "copy" | "cpy" | "cp" => {
            println!("{} copy (alias: cpy, cp)", electric_blue("COMMAND:").bold());
            println!("Copy single files or recursively copy directory trees.\n");
            println!("{}", electric_blue("USAGE:").bold());
            println!("  run copy <src> <dst>      Copy source to destination");
            println!("  run cpy                   Prompt for source and destination\n");
            println!("{}", electric_blue("EXAMPLES:").bold());
            println!("  run cpy document.pdf backup.pdf");
            println!("  run cpy src/ backup_src/");
        }
        "move" | "mov" | "mv" => {
            println!("{} move (alias: mov, mv)", electric_blue("COMMAND:").bold());
            println!("Move or rename files and directories.\n");
            println!("{}", electric_blue("USAGE:").bold());
            println!("  run move <src> <dst>      Move or rename source to destination");
            println!("  run mov                   Prompt for source and destination\n");
            println!("{}", electric_blue("EXAMPLES:").bold());
            println!("  run mov draft.txt final.txt");
            println!("  run mov assets/ public/assets/");
        }
        "path" | "pth" | "pwd" => {
            println!(
                "{} path (alias: pth, pwd)",
                electric_blue("COMMAND:").bold()
            );
            println!("Print or copy current working directory path.\n");
            println!("{}", electric_blue("USAGE:").bold());
            println!("  run path                  Print current directory");
            println!("  run pth -i                Interactive mode with copy to clipboard");
        }
        "list" | "lst" | "ls" => {
            println!("{} list (alias: lst, ls)", electric_blue("COMMAND:").bold());
            println!("List directory contents with formatted sizes and colored directories.\n");
            println!("{}", electric_blue("USAGE:").bold());
            println!("  run list                  List current directory");
            println!("  run lst -a                Include hidden entries");
            println!("  run lst -l                Long listing format");
        }
        "read" | "red" | "cat" => {
            println!(
                "{} read (alias: red, cat)",
                electric_blue("COMMAND:").bold()
            );
            println!("Display file contents with text decoding.\n");
            println!("{}", electric_blue("USAGE:").bold());
            println!("  run read <file>           Display file contents");
            println!("  run red                   Interactive file picker in current directory");
        }
        "find" | "fnd" | "grep" | "search" => {
            println!(
                "{} find (alias: fnd, grep)",
                electric_blue("COMMAND:").bold()
            );
            println!("Search pattern or text across files recursively.\n");
            println!("{}", electric_blue("USAGE:").bold());
            println!("  run find <pattern> [path] Search pattern in path (default current dir)");
            println!("  run fnd                   Prompt for pattern interactively");
        }
        "process" | "prc" | "ps" | "top" => {
            println!(
                "{} process (alias: prc, ps, top)",
                electric_blue("COMMAND:").bold()
            );
            println!("Inspect active processes or display resource usage snapshot.\n");
            println!("{}", electric_blue("USAGE:").bold());
            println!("  run process               List active processes");
            println!("  run prc <filter>          Filter processes by name or user");
            println!("  run prc -s                Resource usage snapshot (CPU & Memory)");
        }
        "kill" | "kil" | "stop" | "stp" => {
            println!(
                "{} kill (alias: kil, stop)",
                electric_blue("COMMAND:").bold()
            );
            println!("Terminate a running process by PID or name with safe confirmation.\n");
            println!("{}", electric_blue("USAGE:").bold());
            println!("  run kill <pid_or_name>    Prompt confirmation and terminate");
            println!("  run kil                   Interactive process selector & confirm");
        }
        "port" | "prt" | "lsof" => {
            println!(
                "{} port (alias: prt, lsof)",
                electric_blue("COMMAND:").bold()
            );
            println!("Inspect active network ports and listening sockets.\n");
            println!("{}", electric_blue("USAGE:").bold());
            println!("  run port <port>           Check process on port (e.g. 3000)");
            println!("  run prt                   Prompt for port or list all listening TCP");
        }
        "fetch" | "fch" | "get" | "curl" | "wget" => {
            println!(
                "{} fetch (alias: fch, get, curl, wget)",
                electric_blue("COMMAND:").bold()
            );
            println!("Unified network client: fetch HTTP responses or download files locally.\n");
            println!("{}", electric_blue("USAGE:").bold());
            println!("  run fetch <url>           Fetch HTTP response headers and body");
            println!("  run fch <url> -o <file>   Download URL to local file with progress bar");
            println!("  run fch                   Prompt for URL interactively");
        }
        "disk" | "dsk" | "df" | "du" => {
            println!(
                "{} disk (alias: dsk, df, du)",
                electric_blue("COMMAND:").bold()
            );
            println!("Inspect disk free space on mounted volumes or directory usage.\n");
            println!("{}", electric_blue("USAGE:").bold());
            println!("  run disk                  Display mounted volumes free space");
            println!("  run dsk <path>            Calculate directory disk usage");
            println!("  run dsk -u                Interactive folder usage selector");
        }
        "pack" | "pck" | "tar" | "zip" => {
            println!(
                "{} pack (alias: pck, tar, zip)",
                electric_blue("COMMAND:").bold()
            );
            println!("Create compressed archives (.tar.gz or .zip).\n");
            println!("{}", electric_blue("USAGE:").bold());
            println!("  run pack <archive> <target>   Create archive from target");
            println!("  run pck                       Interactive prompt for archive and target");
        }
        "unpack" | "upk" | "unzip" | "untar" => {
            println!(
                "{} unpack (alias: upk, unzip)",
                electric_blue("COMMAND:").bold()
            );
            println!("Extract compressed archives (.zip or .tar.gz).\n");
            println!("{}", electric_blue("USAGE:").bold());
            println!("  run unpack <archive> [dst]    Extract archive to destination");
            println!("  run upk                       Interactive archive picker in current dir");
        }
        "permit" | "prm" | "chmod" => {
            println!(
                "{} permit (alias: prm, chmod)",
                electric_blue("COMMAND:").bold()
            );
            println!("Change file or folder permissions with presets.\n");
            println!("{}", electric_blue("USAGE:").bold());
            println!("  run permit <mode> <path>  Update permissions (e.g. 755, 644, +x)");
            println!("  run prm                   Interactive file & preset selector");
        }
        "ping" | "png" => {
            println!("{} ping (alias: png)", electric_blue("COMMAND:").bold());
            println!("Test network host latency with 4 packets.\n");
            println!("{}", electric_blue("USAGE:").bold());
            println!("  run ping <host>           Ping target host");
            println!("  run png                   Interactive host picker");
        }
        "whoami" | "who" => {
            println!("{} whoami (alias: who)", electric_blue("COMMAND:").bold());
            println!("Print user identity, hostname, UID, GID, and home directory.\n");
            println!("{}", electric_blue("USAGE:").bold());
            println!("  run whoami");
            println!("  run who");
        }
        "time" | "tim" | "date" => {
            println!(
                "{} time (alias: tim, date)",
                electric_blue("COMMAND:").bold()
            );
            println!("Display formatted current date and time.\n");
            println!("{}", electric_blue("USAGE:").bold());
            println!("  run time");
            println!("  run tim");
        }
        "history" | "his" => {
            println!("{} history (alias: his)", electric_blue("COMMAND:").bold());
            println!("Display recent shell command history.\n");
            println!("{}", electric_blue("USAGE:").bold());
            println!("  run history [limit]");
            println!("  run his");
        }
        "which" | "whc" => {
            println!("{} which (alias: whc)", electric_blue("COMMAND:").bold());
            println!("Locate binary executable in system PATH.\n");
            println!("{}", electric_blue("USAGE:").bold());
            println!("  run which <binary>");
            println!("  run whc");
        }
        "env" => {
            println!("{} env", electric_blue("COMMAND:").bold());
            println!("Inspect or search environment variables.\n");
            println!("{}", electric_blue("USAGE:").bold());
            println!("  run env                   List all environment variables");
            println!("  run env <query>           Search variables matching query");
        }
        "clear" | "clr" => {
            println!("{} clear (alias: clr)", electric_blue("COMMAND:").bold());
            println!("Clear terminal screen.\n");
            println!("{}", electric_blue("USAGE:").bold());
            println!("  run clear");
            println!("  run clr");
        }
        "go" | "jmp" | "nav" | "cd" => {
            println!(
                "{} go (alias: jmp, nav, cd)",
                electric_blue("COMMAND:").bold()
            );
            println!("Smart directory navigation replacing manual cd commands.\n");
            println!("{}", electric_blue("USAGE:").bold());
            println!("  run go root               Jump to home directory (~)");
            println!("  run go back               Step back to parent directory (..)");
            println!("  run go <folder>           Jump to subfolder or fuzzy-search hubs");
            println!("  run jmp                   Open interactive folder picker menu\n");
            println!("{}", electric_blue("INTEGRATION:").bold());
            println!("  Add 'eval \"$(run init)\"' to ~/.zshrc for seamless in-place switching.");
        }
        "project" | "prj" => {
            println!("{} project (alias: prj)", electric_blue("COMMAND:").bold());
            println!("Smart project hub scanner and interactive IDE launcher.\n");
            println!("{}", electric_blue("USAGE:").bold());
            println!("  run project               Scan development hubs and pick project");
            println!("  run prj .                 Open current directory");
            println!("  run prj <name>            Search and jump directly to project\n");
            println!("{}", electric_blue("IDE / ACTION MENU:").bold());
            println!("  1. Visual Studio Code (code)");
            println!("  2. Cursor (cursor)");
            println!("  3. Antigravity IDE (antigravity)");
            println!("  4. Xcode (xcode - opens .xcworkspace/.xcodeproj if present)");
            println!("  5. Switch Terminal Directory Only (switches active terminal directory)");
            println!("  6. Cancel");
        }
        "dev" => {
            println!("{} dev", electric_blue("COMMAND:").bold());
            println!("Run development server for current active project.\n");
            println!("{}", electric_blue("DETECTED SIGNATURES:").bold());
            println!("  - Cargo.toml     -> cargo run");
            println!("  - package.json   -> pnpm/yarn/bun/npm run dev");
            println!("  - pubspec.yaml   -> flutter run");
            println!("  - go.mod         -> go run .");
            println!("  - Makefile       -> make dev\n");
            println!("{}", electric_blue("USAGE:").bold());
            println!("  run dev");
        }
        "build" | "bld" => {
            println!("{} build (alias: bld)", electric_blue("COMMAND:").bold());
            println!("Build or compile current active project in release mode.\n");
            println!("{}", electric_blue("DETECTED SIGNATURES:").bold());
            println!("  - Cargo.toml     -> cargo build --release");
            println!("  - package.json   -> pnpm/yarn/bun/npm run build");
            println!("  - pubspec.yaml   -> flutter build");
            println!("  - go.mod         -> go build .");
            println!("  - Makefile       -> make build\n");
            println!("{}", electric_blue("USAGE:").bold());
            println!("  run build");
            println!("  run bld");
        }
        "test" | "tst" => {
            println!("{} test (alias: tst)", electric_blue("COMMAND:").bold());
            println!("Smart polyglot test runner detecting project stack automatically.\n");
            println!("{}", electric_blue("DETECTED RUNNERS:").bold());
            println!("  - Cargo.toml     -> cargo test");
            println!("  - package.json   -> pnpm/yarn/bun/npm test");
            println!("  - pytest/unittest-> pytest / python3 -m unittest");
            println!("  - pubspec.yaml   -> flutter test");
            println!("  - go.mod         -> go test ./...\n");
            println!("{}", electric_blue("USAGE:").bold());
            println!("  run test                  Run project test suite");
            println!("  run tst                   Alias");
        }
        "clean" | "cln" => {
            println!("{} clean (alias: cln)", electric_blue("COMMAND:").bold());
            println!(
                "Clean disposable build artifacts and caches with size calculation and confirmation.\n"
            );
            println!("{}", electric_blue("TARGETED ARTIFACTS:").bold());
            println!(
                "  target/, node_modules/, .next/, dist/, build/, __pycache__/, DerivedData/\n"
            );
            println!("{}", electric_blue("USAGE:").bold());
            println!("  run clean                 Scan and prompt confirmation before cleaning");
            println!("  run cln                   Alias");
        }
        "sync" | "snc" | "git" => {
            println!(
                "{} sync (alias: snc, git)",
                electric_blue("COMMAND:").bold()
            );
            println!("1-step Git sync: pull latest, review status, commit, and push.\n");
            println!("{}", electric_blue("USAGE:").bold());
            println!(
                "  run sync                  Interactive pull, commit message prompt, and push"
            );
            println!("  run sync -m \"commit msg\"  Fast commit and push directly");
            println!("  run sync -b <branch>      Switch or create branch");
        }
        "network" | "net" | "ip" => {
            println!(
                "{} network (alias: net, ip)",
                electric_blue("COMMAND:").bold()
            );
            println!(
                "Inspect local LAN and public IP addresses with interactive copy to clipboard.\n"
            );
            println!("{}", electric_blue("USAGE:").bold());
            println!("  run network               Show LAN & public IP");
            println!("  run net                   Alias");
        }
        "share" | "shr" => {
            println!("{} share (alias: shr)", electric_blue("COMMAND:").bold());
            println!("Instant local LAN HTTP file server for current folder.\n");
            println!("{}", electric_blue("USAGE:").bold());
            println!("  run share                 Start file server on port 8000");
            println!("  run shr -p 8080           Start file server on custom port");
        }
        "bench" | "bnc" => {
            println!("{} bench (alias: bnc)", electric_blue("COMMAND:").bold());
            println!("Benchmark command execution duration with high-precision timing.\n");
            println!("{}", electric_blue("USAGE:").bold());
            println!("  run bench <command...>    Benchmark command execution");
            println!("  run bnc cargo build       Example");
        }
        "speedtest" | "spd" | "speed" => {
            println!(
                "{} speedtest (alias: spd, speed)",
                electric_blue("COMMAND:").bold()
            );
            println!(
                "Measure network download throughput, upload throughput and responsiveness.\n"
            );
            println!("{}", electric_blue("USAGE:").bold());
            println!("  run speedtest             Run full parallel download & upload speed test");
            println!("  run spd                   Alias");
            println!("  run spd -s                Run sequentially instead of parallel");
        }
        "docker" | "dck" => {
            println!("{} docker (alias: dck)", electric_blue("COMMAND:").bold());
            println!("Inspect & manage Docker and OrbStack containers, logs, and processes.\n");
            println!("{}", electric_blue("USAGE:").bold());
            println!("  run docker                Open interactive container manager");
            println!("  run dck                   Alias");
        }
        "secret" | "sec" | "dotenv" => {
            println!(
                "{} secret (alias: sec, dotenv)",
                electric_blue("COMMAND:").bold()
            );
            println!("Audit local .env variables against .env.example and generate templates.\n");
            println!("{}", electric_blue("USAGE:").bold());
            println!(
                "  run secret                Audit missing variables between .env & .env.example"
            );
            println!("  run sec                   Alias");
            println!(
                "  run sec --fix             Generate sanitized .env.example from active .env"
            );
        }
        "memo" | "mem" | "clip" => {
            println!(
                "{} memo (alias: mem, clip)",
                electric_blue("COMMAND:").bold()
            );
            println!("Developer quick scratchpad and snippet clipboard manager.\n");
            println!("{}", electric_blue("USAGE:").bold());
            println!("  run memo                  Interactive snippet browser and picker");
            println!("  run mem add <title> <val> Save a quick snippet or command");
            println!("  run mem copy <title>      Copy snippet to macOS system clipboard");
            println!("  run mem rm <title>        Delete snippet from storage");
            println!("  run mem clear             Clear all snippets");
        }
        "completion" | "cmp" => {
            println!(
                "{} completion (alias: cmp)",
                electric_blue("COMMAND:").bold()
            );
            println!("Generate native shell auto-completion script for zsh, bash, or fish.\n");
            println!("{}", electric_blue("USAGE:").bold());
            println!("  run completion zsh        Print zsh completion script");
            println!("  run completion bash       Print bash completion script");
            println!("  run completion fish       Print fish completion script");
            println!("  run cmp zsh               Alias\n");
            println!("{}", electric_blue("SETUP:").bold());
            println!("  eval \"$(run init)\" automatically loads completion into your shell!");
        }
        "config" | "cfg" => {
            println!("{} config (alias: cfg)", electric_blue("COMMAND:").bold());
            println!(
                "Manage run-cli settings, primary theme colors, default editor & auto-clear.\n"
            );
            println!("{}", electric_blue("USAGE:").bold());
            println!("  run config                Open interactive settings dashboard");
            println!("  run cfg                   Alias");
            println!(
                "  run cfg get <key>         Get config value (primary_color, default_ide, auto_clear)"
            );
            println!(
                "  run cfg set <key> <val>   Set config value directly (e.g. run cfg set primary_color '#ff007f')"
            );
            println!("  run cfg path              Print path to ~/.config/run/config.toml");
            println!("  run cfg edit              Open config file in default editor");
            println!("  run cfg reset             Reset configuration to factory defaults");
        }
        "open" | "opn" => {
            println!("{} open (alias: opn)", electric_blue("COMMAND:").bold());
            println!("Launch macOS desktop applications via native open -a.\n");
            println!("{}", electric_blue("USAGE:").bold());
            println!("  run open <app_name>       Open target application");
            println!("  run opn                   Prompt for application name interactively");
        }
        "init" | "ini" => {
            println!("{} init (alias: ini)", electric_blue("COMMAND:").bold());
            println!("Generate shell integration script for automatic cd in active shell.\n");
            println!("{}", electric_blue("SETUP:").bold());
            println!("  echo 'eval \"$(run init)\"' >> ~/.zshrc");
            println!("  source ~/.zshrc");
        }
        "cancel" | "cancellation" => {
            println!("{}", electric_blue("CANCELLATION STANDARD:").bold());
            println!("All commands support non-destructive cancellation:\n");
            println!("  1. Menus: Select 'Cancel' (last option) to abort immediately.");
            println!("  2. Text prompts: Press Enter on blank input to cancel.");
            println!("  3. Delete prompts: Enter 'n' or press Enter (default No) to cancel.");
        }
        other => {
            println!("No dedicated topic found for '{}'.", other);
            print_main_help();
        }
    }
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_theme() {
        let _theme = custom_theme();
    }

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(levenshtein_distance("pak", "pack"), 1);
        assert_eq!(levenshtein_distance("fethc", "fetch"), 2);
        assert_eq!(levenshtein_distance("prject", "project"), 1);
        assert_eq!(levenshtein_distance("same", "same"), 0);
    }

    #[test]
    fn test_cli_parsing_direct_and_aliases() {
        assert!(matches!(
            Cli::try_parse_from(["run"]),
            Ok(Cli { command: None })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "mak", "folder", "my_folder"]),
            Ok(Cli {
                command: Some(Commands::Make {
                    item: Some(ref item),
                    name: Some(name),
                    ..
                })
            }) if item == "folder" && name == Path::new("my_folder")
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "make", "foo.txt"]),
            Ok(Cli {
                command: Some(Commands::Make {
                    item: Some(ref item),
                    name: None,
                    ..
                })
            }) if item == "foo.txt"
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "make"]),
            Ok(Cli {
                command: Some(Commands::Make {
                    item: None,
                    name: None,
                    ..
                })
            })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "opn", "Finder"]),
            Ok(Cli { command: Some(Commands::Open { target: Some(target) }) }) if target == "Finder"
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "cpy", "a.txt", "b.txt"]),
            Ok(Cli {
                command: Some(Commands::Copy { .. })
            })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "mov", "a.txt", "b.txt"]),
            Ok(Cli {
                command: Some(Commands::Move { .. })
            })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "dlt", "temp.txt"]),
            Ok(Cli {
                command: Some(Commands::Remove { .. })
            })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "clr"]),
            Ok(Cli {
                command: Some(Commands::Clear)
            })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "help"]),
            Ok(Cli {
                command: Some(Commands::Help { command: None })
            })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "guide", "make"]),
            Ok(Cli { command: Some(Commands::Help { command: Some(ref cmd) }) }) if cmd == "make"
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "go", "root"]),
            Ok(Cli { command: Some(Commands::Go { target: Some(ref t) }) }) if t == "root"
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "jmp", "back"]),
            Ok(Cli { command: Some(Commands::Go { target: Some(ref t) }) }) if t == "back"
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "g", "root"]),
            Ok(Cli { command: Some(Commands::Go { target: Some(ref t) }) }) if t == "root"
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "nav"]),
            Ok(Cli {
                command: Some(Commands::Go { target: None })
            })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "ini"]),
            Ok(Cli {
                command: Some(Commands::Init)
            })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "project"]),
            Ok(Cli {
                command: Some(Commands::Project { target: None })
            })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "pro"]),
            Ok(Cli {
                command: Some(Commands::Project { target: None })
            })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "prj", "."]),
            Ok(Cli { command: Some(Commands::Project { target: Some(ref t) }) }) if t == "."
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "dev"]),
            Ok(Cli {
                command: Some(Commands::Dev)
            })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "develop"]),
            Ok(Cli {
                command: Some(Commands::Dev)
            })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "build"]),
            Ok(Cli {
                command: Some(Commands::Build)
            })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "bld"]),
            Ok(Cli {
                command: Some(Commands::Build)
            })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "path"]),
            Ok(Cli {
                command: Some(Commands::Path { interactive: false })
            })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "pth"]),
            Ok(Cli {
                command: Some(Commands::Path { interactive: false })
            })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "pwd"]),
            Ok(Cli {
                command: Some(Commands::Path { interactive: false })
            })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "list", "-a", "-l"]),
            Ok(Cli {
                command: Some(Commands::List {
                    all: true,
                    long: true,
                    ..
                })
            })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "lst"]),
            Ok(Cli {
                command: Some(Commands::List { .. })
            })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "ls"]),
            Ok(Cli {
                command: Some(Commands::List { .. })
            })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "cd", "root"]),
            Ok(Cli { command: Some(Commands::Go { target: Some(ref t) }) }) if t == "root"
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "mkdir", "my_dir"]),
            Ok(Cli {
                command: Some(Commands::Make { .. })
            })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "touch", "my_file"]),
            Ok(Cli {
                command: Some(Commands::Make { .. })
            })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "cp", "a", "b"]),
            Ok(Cli {
                command: Some(Commands::Copy { .. })
            })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "mv", "a", "b"]),
            Ok(Cli {
                command: Some(Commands::Move { .. })
            })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "remove", "dir"]),
            Ok(Cli {
                command: Some(Commands::Remove { .. })
            })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "rmv", "dir"]),
            Ok(Cli {
                command: Some(Commands::Remove { .. })
            })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "rm", "dir"]),
            Ok(Cli {
                command: Some(Commands::Remove { .. })
            })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "read", "file.txt"]),
            Ok(Cli {
                command: Some(Commands::Read { path: Some(_) })
            })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "red", "file.txt"]),
            Ok(Cli {
                command: Some(Commands::Read { path: Some(_) })
            })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "cat", "file.txt"]),
            Ok(Cli {
                command: Some(Commands::Read { path: Some(_) })
            })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "find", "hello"]),
            Ok(Cli { command: Some(Commands::Find { pattern: Some(ref p), .. }) }) if p == "hello"
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "fnd", "hello"]),
            Ok(Cli { command: Some(Commands::Find { pattern: Some(ref p), .. }) }) if p == "hello"
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "grep", "hello"]),
            Ok(Cli { command: Some(Commands::Find { pattern: Some(ref p), .. }) }) if p == "hello"
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "process"]),
            Ok(Cli {
                command: Some(Commands::Process { .. })
            })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "prc"]),
            Ok(Cli {
                command: Some(Commands::Process { .. })
            })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "proc"]),
            Ok(Cli {
                command: Some(Commands::Process { .. })
            })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "kill", "1234"]),
            Ok(Cli {
                command: Some(Commands::Kill { .. })
            })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "kil", "1234"]),
            Ok(Cli {
                command: Some(Commands::Kill { .. })
            })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "port", "3000"]),
            Ok(Cli {
                command: Some(Commands::Port { .. })
            })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "prt", "3000"]),
            Ok(Cli {
                command: Some(Commands::Port { .. })
            })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "fetch", "https://example.com"]),
            Ok(Cli {
                command: Some(Commands::Fetch { .. })
            })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "fch", "https://example.com"]),
            Ok(Cli {
                command: Some(Commands::Fetch { .. })
            })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "disk"]),
            Ok(Cli {
                command: Some(Commands::Disk { .. })
            })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "dsk"]),
            Ok(Cli {
                command: Some(Commands::Disk { .. })
            })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "whoami"]),
            Ok(Cli {
                command: Some(Commands::Whoami)
            })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "who"]),
            Ok(Cli {
                command: Some(Commands::Whoami)
            })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "which", "node"]),
            Ok(Cli { command: Some(Commands::Which { binary: Some(ref b) }) }) if b == "node"
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "whc", "node"]),
            Ok(Cli { command: Some(Commands::Which { binary: Some(ref b) }) }) if b == "node"
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "test"]),
            Ok(Cli {
                command: Some(Commands::Test)
            })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "tst"]),
            Ok(Cli {
                command: Some(Commands::Test)
            })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "clean"]),
            Ok(Cli {
                command: Some(Commands::Clean { .. })
            })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "cln"]),
            Ok(Cli {
                command: Some(Commands::Clean { .. })
            })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "sync"]),
            Ok(Cli {
                command: Some(Commands::Sync { .. })
            })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "snc"]),
            Ok(Cli {
                command: Some(Commands::Sync { .. })
            })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "git"]),
            Ok(Cli {
                command: Some(Commands::Sync { .. })
            })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "network"]),
            Ok(Cli {
                command: Some(Commands::Network)
            })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "net"]),
            Ok(Cli {
                command: Some(Commands::Network)
            })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "ip"]),
            Ok(Cli {
                command: Some(Commands::Network)
            })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "share"]),
            Ok(Cli {
                command: Some(Commands::Share { port: None, .. })
            })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "shr", "-p", "8080"]),
            Ok(Cli {
                command: Some(Commands::Share {
                    port: Some(8080),
                    ..
                })
            })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "bench", "echo", "hello"]),
            Ok(Cli { command: Some(Commands::Bench { ref command }) }) if command == &["echo", "hello"]
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "bnc", "ls"]),
            Ok(Cli { command: Some(Commands::Bench { ref command }) }) if command == &["ls"]
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "speedtest"]),
            Ok(Cli {
                command: Some(Commands::Speedtest { sequential: false })
            })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "spd", "-s"]),
            Ok(Cli {
                command: Some(Commands::Speedtest { sequential: true })
            })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "speed"]),
            Ok(Cli {
                command: Some(Commands::Speedtest { sequential: false })
            })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "docker"]),
            Ok(Cli {
                command: Some(Commands::Docker)
            })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "dck"]),
            Ok(Cli {
                command: Some(Commands::Docker)
            })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "secret"]),
            Ok(Cli {
                command: Some(Commands::Secret { fix: false })
            })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "sec", "--fix"]),
            Ok(Cli {
                command: Some(Commands::Secret { fix: true })
            })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "dotenv"]),
            Ok(Cli {
                command: Some(Commands::Secret { fix: false })
            })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "memo"]),
            Ok(Cli {
                command: Some(Commands::Memo {
                    action: None,
                    arg1: None,
                    arg2: None
                })
            })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "mem", "add", "mykey", "val"]),
            Ok(Cli {
                command: Some(Commands::Memo { ref action, ref arg1, ref arg2 })
            }) if action.as_deref() == Some("add") && arg1.as_deref() == Some("mykey") && arg2.as_deref() == Some("val")
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "clip"]),
            Ok(Cli {
                command: Some(Commands::Memo {
                    action: None,
                    arg1: None,
                    arg2: None
                })
            })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "completion", "zsh"]),
            Ok(Cli {
                command: Some(Commands::Completion { ref shell })
            }) if shell == "zsh"
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "cmp", "fish"]),
            Ok(Cli {
                command: Some(Commands::Completion { ref shell })
            }) if shell == "fish"
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "config"]),
            Ok(Cli {
                command: Some(Commands::Config {
                    action: None,
                    key: None,
                    val: None
                })
            })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "cfg", "set", "primary_color", "#ff007f"]),
            Ok(Cli {
                command: Some(Commands::Config { ref action, ref key, ref val })
            }) if action.as_deref() == Some("set") && key.as_deref() == Some("primary_color") && val.as_deref() == Some("#ff007f")
        ));
    }

    #[test]
    fn test_color_parsing() {
        assert_eq!(config::parse_color("#ff007f"), (255, 0, 127));
        assert_eq!(config::parse_color("violet"), (139, 92, 246));
        assert_eq!(config::parse_color("emerald"), (16, 185, 129));
        assert_eq!(config::parse_color("electric-blue"), (0, 162, 255));
    }

    #[test]
    fn test_resolve_command_args_smart_prefix_and_typo() {
        let theme = custom_theme();

        // Exact alias: pro -> project
        let res =
            resolve_command_args_internal(&theme, &["run".into(), "pro".into()], false).unwrap();
        assert_eq!(res, Some(vec!["run".to_string(), "project".to_string()]));

        // Prefix match: proj -> project
        let res =
            resolve_command_args_internal(&theme, &["run".into(), "proj".into()], false).unwrap();
        assert_eq!(res, Some(vec!["run".to_string(), "project".to_string()]));

        // Prefix match: proc -> process
        let res =
            resolve_command_args_internal(&theme, &["run".into(), "proc".into()], false).unwrap();
        assert_eq!(res, Some(vec!["run".to_string(), "process".to_string()]));

        // Single letter prefix: g -> go
        let res =
            resolve_command_args_internal(&theme, &["run".into(), "g".into()], false).unwrap();
        assert_eq!(res, Some(vec!["run".to_string(), "go".to_string()]));

        // Fuzzy / typo in non-interactive environment: pak -> pack
        let res =
            resolve_command_args_internal(&theme, &["run".into(), "pak".into()], false).unwrap();
        assert_eq!(res, Some(vec!["run".to_string(), "pack".to_string()]));

        // Fuzzy / typo in non-interactive environment: fethc -> fetch
        let res =
            resolve_command_args_internal(&theme, &["run".into(), "fethc".into()], false).unwrap();
        assert_eq!(res, Some(vec!["run".to_string(), "fetch".to_string()]));
    }

    #[test]
    fn test_find_matching_apps() {
        let installed = vec![
            "Safari".to_string(),
            "WhatsApp".to_string(),
            "Visual Studio Code".to_string(),
            "Google Chrome".to_string(),
        ];

        // Prefix match: What -> WhatsApp
        assert_eq!(find_matching_apps("What", &installed), vec!["WhatsApp"]);

        // Case-insensitive exact match
        assert_eq!(find_matching_apps("safari", &installed), vec!["Safari"]);

        // Substring match: Code -> Visual Studio Code
        assert_eq!(
            find_matching_apps("Code", &installed),
            vec!["Visual Studio Code"]
        );

        // Typo match (Levenshtein distance <= 3): whasap -> WhatsApp
        assert_eq!(find_matching_apps("whasap", &installed), vec!["WhatsApp"]);
    }
}
