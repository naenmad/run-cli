# Command Reference & Tutorial: `run` CLI

`run` is a clean, human-friendly macOS productivity CLI built in Rust. Every command provides a full semantic English verb/noun, an exact 3-letter alias, and standard Unix alias compatibility for effortless workflow integration.

---

## Command Reference Matrix

| Semantic Command | 3-Letter Alias | Unix Aliases | Purpose |
| :--- | :--- | :--- | :--- |
| `make` | `mak` | `touch`, `mkdir` | Create folders or empty files with parent auto-creation |
| `remove` | `rmv` | `rm`, `del`, `dlt` | Safely delete files or directories with confirmation |
| `copy` | `cpy` | `cp` | Copy files or directory trees recursively |
| `move` | `mov` | `mv` | Move or rename files and directories |
| `list` | `lst` | `ls` | List directory contents with formatted sizes |
| `path` | `pth` | `pwd` | Print or copy current working directory path |
| `go` | `jmp` | `cd`, `nav` | Smart directory jump, root/back shortcuts, and fuzzy hub navigation |
| `read` | `red` | `cat` | Inspect file contents directly |
| `find` | `fnd` | `grep`, `search` | Search pattern or text across files recursively |
| `permit` | `prm` | `chmod` | Change file permissions with presets (755, 644, +x) |
| `process` | `prc` | `ps`, `top` | Inspect active processes or display resource usage snapshot |
| `kill` | `kil` | `stop`, `stp` | Terminate process with search & confirmation |
| `disk` | `dsk` | `df`, `du` | Inspect disk free space or directory usage |
| `whoami` | `who` | `user`, `usr` | Display user identity, UID, GID, and hostname |
| `time` | `tim` | `date`, `dat` | Display current date and time |
| `history` | `his` | - | Display recent shell command history |
| `which` | `whc` | `loc` | Locate executable binary in system PATH |
| `env` | `env` | - | Inspect or search environment variables |
| `port` | `prt` | `lsof` | Check active listening TCP ports and sockets |
| `fetch` | `fch` | `curl`, `wget`, `get` | Fetch HTTP response or download file locally with progress |
| `ping` | `png` | - | Test network host latency |
| `pack` | `pck` | `tar`, `zip` | Create compressed archive (.tar.gz or .zip) |
| `unpack` | `upk` | `unzip`, `untar` | Extract compressed archive (.zip or .tar.gz) |
| `project` | `prj` | - | Scan workspace projects and open in IDE or terminal |
| `dev` | `dev` | - | Run active project development server |
| `build` | `bld` | - | Compile active project in release mode |
| `open` | `opn` | - | Launch macOS applications (via `open -a`) |
| `clear` | `clr` | - | Clear terminal screen |
| `init` | `ini` | - | Generate shell integration wrapper for `~/.zshrc` |
| `help` | `doc` | `guide` | Interactive terminal documentation |

---

## Smart Interaction, Prefix Matching & Anti-Typo

`run` is designed to delight users with effortless and fast interactions:

1. **Bare `run` Fallback Menu:**
   - Running `run` without any arguments automatically displays an interactive selection menu listing all available commands and their 3-letter aliases with descriptions.
   - Selecting any command seamlessly launches into its interactive mode.
   - The last option is always `Cancel`.

2. **Smart Prefix Matching:**
   - Type faster with unique prefixes:
     - `run pro` or `run proj` immediately executes `run project`.
     - `run proc` immediately executes `run process`.
     - `run cl` immediately executes `run clear`.
     - `run g` immediately executes `run go`.

3. **Ambiguous Prefix Disambiguation:**
   - If multiple commands share a prefix, `run` opens an interactive disambiguation menu:
     - `run p` lists `pack`, `path`, `permit`, `ping`, `port`, `process`, `project`, and `Cancel`.
     - `run m` lists `make`, `move`, and `Cancel`.
     - `run f` lists `fetch`, `find`, and `Cancel`.
   - Arguments passed after the prefix are preserved (e.g. `run p 3000` -> pick `port` -> executes `run port 3000`).

4. **Anti-Typo Fuzzy Suggestions:**
   - If a typo is entered (e.g. `run pak` or `run fethc`), `run` calculates Levenshtein distance against command names and aliases.
   - It presents a "Did you mean:" menu with the closest matches ordered by relevance, plus `Cancel`.

---

## Cancellation Standard

All interactive prompts and menus adhere to a strict cancellation standard:
- **Interactive selection menus** (`make`, `go`, `project`, `kill`, `disk`, `port`, `pack`, `unpack`, `permit`, `ping`): The last option is always `Cancel`. Selecting it immediately aborts the action and prints `Cancelled.`.
- **Text prompts** (`open`, `copy`, `move`, `remove`, `find`, `fetch`, `which`): Configured with `(leave blank to cancel)`. Pressing Enter on empty input aborts without changes.
- **Confirmation dialogs** (`remove`, `kill`): Defaults to `[y/N]` (No). Pressing Enter or `n` cancels immediately.

---

## Usage Examples

### 1. Workspace & Development
```bash
# Scan hubs and choose project + IDE (VS Code, Cursor, Xcode, Terminal)
run prj

# Open current directory in IDE picker
run prj .

# Run development server (auto-detects Cargo.toml, package.json, flutter, etc.)
run dev

# Compile active project
run bld
```

### 2. Filesystem & Navigation
```bash
# Jump to user home
run jmp root

# Step back to parent directory
run jmp back

# Fuzzy jump to project or subfolder
run jmp run-cli

# Print current directory (or run pth -i to copy to clipboard)
run pth

# List files and folders with sizes
run lst
run lst -a -l

# Create folder or file (unified mkdir & touch: direct path or typed)
run mak src/utils/token.ts
run mak src/utils/
run mak folder src/utils
run mak file src/utils/token.ts

# Inspect file contents (unified cat)
run red Cargo.toml

# Search text across files (unified grep)
run fnd "handle_project" src/

# Safely delete file or directory (unified rm & del)
run rmv target_file.txt
```

### 3. System & Processes
```bash
# Inspect processes or view resource snapshot
run prc
run prc -s

# Terminate process with search and confirmation
run kil 1234
run kil Safari

# Check disk space and folder usage (unified df & du)
run dsk
run dsk src/

# Display current user and host
run who

# Display current date and time
run tim

# Search environment variables
run env PATH
```

### 4. Networking & Archives
```bash
# Check what is listening on port 3000 (unified lsof)
run prt 3000

# Download file or fetch HTTP response (unified curl & wget)
run fch https://api.github.com
run fch https://example.com/archive.zip -o output.zip

# Ping host
run png 1.1.1.1

# Create archive (unified tar & zip)
run pck backup.tar.gz src/

# Extract archive (unified unzip & untar)
run upk backup.tar.gz
```
