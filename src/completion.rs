use std::io;

use anyhow::{Result, bail};
use clap::CommandFactory;
use clap_complete::{Shell, generate};

use crate::Cli;

/// Generates shell completion script to stdout for the specified shell
pub fn generate_completion(shell_name: &str) -> Result<()> {
    let shell = match shell_name.to_lowercase().as_str() {
        "zsh" => Shell::Zsh,
        "bash" => Shell::Bash,
        "fish" => Shell::Fish,
        "powershell" | "pwsh" => Shell::PowerShell,
        "elvish" => Shell::Elvish,
        other => bail!(
            "unsupported shell '{other}'. Supported shells: zsh, bash, fish, powershell, elvish"
        ),
    };

    let mut cmd = Cli::command();
    generate(shell, &mut cmd, "run", &mut io::stdout());
    Ok(())
}
