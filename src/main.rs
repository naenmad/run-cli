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
    }
}
