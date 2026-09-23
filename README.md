# run

`run` is a command-line productivity utility for macOS built around human-friendly verbs and dual-mode interaction.

## Core Philosophy

* **Human-Friendly First**: Replaces standard Unix command flags with direct, natural verbs that reflect common daily tasks.
* **Dual-Mode Execution**: Accepts direct CLI arguments for quick execution, or drops into an interactive prompt menu when arguments are omitted.
* **Ergonomic**: Standardized three-letter aliases across all commands reduce keystrokes and typing friction.

## Commands and Aliases

Every primary command supports both its full English verb and a mandatory three-letter alias:

| Command | Alias (3 Letters) | Description | Example Usage |
| :--- | :--- | :--- | :--- |
| `make` | `mak` | Create a folder or empty file (dual-mode) | `run make folder notes` / `run mak` |
| `open` | `opn` | Open a macOS application via `open -a` | `run open Safari` / `run opn Code` |
| `copy` | `cpy` | Copy files or directories | `run copy file.txt backup.txt` |
| `move` | `mov` | Move or rename files and directories | `run move old.txt new.txt` |
| `del` | `dlt` | Safely delete files or directories | `run del temp/` |
| `clear` | `clr` | Clear the terminal screen | `run clr` |

## Dual-Mode Interaction

* **Direct Mode**: When you provide complete arguments (such as `run make folder project-x` or `run cpy notes.txt backup.txt`), the command executes immediately without prompt pauses.
* **Interactive Mode**: When you type only the command or its alias (such as `run make` or `run opn`), an interactive prompt guides you through options and required inputs.

## Safety and Error Handling

* **Safe Deletion**: The `del` / `dlt` command always requires confirmation before deleting files or directories.
* **Clear Error Messages**: Underlying operating system and process failures display clean, readable status notes rather than raw stack traces.

## Installation

### Prerequisites
* Rust toolchain (1.80 or newer recommended)
* macOS

### Build Release Binary
```bash
cargo build --release
```

The compiled executable is placed at `target/release/run`.

### Install to System PATH
To run `run` globally from any directory:
```bash
cargo install --path .
```

Verify the installation:
```bash
run --help
```

## Running Tests

```bash
cargo test
```

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.
