# Complete Command Reference: `run` CLI

`run` is a clean, human-friendly macOS productivity CLI built in Rust. Every command provides a full semantic English verb/noun, an exact 3-letter alias, and standard Unix alias compatibility for effortless workflow integration.

---

## Command Reference Matrix (41 Commands)

### 🛠️ Developer & Workspace Suite

| Semantic Command | 3-Letter Alias | Additional Aliases | Purpose |
| :--- | :--- | :--- | :--- |
| `project` | `prj` | - | Scan workspace hubs and launch in Antigravity, VS Code, Cursor, Xcode, or terminal |
| `dev` | `dev` | `develop` | Auto-detect stack (Rust, Node, Flutter, Go) and start development server |
| `build` | `bld` | - | Compile active project in release mode |
| `test` | `tst` | - | Polyglot automated test runner (`cargo test`, `npm test`, `pytest`, `flutter test`, `go test`) |
| `clean` | `cln` | - | Scan and clean disposable build artifacts (`target/`, `node_modules/`, `.next/`, `__pycache__/`, `DerivedData/`) with size confirmation |
| `sync` | `snc` | `git` | 1-step Git pull, status review, commit message prompt, and push |
| `docker` | `dck` | - | Inspect and manage Docker/OrbStack containers, view logs, start or stop |
| `secret` | `sec` | `dotenv` | Audit local `.env` variables against `.env.example` and generate sanitized templates (`--fix`) |
| `config` | `cfg` | - | Manage CLI settings, custom primary colors, editor & auto-clear (`run cfg`) |
| `network` | `net` | `ip` | Inspect local LAN and public WAN IP addresses with interactive copy to clipboard |
| `share` | `shr` | - | Instant local HTTP file server on local network (`run share -p 8080`) |
| `bench` | `bnc` | - | Benchmark command execution duration with high-resolution microsecond timer |

### 📂 Filesystem & Navigation

| Semantic Command | 3-Letter Alias | Unix Aliases | Purpose |
| :--- | :--- | :--- | :--- |
| `make` | `mak` | `touch`, `mkdir` | Create folders or empty files with parent auto-creation |
| `remove` | `rmv` | `rm`, `del`, `dlt`, `delete` | Safely delete files or directories with confirmation prompt (`y/N`) |
| `copy` | `cpy` | `cp` | Copy files or directory trees recursively |
| `move` | `mov` | `mv` | Move or rename files and directories |
| `list` | `lst` | `ls` | List directory contents with formatted sizes and colorized directory badges |
| `path` | `pth` | `pwd` | Print or copy current working directory path |
| `go` | `jmp` | `cd`, `nav`, `g` | Smart directory navigation, root/back shortcuts, and fuzzy hub navigation |
| `read` | `red` | `cat` | Inspect file contents directly |
| `find` | `fnd` | `grep`, `search` | Search pattern or text across files recursively |
| `permit` | `prm` | `chmod` | Change file permissions with presets (`755`, `644`, `+x`) |
| `memo` | `mem` | `clip` | Developer scratchpad and snippet clipboard manager (`add`, `copy`, `rm`, `clear`) |

### ⚙️ System & Process Management

| Semantic Command | 3-Letter Alias | Unix Aliases | Purpose |
| :--- | :--- | :--- | :--- |
| `process` | `prc` | `ps`, `top`, `proc` | Inspect active processes or display resource usage snapshot (CPU & Memory) |
| `kill` | `kil` | `stop`, `stp` | Terminate process with search & confirmation |
| `disk` | `dsk` | `df`, `du` | Inspect disk free space or directory usage |
| `whoami` | `who` | `user`, `usr` | Display user identity, UID, GID, and hostname |
| `time` | `tim` | `date`, `dat` | Display current date and time |
| `history` | `his` | - | Display recent shell command history |
| `which` | `whc` | `loc` | Locate binary executable in system `$PATH` |
| `env` | `env` | - | Inspect or search environment variables |

### 🌐 Networking & Archives

| Semantic Command | 3-Letter Alias | Unix Aliases | Purpose |
| :--- | :--- | :--- | :--- |
| `speedtest` | `spd` | `speed`, `networkQuality` | Measure internet download, upload throughput and responsiveness |
| `port` | `prt` | `lsof` | Check active listening TCP ports and sockets |
| `fetch` | `fch` | `curl`, `wget`, `get` | Fetch HTTP response or download file locally with progress bar |
| `ping` | `png` | - | Test network host latency |
| `pack` | `pck` | `tar`, `zip` | Create compressed archive (`.tar.gz` or `.zip`) |
| `unpack` | `upk` | `unzip`, `untar` | Extract compressed archive (`.zip` or `.tar.gz`) |

### 💡 General Utilities

| Semantic Command | 3-Letter Alias | Additional Aliases | Purpose |
| :--- | :--- | :--- | :--- |
| `open` | `opn` | - | Launch macOS applications (via `open -a`) |
| `clear` | `clr` | - | Clear terminal screen |
| `init` | `ini` | - | Generate shell integration wrapper and auto-completion for `~/.zshrc` |
| `completion` | `cmp` | - | Generate native shell completion scripts (`zsh`, `bash`, `fish`) |
| `help` | `doc` | `guide` | Interactive terminal documentation and detailed guides |

---

## Smart Interaction, Prefix Matching & Anti-Typo

