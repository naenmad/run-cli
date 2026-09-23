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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
enum MakeTargetType {
    Folder,
    File,
}

fn main() {
    if let Err(err) = run_app() {
        eprintln!(
            "\n{} {}\n",
            "❌ Error:".bold().red(),
            format!("{err:#}").red()
        );
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
            println!("\n{} {}", "🚀".cyan(), "Interactive Item Creation".bold().cyan());
            let options = ["Folder", "File"];
            let selection = Select::with_theme(theme)
                .with_prompt("Choose item type to create")
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
                MakeTargetType::Folder => "Enter new folder path",
                MakeTargetType::File => "Enter new file path",
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
            println!("\n{} {}", "ℹ️".cyan(), "Open macOS Application".bold().cyan());
            let input: String = Input::with_theme(theme)
                .with_prompt("Enter application name")
                .interact_text()?;
            input.trim().to_string()
        }
    };

    if app_name.is_empty() {
        bail!("Application name cannot be empty");
    }

    println!(
        "\n{} {}",
        "🚀".cyan(),
        format!("Launching application '{app_name}'...").cyan()
    );

    let status = Command::new("open")
        .arg("-a")
        .arg(&app_name)
        .status()
        .with_context(|| format!("Failed to execute 'open -a {app_name}'"))?;

    if !status.success() {
        bail!("Application '{app_name}' was not found or failed to launch");
    }

    println!(
        "{} {}\n",
        "✅".green(),
        format!("Success: '{app_name}' is now open.").bold().green()
    );
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
            println!("\n{} {}", "ℹ️".cyan(), "Copy Item".bold().cyan());
            let input: String = Input::with_theme(theme)
                .with_prompt("Enter source path (file or folder)")
                .interact_text()?;
            PathBuf::from(input.trim())
        }
    };

    let dst = match destination {
        Some(path) => path,
        None => {
            let input: String = Input::with_theme(theme)
                .with_prompt("Enter destination path")
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
            println!("\n{} {}", "ℹ️".cyan(), "Move or Rename Item".bold().cyan());
            let input: String = Input::with_theme(theme)
                .with_prompt("Enter source path (file or folder)")
                .interact_text()?;
            PathBuf::from(input.trim())
        }
    };

    let dst = match destination {
        Some(path) => path,
        None => {
            let input: String = Input::with_theme(theme)
                .with_prompt("Enter destination path")
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
            println!("\n{} {}", "⚠️".yellow(), "Delete Item".bold().yellow());
            let input: String = Input::with_theme(theme)
                .with_prompt("Enter path to delete")
                .interact_text()?;
            PathBuf::from(input.trim())
        }
    };

    if !path.exists() {
        bail!("Target path does not exist: {}", path.display());
    }

    let item_kind = if path.is_dir() { "directory" } else { "file" };

    let prompt = format!(
        "⚠️  Are you sure you want to permanently delete {item_kind} '{}'?",
        path.display()
    );

    let confirmation = Confirm::with_theme(theme)
        .with_prompt(prompt.yellow().to_string())
        .default(false)
        .interact()?;

    if !confirmation {
        println!(
            "\n{} {}\n",
            "⚠️".yellow(),
            "Warning: Deletion cancelled by user.".yellow()
        );
        return Ok(());
    }

    delete_item(&path)
}

/// Creates a nested directory tree using fs::create_dir_all.
fn make_directory(path: &Path) -> Result<()> {
    fs::create_dir_all(path)
        .with_context(|| format!("Failed to create directory '{}'", path.display()))?;

    println!(
        "\n{} {}\n",
        "✅".green(),
        format!("Success: Directory '{}' created.", path.display())
            .bold()
            .green()
    );
    Ok(())
}

/// Creates a new empty file and ensures the parent directory exists.
fn create_empty_file(path: &Path) -> Result<()> {
    ensure_parent_exists(path)?;

    fs::File::create_new(path)
        .with_context(|| format!("Failed to create file '{}' (file may already exist)", path.display()))?;

    println!(
        "\n{} {}\n",
        "✅".green(),
        format!("Success: File '{}' created.", path.display())
            .bold()
            .green()
    );
    Ok(())
}

/// Copies a single file or recursively copies an entire directory tree.
fn copy_item(source: &Path, destination: &Path) -> Result<()> {
    if !source.exists() {
        bail!("Source does not exist: {}", source.display());
    }

    if source.is_dir() {
        copy_directory_recursive(source, destination)?;
    } else {
        ensure_parent_exists(destination)?;
        fs::copy(source, destination).with_context(|| {
            format!(
                "Failed to copy file from '{}' to '{}'",
                source.display(),
                destination.display()
            )
        })?;
    }

    println!(
        "\n{} {}\n",
        "✅".green(),
        format!(
            "Success: Copied '{}' to '{}'.",
            source.display(),
            destination.display()
        )
        .bold()
        .green()
    );
    Ok(())
}

/// Recursively copies a directory tree to a target destination.
fn copy_directory_recursive(source: &Path, destination: &Path) -> Result<()> {
    fs::create_dir_all(destination).with_context(|| {
        format!(
            "Failed to create destination directory '{}'",
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
        bail!("Source does not exist: {}", source.display());
    }

    ensure_parent_exists(destination)?;

    fs::rename(source, destination).with_context(|| {
        format!(
            "Failed to move '{}' to '{}'",
            source.display(),
            destination.display()
        )
    })?;

    println!(
        "\n{} {}\n",
        "✅".green(),
        format!(
            "Success: Moved '{}' to '{}'.",
            source.display(),
            destination.display()
        )
        .bold()
        .green()
    );
    Ok(())
}

/// Deletes a single file or an entire directory tree.
fn delete_item(path: &Path) -> Result<()> {
    let item_kind = if path.is_dir() {
        fs::remove_dir_all(path)
            .with_context(|| format!("Failed to delete directory '{}'", path.display()))?;
        "Directory"
    } else {
        fs::remove_file(path)
            .with_context(|| format!("Failed to delete file '{}'", path.display()))?;
        "File"
    };

    println!(
        "\n{} {}\n",
        "✅".green(),
        format!("Success: {item_kind} '{}' deleted.", path.display())
            .bold()
            .green()
    );
    Ok(())
}

/// Ensures the parent directory of a path exists before file operations.
fn ensure_parent_exists(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
        && !parent.exists()
    {
        fs::create_dir_all(parent).with_context(|| {
            format!("Failed to create parent directory '{}'", parent.display())
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
        .context("Failed to flush stdout")?;

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
    }
}
