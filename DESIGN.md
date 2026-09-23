# Design & Rules: CLI `run`

This document defines the design guidelines, command naming conventions, three-letter alias rules, and the dual-mode interaction philosophy for the `run` CLI utility.

## 1. Core Philosophy
* **Human-Friendly First**: Replaces complex Unix flag syntax (`mkdir -p`, `cp -r`) with intuitive natural English verbs.
* **Dual-Mode Execution**: Supports fast direct command-line arguments for power users alongside interactive step-by-step terminal prompts when arguments are omitted.
* **Ergonomic Design**: Optimized for typing speed and reduced finger fatigue.

## 2. Command Standardization and 3-Letter Alias Rules
All primary commands in the `run` CLI follow two requirements:
1. **Full Command Name**: Uses clear, standard English verbs (`make`, `open`, `copy`, etc.).
2. **Mandatory 3-Letter Alias**: Every command must have an alias consisting of exactly three letters for rapid typing.

Command mapping:

| Full Command | Alias (3 Letters) | Primary Function | Example Usage |
| :--- | :--- | :--- | :--- |
| `make` | `mak` | Create a folder or file (dual-mode) | `run make folder notes` / `run mak` |
| `open` | `opn` | Open a macOS application | `run open Safari` / `run opn Code` |
| `copy` | `cpy` | Copy files or directories | `run copy file.txt backup.txt` |
| `move` | `mov` | Move or rename files and directories | `run move old.txt new.txt` |
| `del`  | `dlt` | Safely delete files or directories | `run del temp/` |
| `clear`| `clr` | Clear the terminal screen | `run clr` |
| `go`   | `jmp` | Smart folder navigation (root, back, subfolder, picker) | `run go root` / `run jmp` |
| `help` | `guide`| Display complete documentation and tutorial | `run help` / `run help go` |

## 3. Dual-Mode Interaction Rules
* **Direct Mode**: When arguments are provided in full (such as `run make folder project-x`), the CLI executes the command immediately without interactive pauses.
* **Interactive Mode**: When only the command name or alias is entered (such as `run make` or `run mak`), the CLI presents an interactive prompt to collect the necessary input.

## 4. Safety and Error Handling
* **Safe Deletion**: Deletion commands (`del` / `dlt`) require explicit user confirmation before removing any path permanently.
* **Clear Error Reporting**: Operating system and filesystem errors are formatted as readable diagnostics without surfacing raw stack traces.
