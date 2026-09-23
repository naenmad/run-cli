use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use dialoguer::{Confirm, Input, Select};

#[derive(Parser)]
#[command(name = "run")]
#[command(about = "CLI utilitas produktivitas untuk macOS dengan mode ganda", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Membuat folder atau file baru secara langsung atau melalui menu interaktif
    #[command(alias = "mak")]
    Make {
        /// Jenis target yang akan dibuat
        #[arg(value_enum)]
        target_type: Option<MakeTargetType>,
        /// Nama atau jalur berkas/folder
        name: Option<PathBuf>,
    },

    /// Membuka aplikasi macOS menggunakan perintah open -a
    #[command(alias = "opn")]
    Open {
        /// Nama aplikasi target
        target: Option<String>,
    },

    /// Menyalin file atau folder dari sumber ke tujuan
    #[command(name = "copy", alias = "cpy")]
    Copy {
        /// File atau folder sumber
        source: Option<PathBuf>,
        /// Lokasi tujuan
        destination: Option<PathBuf>,
    },

    /// Memindahkan atau mengganti nama file atau folder
    #[command(name = "move", alias = "mov")]
    Move {
        /// File atau folder sumber
        source: Option<PathBuf>,
        /// Lokasi tujuan
        destination: Option<PathBuf>,
    },

    /// Menghapus file atau folder dengan konfirmasi aman
    #[command(name = "del", alias = "dlt", alias = "delete")]
    Del {
        /// Jalur file atau folder yang akan dihapus
        target: Option<PathBuf>,
    },

    /// Membersihkan layar terminal
    #[command(name = "clear", alias = "clr")]
    Clear,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
enum MakeTargetType {
    Folder,
    File,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Make { target_type, name } => handle_make(target_type, name),
        Commands::Open { target } => handle_open(target),
        Commands::Copy {
            source,
            destination,
        } => handle_copy(source, destination),
        Commands::Move {
            source,
            destination,
        } => handle_move(source, destination),
        Commands::Del { target } => handle_del(target),
        Commands::Clear => clear_terminal(),
    }
}

