# Command Reference & Tutorial: `run` CLI

This document is the complete reference guide and tutorial for the `run` CLI utility.

---

## Command Reference

Every command supports its standard full verb and an exact 3-letter alias.

### 1. `make` (Alias: `mak`)
Creates directories or empty files.

* **Direct Mode:**
  * Create folder (creates parent directories automatically):
    ```bash
    run make folder path/to/new_dir
    # or with alias:
    run mak folder path/to/new_dir
    ```
  * Create empty file (creates parent directories if needed):
    ```bash
    run make file path/to/file.txt
    # or with alias:
    run mak file path/to/file.txt
    ```
* **Interactive Mode:**
  Run without arguments to choose the type and enter the path:
  ```bash
  run mak
  # 1. Select Folder or File using arrow keys
  # 2. Type target path when prompted
  ```

---

### 2. `open` (Alias: `opn`)
Launches macOS applications using the native `open -a` subsystem.

* **Direct Mode:**
  ```bash
  run open Safari
  # or with alias:
  run opn "Visual Studio Code"
  ```
* **Interactive Mode:**
  ```bash
  run opn
  # Prompts: Application name:
  ```

---

### 3. `copy` (Alias: `cpy`)
Copies a file or an entire directory tree.

* **Direct Mode:**
  ```bash
  # Copy file
  run copy document.pdf backup.pdf

  # Copy directory recursively
  run cpy src/ backup_src/
  ```
* **Interactive Mode:**
  ```bash
  run cpy
  # Prompts:
  # Source path:
  # Destination path:
  ```

---

### 4. `move` (Alias: `mov`)
Moves or renames files and directories.

* **Direct Mode:**
  ```bash
  # Rename file
  run move draft.txt final.txt

  # Move folder into another directory
  run mov assets/ public/assets/
  ```
* **Interactive Mode:**
  ```bash
  run mov
  # Prompts:
  # Source path:
  # Destination path:
  ```

---

### 5. `del` (Alias: `dlt`)
Deletes a file or an entire directory tree. Always prompts for confirmation before proceeding.

* **Direct Mode:**
  ```bash
  run del temp.log
  # or with alias:
  run dlt build_output/
  ```
  Even in direct mode, you must confirm:
  ```text
  Delete file 'temp.log'? [y/N]
  ```
* **Interactive Mode:**
  ```bash
  run dlt
  # 1. Enter path to delete
  # 2. Confirm deletion [y/N]
  ```

---

### 6. `clear` (Alias: `clr`)
Clears the terminal viewport.

* **Usage:**
  ```bash
  run clear
  # or with alias:
  run clr
  ```

---

### 7. `go` (Alias: `jmp`, `nav`)
Smart directory navigation that replaces manual `cd` commands.

* **Jump to Home (`root`):**
  ```bash
  run go root
  # or with alias:
  run jmp root
  ```
* **Step Back to Parent Directory (`back`):**
  ```bash
  run go back
  # or with alias:
  run jmp back
  ```
* **Jump to Child or Nested Folder:**
  ```bash
  run go code
  # or with alias:
  run jmp src
  ```
* **Interactive Mode (Folder Picker):**
  Run without arguments to display an interactive menu of subdirectories and home/parent shortcuts:
  ```bash
  run go
  # or with alias:
  run jmp
  ```

> Note: To enable in-place directory switching in your active terminal, add this to your `~/.zshrc`:
> ```bash
> eval "$(run init)"
> ```

---

### 8. `project` (Alias: `prj`)
Smart project scanner and interactive IDE launcher.

* **Global Scan Mode:**
  Scans development hubs (`~/Developer`, `~/Projects`, `~/Code`, `~/Documents`, `~/Desktop`) for projects (`.git`, `Cargo.toml`, `package.json`, `pubspec.yaml`, etc.):
  ```bash
  run project
  # or with alias:
  run prj
  ```
  1. Pick a detected project from the interactive list (or select `Cancel`).
  2. Pick the target editor (`Visual Studio Code`, `Cursor`, `Xcode`, `Hanya Pindah Terminal / Saja`, or `Cancel`).

* **Contextual Current Directory Mode:**
  Open the current directory directly:
  ```bash
  run project .
  # or with alias:
  run prj .
  ```

* **Query Search Mode:**
  Open or jump to a project by name:
  ```bash
  run prj run-cli
  ```

---

### 9. `dev` (Alias: `dev`)
Runs the development server for the current active project based on detected file signatures:
- `Cargo.toml`: `cargo run`
- `package.json`: `pnpm run dev`, `yarn dev`, `bun run dev`, or `npm run dev`
- `pubspec.yaml`: `flutter run`
- `go.mod`: `go run .`
- `Makefile`: `make dev`

* **Usage:**
  ```bash
  run dev
  ```

---

### 10. `build` (Alias: `bld`)
Builds and compiles the current active project based on detected file signatures:
- `Cargo.toml`: `cargo build --release`
- `package.json`: `npm run build` (or pnpm/yarn/bun)
- `pubspec.yaml`: `flutter build`
- `go.mod`: `go build .`
- `Makefile`: `make build`

* **Usage:**
  ```bash
  run build
  # or with alias:
  run bld
  ```

---

### 11. `init` (Alias: `ini`)
Generates the shell integration script for `~/.zshrc` or `~/.bashrc`. Enables in-place directory switching for `go` and `project`.

* **Usage:**
  ```bash
  eval "$(run init)"
  ```

---

### 12. `help` (Alias: `guide`)
Displays this comprehensive reference and tutorial in the terminal.

* **Usage:**
  ```bash
  run help
  # or view specific command documentation:
  run help project
  run help dev
  run help build
  ```

---

## Cancellation Standard

All interactive prompts and menus adhere to a strict cancellation standard:
- **Interactive selection menus** (`make`, `go`, `project`): The last option is always `Cancel`. Selecting it immediately aborts the action and outputs `Cancelled.`.
- **Text prompts** (`open`, `copy`, `move`, `del`): Configured with `(leave blank to cancel)`. Pressing Enter without input immediately aborts the action.
- **Confirmation dialogs** (`del`): Defaults to `[y/N]` (No). Pressing Enter or `n` immediately aborts the action.

---

## Quickstart Tutorial

### Scenario 1: Scaffold a New Project Structure
Use `mak` to create directories and files without worrying about flags:
```bash
# 1. Create nested source folder
run mak folder my_project/src

# 2. Create entry file
run mak file my_project/src/index.js

# 3. Create readme file
run mak file my_project/README.md
```

### Scenario 2: Backup and Reorganize Assets
Use `cpy` and `mov` to duplicate and restructure:
```bash
# 1. Duplicate source folder for backup
run cpy my_project/src my_project/src_backup

# 2. Rename or move documentation
run mov my_project/README.md my_project/DOCS.md
```

### Scenario 3: Clean Up Safely
Use `dlt` to clean up temporary artifacts with built-in safety prompts:
```bash
run dlt my_project/src_backup
# Press 'y' to confirm, or 'n' / Enter to cancel
```

### Scenario 4: Fast App Launching
Open desktop tools straight from your terminal workflow:
```bash
run opn Safari
run opn "Google Chrome"
```

### Scenario 5: Smart Directory Navigation
Quickly jump around directories without typing path slashes or cd:
```bash
# Jump to user home
run go root

# Move into project subfolder
run go src

# Step back to parent folder
run go back

# Open interactive folder picker
run go
```
