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
            let options = ["Folder", "File"];
            let selection = Select::with_theme(theme)
                .with_prompt("Select item type to create")
                .items(&options)
                .default(0)
                .interact()?;

            match selection {
                0 => MakeTargetType::Folder,
                _ => MakeTargetType::File,
            }
        }
    };

    let resolved_name = match name {
        Some(path) => path,
        None => {
            let prompt_text = match resolved_type {
                MakeTargetType::Folder => "Folder path",
                MakeTargetType::File => "File path",
            };
            let input: String = Input::with_theme(theme)
                .with_prompt(prompt_text)
                .interact_text()?;
            PathBuf::from(input.trim())
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
                .with_prompt("Application name")
                .interact_text()?;
            input.trim().to_string()
        }
    };

    if app_name.is_empty() {
        bail!("Application name cannot be empty");
    }

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
                .with_prompt("Source path")
                .interact_text()?;
            PathBuf::from(input.trim())
        }
    };

    let dst = match destination {
        Some(path) => path,
        None => {
            let input: String = Input::with_theme(theme)
                .with_prompt("Destination path")
                .interact_text()?;
            PathBuf::from(input.trim())
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
                .with_prompt("Source path")
                .interact_text()?;
            PathBuf::from(input.trim())
        }
    };

    let dst = match destination {
        Some(path) => path,
        None => {
            let input: String = Input::with_theme(theme)
                .with_prompt("Destination path")
                .interact_text()?;
            PathBuf::from(input.trim())
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
                .with_prompt("Path to delete")
                .interact_text()?;
            PathBuf::from(input.trim())
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

                    let display_items: Vec<String> = candidates
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

                    let prompt = format!("Multiple folders match '{query}'. Select target:");
                    let selection = Select::with_theme(theme)
                        .with_prompt(prompt)
                        .items(&display_items)
                        .default(0)
                        .interact_on(&term)?;

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

            let selection = Select::with_theme(theme)
                .with_prompt("Select target folder")
                .items(&options)
                .default(0)
                .interact_on(&term)?;

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
            search_roots.push((hub_path, 2));
        }
    }

    let ignored_names = [
        "Library", ".Trash", ".git", "node_modules", "target", ".cargo", ".rustup",
        ".gemini", ".vscode", ".npm", ".cache", ".local", "venv", ".venv",
    ];

    for (root, max_depth) in search_roots {
        scan_for_query(
            &root,
            &q_lower,
            current_dir,
            max_depth,
            &ignored_names,
            &mut visited,
            &mut scored,
        );
    }

    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    scored.into_iter().map(|(_, path)| path).collect()
}

fn scan_for_query(
    dir: &Path,
    q_lower: &str,
    current_dir: &Path,
    depth: usize,
    ignored_names: &[&str],
    visited: &mut HashSet<PathBuf>,
    scored: &mut Vec<(f64, PathBuf)>,
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
                if path.starts_with(current_dir) {
                    score += 0.05;
                }
                scored.push((score, path.clone()));
            }

            scan_for_query(
                &path,
                q_lower,
                current_dir,
                depth - 1,
                ignored_names,
                visited,
                scored,
            );
        }
    }
}

/// Generates the shell integration wrapper function for zsh and bash.
fn handle_init() -> Result<()> {
    println!(
        r#"run() {{
    if [ "$1" = "go" ] || [ "$1" = "jmp" ] || [ "$1" = "nav" ]; then
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
            }) if name == PathBuf::from("my_folder")
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
    }
}