/// Menangani pembuatan folder atau file dalam mode langsung maupun interaktif.
fn handle_make(target_type: Option<MakeTargetType>, name: Option<PathBuf>) -> Result<()> {
    let resolved_type = match target_type {
        Some(t) => t,
        None => {
            let options = ["Folder", "File"];
            let selection = Select::new()
                .with_prompt("Pilih jenis item yang ingin dibuat")
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
                MakeTargetType::Folder => "Nama atau jalur folder",
                MakeTargetType::File => "Nama atau jalur file",
            };
            let input: String = Input::new()
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

/// Menangani pembukaan aplikasi macOS dengan fallback prompt jika target belum diberikan.
fn handle_open(target: Option<String>) -> Result<()> {
    let app_name = match target {
        Some(name) => name,
        None => {
            let input: String = Input::new()
                .with_prompt("Nama aplikasi macOS yang ingin dibuka")
                .interact_text()?;
            input.trim().to_string()
        }
    };

    if app_name.is_empty() {
        bail!("Nama aplikasi tidak boleh kosong");
    }

    let status = Command::new("open")
        .arg("-a")
        .arg(&app_name)
        .status()
        .with_context(|| format!("Gagal mengeksekusi 'open -a {app_name}'"))?;

    if !status.success() {
        bail!("Aplikasi '{app_name}' tidak ditemukan atau gagal dibuka");
    }

    println!("Aplikasi '{app_name}' berhasil dibuka.");
    Ok(())
}

/// Menangani penyalinan item dengan meminta sumber dan tujuan jika tidak disediakan di argumen CLI.
fn handle_copy(source: Option<PathBuf>, destination: Option<PathBuf>) -> Result<()> {
    let src = match source {
        Some(path) => path,
        None => {
            let input: String = Input::new()
                .with_prompt("Jalur sumber (file atau folder)")
                .interact_text()?;
            PathBuf::from(input.trim())
        }
    };

    let dst = match destination {
        Some(path) => path,
        None => {
            let input: String = Input::new()
                .with_prompt("Jalur tujuan")
                .interact_text()?;
            PathBuf::from(input.trim())
        }
    };

    copy_item(&src, &dst)
}

/// Menangani pemindahan atau penggantian nama item dengan fallback input interaktif.
fn handle_move(source: Option<PathBuf>, destination: Option<PathBuf>) -> Result<()> {
    let src = match source {
        Some(path) => path,
        None => {
            let input: String = Input::new()
                .with_prompt("Jalur sumber (file atau folder)")
                .interact_text()?;
            PathBuf::from(input.trim())
        }
    };

    let dst = match destination {
        Some(path) => path,
        None => {
            let input: String = Input::new()
                .with_prompt("Jalur tujuan")
                .interact_text()?;
            PathBuf::from(input.trim())
        }
    };

    move_item(&src, &dst)
}

/// Menangani penghapusan file atau folder dengan dialog konfirmasi ya atau tidak.
fn handle_del(target: Option<PathBuf>) -> Result<()> {
    let path = match target {
        Some(p) => p,
        None => {
            let input: String = Input::new()
                .with_prompt("Jalur file atau folder yang ingin dihapus")
                .interact_text()?;
            PathBuf::from(input.trim())
        }
    };

    if !path.exists() {
        bail!("Jalur target tidak ditemukan: {}", path.display());
    }

    let confirmation = Confirm::new()
        .with_prompt(format!("Yakin ingin menghapus '{}'?", path.display()))
        .default(false)
        .interact()?;

    if !confirmation {
        println!("Operasi penghapusan dibatalkan.");
        return Ok(());
    }

    delete_item(&path)
}

/// Membuat direktori bersarang menggunakan fs::create_dir_all.
fn make_directory(path: &Path) -> Result<()> {
    fs::create_dir_all(path)
        .with_context(|| format!("Gagal membuat direktori '{}'", path.display()))?;

    println!("Direktori berhasil dibuat: {}", path.display());
    Ok(())
}

/// Membuat file kosong baru serta membuat direktori induknya bila belum tersedia.
fn create_empty_file(path: &Path) -> Result<()> {
    ensure_parent_exists(path)?;

    fs::File::create_new(path)
        .with_context(|| format!("Gagal membuat file '{}' (file mungkin sudah ada)", path.display()))?;

    println!("File berhasil dibuat: {}", path.display());
    Ok(())
}

/// Menyalin file atau memanggil penyalinan direktori rekursif jika sumber adalah folder.
fn copy_item(source: &Path, destination: &Path) -> Result<()> {
    if !source.exists() {
        bail!("Sumber tidak ditemukan: {}", source.display());
    }

    if source.is_dir() {
        copy_directory_recursive(source, destination)?;
    } else {
        ensure_parent_exists(destination)?;
        fs::copy(source, destination).with_context(|| {
            format!(
                "Gagal menyalin file dari '{}' ke '{}'",
                source.display(),
                destination.display()
            )
        })?;
    }

    println!(
        "Berhasil menyalin '{}' ke '{}'",
        source.display(),
        destination.display()
    );
    Ok(())
}

/// Menyalin pohon direktori secara rekursif ke lokasi tujuan.
fn copy_directory_recursive(source: &Path, destination: &Path) -> Result<()> {
    fs::create_dir_all(destination).with_context(|| {
        format!(
            "Gagal membuat direktori tujuan '{}'",
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

/// Memindahkan atau mengganti nama berkas atau folder ke lokasi tujuan.
fn move_item(source: &Path, destination: &Path) -> Result<()> {
    if !source.exists() {
        bail!("Sumber tidak ditemukan: {}", source.display());
    }

    ensure_parent_exists(destination)?;

    fs::rename(source, destination).with_context(|| {
        format!(
            "Gagal memindahkan '{}' ke '{}'",
            source.display(),
            destination.display()
        )
    })?;

    println!(
        "Berhasil memindahkan '{}' ke '{}'",
        source.display(),
        destination.display()
    );
    Ok(())
}

/// Menghapus file tunggal atau menghapus seluruh direktori bersarang.
fn delete_item(path: &Path) -> Result<()> {
    if path.is_dir() {
        fs::remove_dir_all(path)
            .with_context(|| format!("Gagal menghapus direktori '{}'", path.display()))?;
        println!("Direktori berhasil dihapus: {}", path.display());
    } else {
        fs::remove_file(path)
            .with_context(|| format!("Gagal menghapus file '{}'", path.display()))?;
        println!("File berhasil dihapus: {}", path.display());
    }

    Ok(())
}

/// Memastikan direktori induk dari suatu jalur telah tersedia sebelum operasi berkas dilakukan.
fn ensure_parent_exists(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
        && !parent.exists()
    {
        fs::create_dir_all(parent).with_context(|| {
            format!("Gagal membuat direktori induk '{}'", parent.display())
        })?;
    }
    Ok(())
}

/// Mengosongkan tampilan layar terminal.
fn clear_terminal() -> Result<()> {
    if Command::new("clear").status().is_ok() {
        return Ok(());
    }

    print!("\x1B[2J\x1B[1;1H");
    std::io::stdout()
        .flush()
        .context("Gagal melakukan flush ke stdout")?;

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