`run` is designed to delight users with effortless and fast interactions:

1. **Bare `run` Fallback Menu:**
   - Running `run` without any arguments automatically displays an interactive selection menu listing all available commands and their 3-letter aliases with descriptions.
   - Selecting any command seamlessly launches into its interactive mode.
   - The last option is always **Cancel** highlighted in high-contrast red.

2. **Smart Prefix Matching:**
   - Type faster with unique prefixes:
     - `run pro` or `run proj` immediately executes `run project`.
     - `run proc` immediately executes `run process`.
     - `run cl` immediately executes `run clear`.
     - `run g` immediately executes `run go`.

3. **Ambiguous Prefix Disambiguation:**
   - When a prefix matches multiple commands (e.g. `run p`), `run` opens an interactive menu displaying all matching candidates (`project`, `path`, `process`, `port`, `pack`, `permit`, `ping`).
   - Selecting one immediately continues execution with your remaining arguments intact.

4. **Levenshtein Distance Anti-Typo:**
   - If an unrecognized command is typed (e.g., `run fethc` or `run pak`), `run` calculates edit distances and prompts you with likely matches. If only one command is close, it seamlessly corrects.

---

## Detailed Usage Examples

### 1. Developer Workflows

```bash
# Scan all projects in ~/Developer, ~/Projects, etc. and choose IDE:
run project
# Or with 3-letter alias:
run prj

# Run the active project's dev server:
run dev

# Compile active project in release mode:
run build

# Run automated tests automatically matching Cargo, npm, pytest, flutter, etc.:
run test
run tst

# Clean disposable build caches (target, node_modules, DerivedData):
run clean
run cln

# 1-step Git pull, commit, and push:
run sync
run sync -m "feat: add network inspector"
run snc -b feature/auth

# Inspect local LAN and public WAN IP:
run network
run net

# Instant HTTP file server on local LAN:
run share
run shr -p 8080

# Benchmark command duration:
run bench cargo build
run bnc sleep 1
```

### 2. Filesystem & Navigation

```bash
# Jump to home or parent directory:
run go root
run go back

# Jump to folder or fuzzy search development hubs:
run go my-repo
run jmp

# Create directory or empty file:
run make folder src/utils
run make file src/utils/token.ts
run mak

# Inspect file contents:
run read Cargo.toml
run red

# Search text across files:
run find "handle_project" src/
run fnd

# Safely delete file or directory with confirmation:
run remove target_file.txt
run rmv
```

### 3. System & Processes

```bash
# Inspect processes or view resource snapshot:
run process
run prc -s

# Terminate process with search and confirmation:
run kill 1234
run kil Safari

# Check disk space and folder usage:
run disk
run dsk src/

# Display current user and host:
run whoami
run who

# Display current date and time:
run time
run tim

# Search environment variables:
run env PATH
```

### 4. Networking & Archives

```bash
# Check what is listening on port 3000:
run port 3000
run prt

# Download file or fetch HTTP response:
run fetch https://api.github.com
run fetch https://example.com/archive.zip -o output.zip
run fch

# Ping host:
run ping 1.1.1.1
run png

# Create archive:
run pack backup.tar.gz src/
run pck

# Extract archive:
run unpack backup.tar.gz
run upk
```

### 5. Docker, Secret & Clipboard Memo

```bash
# Manage Docker / OrbStack containers interactively (inspect, start, stop, view logs):
run docker
run dck

# Audit .env variables against .env.example:
run secret
run sec

# Generate sanitized .env.example from current .env file:
run secret --fix
run sec --fix

# Open interactive snippet / scratchpad manager:
run memo
run mem

# Save a quick command or note:
run mem add docker-stop "docker stop \$(docker ps -q)"

# Copy snippet directly to macOS system clipboard:
run mem copy docker-stop

# Remove snippet:
run mem rm docker-stop
```

### 6. Shell Auto-Completion

```bash
# Generate shell completion script:
run completion zsh
run completion bash
run completion fish

# Tip: eval "$(run init)" in ~/.zshrc automatically sources completions!
```

### 7. Configuration & Custom Theme

```bash
# Open interactive configuration dashboard:
run config
run cfg

# Set custom primary accent color (supports presets or arbitrary hex):
run cfg set primary_color "#ff007f"
run cfg set primary_color violet

# Get current configuration value:
run cfg get primary_color
run cfg get default_ide

# Toggle auto-clear terminal screen:
run cfg set auto_clear true

# Print config file path or open in system editor:
run cfg path
run cfg edit

# Reset configuration to factory defaults:
run cfg reset
```

---

## ⚙️ Configuration File (`~/.config/run/config.toml`)

`run` automatically creates a configuration template at `~/.config/run/config.toml` upon first run.

```toml
# Primary theme accent color:
# Presets: "electric-blue" (default), "violet", "emerald", "amber", "rose", "cyan"
# Or use any custom HEX color code: "#ff007f", "#8b5cf6", "#10b981", "#00a2ff"
primary_color = "electric-blue"

# Automatically clear terminal screen before interactive menus:
auto_clear = false

# Default editor to open projects directly without prompting:
# Options: "antigravity", "cursor", "vscode", "xcode", "terminal", "ask" (default)
default_ide = "ask"

# Additional custom directory hubs to scan for projects in `run project`:
custom_hubs = [
    "~/Developer/Summit",
    "~/Work/Projects"
]
```
