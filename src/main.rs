use std::collections::HashSet;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use colored::Colorize;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Confirm, Input, Select};

#[derive(Parser)]
#[command(name = "run")]
#[command(about = "Productivity CLI utility for macOS with dual-mode interaction", version)]
#[command(disable_help_subcommand = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new folder or file directly or through an interactive menu
    #[command(alias = "mak")]
    Make {
        /// Target type to create
        #[arg(value_enum)]
        target_type: Option<MakeTargetType>,
        /// Name or path of the target item
        name: Option<PathBuf>,
    },

    /// Open a macOS application using open -a
    #[command(alias = "opn")]
    Open {
        /// Target application name
        target: Option<String>,
    },

    /// Copy a file or folder from source to destination
    #[command(name = "copy", alias = "cpy")]
    Copy {
        /// Source file or folder
        source: Option<PathBuf>,
        /// Destination path
        destination: Option<PathBuf>,
    },

    /// Move or rename a file or folder
    #[command(name = "move", alias = "mov")]
    Move {
        /// Source file or folder
        source: Option<PathBuf>,
        /// Destination path
        destination: Option<PathBuf>,
    },

    /// Delete a file or folder with safe confirmation
    #[command(name = "del", alias = "dlt", alias = "delete")]
    Del {
        /// Path of the file or folder to delete
        target: Option<PathBuf>,
    },

    /// Clear the terminal screen
    #[command(name = "clear", alias = "clr")]
    Clear,

    /// Smart directory navigation (root, back, subfolder, or interactive menu)
    #[command(name = "go", alias = "jmp", alias = "nav")]
    Go {
        /// Target folder name, 'root', or 'back'
        target: Option<String>,
    },

    /// Smart project management, workspace scanner, and IDE selector
    #[command(name = "project", alias = "prj")]
    Project {
        /// Target project path or keyword ('.' for current directory)
        target: Option<String>,
    },

    /// Run development server for the current active project
    #[command(name = "dev")]
    Dev,

    /// Build or compile the current active project
    #[command(name = "build", alias = "bld")]
    Build,

    /// Generate shell integration script for automatic directory switching
    #[command(name = "init", alias = "ini")]
    Init,

    /// Display complete command reference and usage tutorial
    #[command(name = "help", alias = "guide", alias = "doc")]
    Help {
        /// Optional command name or alias to inspect
        command: Option<String>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
enum MakeTargetType {
    Folder,
    File,
}

fn main() {
    if let Err(err) = run_app() {
        eprintln!("{} {err:#}", "error:".bold().red());
        std::process::exit(1);
    }
}

fn run_app() -> Result<()> {
    let cli = Cli::parse();
    let theme = ColorfulTheme::default();

    match cli.command {
        Commands::Make { target_type, name } => handle_make(&theme, target_type, name),
        Commands::Open { target } => handle_open(&theme, target),
        Commands::Copy {
            source,
            destination,
        } => handle_copy(&theme, source, destination),
        Commands::Move {
            source,
            destination,
        } => handle_move(&theme, source, destination),
        Commands::Del { target } => handle_del(&theme, target),
        Commands::Clear => clear_terminal(),
        Commands::Go { target } => handle_go(&theme, target.as_deref()),
        Commands::Project { target } => handle_project(&theme, target.as_deref()),
        Commands::Dev => handle_dev(),
        Commands::Build => handle_build(),
        Commands::Init => handle_init(),
        Commands::Help { command } => handle_help(command.as_deref()),
    }
}

/// Handles folder or file creation in both direct and interactive modes.
fn handle_make(
    theme: &ColorfulTheme,
    target_type: Option<MakeTargetType>,
    name: Option<PathBuf>,
) -> Result<()> {
    let resolved_type = match target_type {
        Some(t) => t,
        None => {
            let options = ["Folder", "File", "Cancel"];
            let selection = Select::with_theme(theme)
                .with_prompt("Select item type to create")
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

    let resolved_name = match name {
        Some(path) => path,
        None => {
            let prompt_text = match resolved_type {
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
            PathBuf::from(trimmed)
        }
    };

    match resolved_type {
        MakeTargetType::Folder => make_directory(&resolved_name),
        MakeTargetType::File => create_empty_file(&resolved_name),
    }
}

/// Handles opening macOS applications with a fallback prompt when target is omitted.
fn handle_open(theme: &ColorfulTheme, target: Option<String>) -> Result<()> {
    let app_name = match target {
        Some(name) => name,
        None => {
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
        }
    };

    let status = Command::new("open")
        .arg("-a")
        .arg(&app_name)
        .status()
        .with_context(|| format!("failed to execute 'open -a {app_name}'"))?;

    if !status.success() {
        bail!("application '{app_name}' was not found or failed to launch");
    }

    println!("Opened application: {app_name}");
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

    fs::File::create_new(path)
        .with_context(|| format!("failed to create file '{}' (file may already exist)", path.display()))?;

    println!("Created file: {}", path.display());
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

    println!("Copied '{}' to '{}'", source.display(), destination.display());
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

    println!("Moved '{}' to '{}'", source.display(), destination.display());
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
        fs::create_dir_all(parent).with_context(|| {
            format!("failed to create parent directory '{}'", parent.display())
        })?;
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
    let current_dir = std::env::current_dir().context("failed to read current working directory")?;
    let home_dir = std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/"));

    let target_path = match target {
        Some("root") | Some("~") => home_dir,
        Some("back") | Some("..") => {
            current_dir.parent().unwrap_or(&current_dir).to_path_buf()
        }
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
                    display_items.push("Cancel".to_string());

                    let prompt = format!("Multiple folders match '{query}'. Select target:");
                    let selection = Select::with_theme(theme)
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
            options.push("Cancel".to_string());

            let selection = Select::with_theme(theme)
                .with_prompt("Select target folder")
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
        "Library", ".Trash", ".git", "node_modules", "target", ".cargo", ".rustup",
        ".gemini", ".vscode", ".npm", ".cache", ".local", "venv", ".venv",
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

fn scan_for_query(
    dir: &Path,
    depth: usize,
    depth_from_root: usize,
    ctx: &mut SearchContext,
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
    if [ "$1" = "go" ] || [ "$1" = "jmp" ] || [ "$1" = "nav" ] || [ "$1" = "project" ] || [ "$1" = "prj" ]; then
        local target
        target="$(RUN_SHELL_RESOLVE=1 command run "$@")" || return $?
        if [ -n "$target" ] && [ -d "$target" ]; then
            cd "$target"
        fi
    else
        command run "$@"
    fi
}}"#
    );
    Ok(())
}

/// Handles smart project scanning, contextual project selection, and interactive IDE launching.
fn handle_project(theme: &ColorfulTheme, target: Option<&str>) -> Result<()> {
    let current_dir = std::env::current_dir().context("failed to read current working directory")?;
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
                    display_items.push("Cancel".to_string());

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
            let scanned = scan_projects(&home_dir, &current_dir);
            if scanned.is_empty() {
                bail!("no projects found in common development hubs");
            }

            let home_str = home_dir.to_string_lossy();
            let mut display_items: Vec<String> = scanned
                .iter()
                .take(30)
                .map(|p| {
                    let p_str = p.to_string_lossy();
                    if p_str.starts_with(home_str.as_ref()) {
                        format!("~{}", &p_str[home_str.len()..])
                    } else {
                        p_str.to_string()
                    }
                })
                .collect();
            display_items.push("Cancel".to_string());

            let selection = Select::with_theme(theme)
                .with_prompt("Select project")
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

    let ide_options = [
        "Visual Studio Code (code)",
        "Cursor (cursor)",
        "Xcode (xcode)",
        "Hanya Pindah Terminal / Saja",
        "Cancel",
    ];

    let term = dialoguer::console::Term::stderr();
    let project_name = canonical
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project");
    let prompt = format!("Open '{project_name}' with:");

    let ide_selection = Select::with_theme(theme)
        .with_prompt(prompt)
        .items(&ide_options)
        .default(0)
        .interact_on(&term)?;

    let is_shell_resolve = std::env::var("RUN_SHELL_RESOLVE")
        .map(|v| v == "1")
        .unwrap_or(false);

    match ide_selection {
        0 => open_in_vscode(&canonical, is_shell_resolve)?,
        1 => open_in_cursor(&canonical, is_shell_resolve)?,
        2 => open_in_xcode(&canonical, is_shell_resolve)?,
        3 => {
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
        Command::new("open").arg("-a").arg("Xcode").arg(path).status()
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

    let candidate_hubs = [
        home_dir.join("Developer"),
        home_dir.join("Projects"),
        home_dir.join("Code"),
        home_dir.join("Documents"),
        home_dir.join("Desktop"),
    ];

    let ignored_names = [
        "Library", ".Trash", ".git", "node_modules", "target", ".cargo", ".rustup",
        ".gemini", ".vscode", ".npm", ".cache", ".local", "venv", ".venv", "Pods",
        "DerivedData", ".build", ".next", "dist", "build",
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
            .cyan()
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
        format!("Building project ({label})...").cyan().bold()
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

const COMMANDS_DOC: &str = include_str!("../COMMANDS.md");

/// Displays the complete command reference or details for a requested command.
fn handle_help(target_command: Option<&str>) -> Result<()> {
    match target_command {
        None => {
            println!("{COMMANDS_DOC}");
        }
        Some(cmd) => {
            let cmd_lower = cmd.to_lowercase();
            let mut matched_section = None;

            for section in COMMANDS_DOC.split("\n---") {
                let trimmed = section.trim();
                for line in trimmed.lines() {
                    let line_lower = line.to_lowercase();
                    if line.starts_with('#') && line_lower.contains(&cmd_lower) {
                        matched_section = Some(trimmed);
                        break;
                    }
                }
                if matched_section.is_some() {
                    break;
                }
            }

            match matched_section {
                Some(section) => println!("\n{section}\n"),
                None => {
                    println!("No specific reference found for '{cmd}'. Displaying full guide:\n");
                    println!("{COMMANDS_DOC}");
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parsing_direct_and_aliases() {
        assert!(matches!(
            Cli::try_parse_from(["run", "mak", "folder", "my_folder"]),
            Ok(Cli {
                command: Commands::Make {
                    target_type: Some(MakeTargetType::Folder),
                    name: Some(name),
                }
            }) if name == Path::new("my_folder")
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "make"]),
            Ok(Cli {
                command: Commands::Make {
                    target_type: None,
                    name: None,
                }
            })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "opn", "Finder"]),
            Ok(Cli { command: Commands::Open { target: Some(target) } }) if target == "Finder"
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "cpy", "a.txt", "b.txt"]),
            Ok(Cli { command: Commands::Copy { .. } })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "mov", "a.txt", "b.txt"]),
            Ok(Cli { command: Commands::Move { .. } })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "dlt", "temp.txt"]),
            Ok(Cli { command: Commands::Del { .. } })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "clr"]),
            Ok(Cli { command: Commands::Clear })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "help"]),
            Ok(Cli { command: Commands::Help { command: None } })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "guide", "make"]),
            Ok(Cli { command: Commands::Help { command: Some(ref cmd) } }) if cmd == "make"
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "go", "root"]),
            Ok(Cli { command: Commands::Go { target: Some(ref t) } }) if t == "root"
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "jmp", "back"]),
            Ok(Cli { command: Commands::Go { target: Some(ref t) } }) if t == "back"
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "nav"]),
            Ok(Cli { command: Commands::Go { target: None } })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "ini"]),
            Ok(Cli { command: Commands::Init })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "project"]),
            Ok(Cli { command: Commands::Project { target: None } })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "prj", "."]),
            Ok(Cli { command: Commands::Project { target: Some(ref t) } }) if t == "."
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "dev"]),
            Ok(Cli { command: Commands::Dev })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "build"]),
            Ok(Cli { command: Commands::Build })
        ));

        assert!(matches!(
            Cli::try_parse_from(["run", "bld"]),
            Ok(Cli { command: Commands::Build })
        ));
    }
}
