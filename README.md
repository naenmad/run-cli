# run-cli (`run`)

<p align="center">
  <img src="https://raw.githubusercontent.com/naenmad/run-cli/main/assets/banner.png" alt="run-cli banner" width="600" onerror="this.style.display='none'"/>
</p>

<p align="center">
  <strong>Lightning-fast, intuitive macOS productivity CLI with dual-mode interaction, smart anti-typo resolution, and built-in developer workflows.</strong>
</p>

<p align="center">
  <a href="https://github.com/naenmad/run-cli/releases"><img src="https://img.shields.io/github/v/release/naenmad/run-cli?style=flat-square&color=00a2ff" alt="Latest Release"></a>
  <a href="https://github.com/naenmad/run-cli/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/naenmad/run-cli/ci.yml?branch=main&style=flat-square&label=CI" alt="Build Status"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square" alt="License: MIT"></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/rust-2024%20edition-orange.svg?style=flat-square" alt="Rust Edition"></a>
  <a href="#"><img src="https://img.shields.io/badge/platform-macOS-lightgrey.svg?style=flat-square" alt="Platform macOS"></a>
</p>

---

## ⚡ Why `run`?

Standard Unix commands are cryptic and filled with archaic flags (`lsof -iTCP -sTCP:LISTEN -P`, `tar -czvf`, `find . -name`, `kill -9`). 

`run` reimagines terminal productivity for modern developers:
* **Semantic Verbs + 3-Letter Aliases**: Every action is intuitive (`make`/`mak`, `remove`/`rmv`, `port`/`prt`, `fetch`/`fch`, `clean`/`cln`).
* **Dual-Mode Execution**: Run with direct arguments for lightning execution, or omit arguments to trigger an interactive prompt menu.
* **Anti-Typo & Smart Auto-Completion**: Type `run pro` to jump straight to `project`. Mistyped `run fethc`? Levenshtein distance automatically detects `fetch`.
* **Zero-Hesitation Workspace Hub**: Auto-scans all development folders (`~/Developer`, `~/Projects`, etc.) and opens them in VS Code, Cursor, Xcode, or switches active terminal directories in-place.
* **Modern Developer Suite**: Out-of-the-box runners for testing, cache cleaning, 1-step Git syncing, LAN file sharing, network inspection, and microsecond command benchmarking.
* **Human-Centric Safety**: Safe defaults, confirm-before-delete, and prominent **Red Cancel** options in all interactive menus.

---

## 🚀 Quick Install

### Option 1: Install via Homebrew (Recommended)

```bash
brew install naenmad/run-cli/run
```

### Option 2: Install via Cargo

If you have Rust installed:

```bash
cargo install --git https://github.com/naenmad/run-cli
```

### Option 3: Pre-compiled GitHub Release (macOS Apple Silicon & Intel)

Download and install the latest binary with a single terminal command:

```bash
# Apple Silicon (M1/M2/M3/M4)
curl -fsSL https://github.com/naenmad/run-cli/releases/latest/download/run-macos-aarch64.tar.gz | tar -xz -C /usr/local/bin run

# Intel Mac
curl -fsSL https://github.com/naenmad/run-cli/releases/latest/download/run-macos-x86_64.tar.gz | tar -xz -C /usr/local/bin run
```

*(If `/usr/local/bin` requires root, use `sudo` or extract into `~/.local/bin`)*.

### Option 4: Build from Source

```bash
git clone https://github.com/naenmad/run-cli.git
cd run-cli
cargo build --release
sudo cp target/release/run /usr/local/bin/
```

---

## ⚙️ Shell Integration & Auto-Completion

`run` includes automatic in-place directory switching (`run go`, `run project`) and native tab completion (`zsh`, `bash`, `fish`).

Add this single line to your `~/.zshrc` (or `~/.bashrc`):

```bash
eval "$(run init)"
```

Then reload your shell:
```bash
source ~/.zshrc
```

Now `run go root`, `run go back`, or selecting a project with **"Switch Terminal Directory Only"** will change your terminal's directory instantly, and pressing **Tab** after `run ` will auto-complete all commands and flags!

---

## 🛠️ Configuration (`~/.config/run/config.toml`)

`run` auto-creates a config file at `~/.config/run/config.toml` (or manage it interactively via `run config`):

```toml
# Primary theme accent color:
# Presets: "electric-blue" (default), "violet", "emerald", "amber", "rose", "cyan"
# Or use any custom HEX color code: "#ff007f", "#8b5cf6", "#10b981", "#00a2ff"
primary_color = "electric-blue"

# Automatically clear terminal screen before running commands and interactive menus
auto_clear = false

# Compact mode (sleek minimalist layout without large ASCII banners)
compact_mode = false

# Default editor to open projects directly without prompting:
# Options: "antigravity", "cursor", "vscode", "xcode", "terminal", "ask" (default)
default_ide = "ask"

# Additional custom directory hubs to scan in `run project`:
custom_hubs = [
    "~/Developer/Summit",
    "~/Work"
]
```

---

## 🎯 Command Cheatsheet

`run` includes **68 productivity commands** categorized below. Every single command supports its full semantic name and exact 3-letter alias:

### 🍏 macOS Native Interaction Suite (Dual-Mode: Direct & Interactive)
| Command | Alias | Additional Aliases | Purpose & Highlights |
| :--- | :--- | :--- | :--- |
| `wifi` | `wif` | - | Manage Wi-Fi, scan nearby networks, connect, toggle power, or view Keychain passwords (`run wifi`, `run wif scan`, `run wif pass`) |
| `bluetooth` | `blt` | `bt`, `blue` | Inspect Bluetooth controller state & connected peripherals (`run bt`, `run bt on`) |
| `airpods` | `pod` | `pods` | One-click connect paired AirPods or Bluetooth headphones (`run pods`) |
| `airdrop` | `drp` | `drop` | Open AirDrop finder window or share files via native macOS Share Sheet (`run drop [file]`) |
| `music` | `msc` | `spotify` | Apple Music & Spotify player control (`play`, `pause`, `next`, `prev`) and track HUD (`run music`) |
| `volume` | `vol` | `sound` | Audio volume HUD & slider (0-100%, mute/unmute) (`run vol 50`, `run vol mute`) |
| `note` | `not` | `notes` | Quick scratchpad saver directly into Apple Notes (`run note "Idea"`, `run note`) |
| `fixapp` | `fix` | `xattr` | Fix Gatekeeper quarantine ("App is damaged and can't be opened") with fuzzy app selector (`run fixapp Figma`, `run fix`) |
| `battery` | `bat` | `batt` | Detailed battery health, cycle count, max capacity %, and charger wattage (`run bat`) |
| `awake` | `caf` | `caffeinate` | Keep Mac awake and prevent display/system sleep with countdown timer (`run awake 60`, `run caf`) |
| `peek` | `pek` | `ql`, `quicklook` | Launch native macOS QuickLook preview popup for any document or image (`run peek image.png`) |
| `trash` | `tsh` | - | Inspect Trash disk usage or safely empty trash (`run trash`, `run tsh empty`) |
| `shot` | `snt` | `snip`, `screenshot` | Capture screen selection or window directly to clipboard (`run shot`, `run snip window`) |
| `notify` | `ntf` | `alert` | Dispatch native macOS notification banner with sound (`run notify "Build" "Finished"`) |
| `dark` | `drk` | - | Toggle or set macOS Dark Mode (`run dark`, `run drk on`) |
| `light` | `lit` | - | Set macOS Light Mode (`run light`) |
| `lock` | `lok` | - | Lock macOS screen immediately (`run lock`) |
| `desktop` | `dkt` | `desk` | Hide or show desktop icons for clean presentations (`run desk hide`, `run desk show`) |
| `dns` | `fls` | `flush` | Flush macOS DNS cache in 1 click (`dscacheutil` + `mDNSResponder`) (`run dns`) |
| `voice` | `say` | `voc` | Native macOS Text-to-Speech synthesis with fun voices (`run voice "Hello"`, `run say "Hi" -v Zarvox`) |

### 🛠️ Developer & Workspace Suite
| Command | Alias | Purpose & Highlights |
| :--- | :--- | :--- |
| `project` | `prj` | Scan project directories & open in **Antigravity**, **VS Code**, **Cursor**, **Xcode**, or terminal |
| `dev` | `dev` | Auto-detects project stack (`cargo`, `npm`, `pnpm`, `flutter`, `go`) and starts dev server |
| `build` | `bld` | Compiles active project in release mode |
| `test` | `tst` | Polyglot automated test runner (`cargo test`, `npm test`, `pytest`, `flutter test`, `go test`) |
| `clean` | `cln` | Scans and deletes disposable caches (`target/`, `node_modules/`, `.next/`, `__pycache__/`, `DerivedData/`) with size report & confirmation |
| `sync` | `snc`, `git` | 1-step Git pull, status review, commit message prompt, and push |
| `docker` | `dck` | Inspect and manage Docker/OrbStack containers, view logs, start or stop |
| `secret` | `sec`, `dotenv` | Audit local `.env` against `.env.example` and generate sanitized templates (`--fix`) |
| `config` | `cfg` | Manage CLI settings, custom primary colors, editor & auto-clear (`run cfg`) |
| `alias` | `als`, `shortcut` | Define custom command shortcuts (`run alias add c "cargo check"`) with execution fallback |
| `stats` | `sts`, `analytics` | Terminal usage analytics dashboard with visual frequency bars (`run stats`) |
| `timer` | `tmr`, `pomo` | Focus Pomodoro / countdown timer with live progress bar and completion chime alert (`run timer 25`) |
| `uuid` | `uid` | Generate UUID v4 and automatically copy to system clipboard (`run uuid`) |
| `pass` | `pas`, `password` | Generate cryptographically secure random password/API token and copy to clipboard (`run pass 24`) |
| `update` | `upd`, `upgrade` | 1-step updater for Homebrew, Rust toolchain, and global Node packages (`run upd`) |
| `network` | `net`, `ip` | Inspects local LAN (en0/en1) and public WAN IP with interactive clipboard copy |
| `share` | `shr` | Instant local HTTP file server on LAN (`run share -p 8080`) |
| `bench` | `bnc` | Microsecond-precision command execution benchmark timer |

### 📂 Filesystem & Navigation
| Command | Alias | Unix Alias | Purpose |
| :--- | :--- | :--- | :--- |
| `go` | `jmp` | `cd`, `nav` | Fast jump to root (`~`), back (`..`), subdirectories, or fuzzy hub picker |
| `path` | `pth` | `pwd` | Print or copy current working directory to clipboard |
| `list` | `lst` | `ls` | Colorized directory listing with human-readable file sizes |
| `make` | `mak` | `touch`, `mkdir` | Unified creation of files or folders with automatic parent directory creation |
| `remove` | `rmv` | `rm`, `del` | Safely remove files or directories with confirmation prompt (`y/N`) |
| `copy` | `cpy` | `cp` | Copy files or directories recursively |
| `move` | `mov` | `mv` | Move or rename files and directories |
| `read` | `red` | `cat` | Inspect file contents directly in terminal |
| `find` | `fnd` | `grep`, `search` | Recursive pattern and text search across files |
| `permit` | `prm` | `chmod` | Modify file permissions using human presets (`755`, `644`, `+x`) |
| `memo` | `mem`, `clip` | - | Developer quick scratchpad & snippet clipboard manager |

### ⚙️ System & Process Management
| Command | Alias | Unix Alias | Purpose |
| :--- | :--- | :--- | :--- |
| `process` | `prc` | `ps`, `top` | List active processes or view system resource snapshot (CPU & Memory) |
| `kill` | `kil` | `stop`, `stp` | Terminate process by PID or name with safe confirmation |
| `disk` | `dsk` | `df`, `du` | Mounted volume free space or interactive directory disk usage selector |
| `port` | `prt`, `killport` | `lsof` | Inspect active TCP listening ports, detect conflicts, or kill blocking processes (`run port 8000 -k`) |
| `whoami` | `who` | `user` | Display user identity, UID, GID, and system hostname |
| `time` | `tim` | `date` | Formatted current date and time |
| `history` | `his` | - | Display recent shell command history |
| `which` | `whc` | `loc` | Locate executable binary path in system `$PATH` |
| `env` | `env` | - | Inspect or search environment variables |

### 🌐 Network & Archive
| Command | Alias | Unix Alias | Purpose |
| :--- | :--- | :--- | :--- |
| `qr` | `qrc` | - | Render terminal visual Unicode QR code from text, URL, or clipboard (`run qr`) |
| `speedtest` | `spd`, `speed` | `networkQuality` | Measure internet download, upload throughput and responsiveness |
| `fetch` | `fch` | `curl`, `wget` | Fetch HTTP response headers/body or download files with progress bar |
| `ping` | `png` | - | Test host latency with packet summary |
| `pack` | `pck` | `tar`, `zip` | Create compressed `.tar.gz` or `.zip` archive |
| `unpack` | `upk` | `unzip`, `untar` | Extract `.tar.gz` or `.zip` archives |

### 💡 General Utilities
| Command | Alias | Purpose |
| :--- | :--- | :--- |
| `open` | `opn` | Launch macOS applications via native `open -a` |
| `clear` | `clr` | Clear terminal screen |
| `init` | `ini` | Output shell wrapper script and auto-completion for `~/.zshrc` |
| `completion` | `cmp` | Generate native shell auto-completion (`zsh`, `bash`, `fish`) |
| `help` | `doc`, `guide` | Interactive terminal documentation and detailed guides (`run help <cmd>`) |

---

## 🧠 Interactive Intelligence

### 1. Bare `run` Command Menu
Running just `run` opens an interactive launcher listing all available tools. Use arrow keys to select and run:

```bash
$ run
? Select a command to run ›
❯ project       (prj)    Scan projects & open in IDE or terminal
  dev                    Run active project dev server
  build         (bld)    Build or compile active project
  test          (tst)    Smart polyglot test runner (Rust, Node, Python, etc.)
  clean         (cln)    Clean disposable build artifacts and cache folders
  sync          (snc)    Interactive 1-step Git pull, commit, and push
  network       (net)    Inspect local LAN and public IP with quick copy
  ...
  Cancel
```

### 2. Smart Prefix & Fuzzy Anti-Typo
* **Shortcuts**: `run pro` immediately expands to `run project` because it's a unique prefix.
* **Disambiguation**: `run p` lists all commands starting with `p` (`project`, `path`, `process`, `port`, `pack`, `permit`, `ping`).
* **Typo Correction**: Mistyped `run fethc` or `run pak`? `run` suggests or auto-selects the nearest matching command via Levenshtein distance.

---

## 🤝 Contributing

Contributions, bug reports, and suggestions are welcome!

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Ensure all tests and clippy pass:
   ```bash
   cargo test
   cargo clippy --all-targets -- -D warnings
   ```
5. Push to your branch (`git push origin feature/amazing-feature`)
6. Open a Pull Request

---

## 📄 License

Distributed under the MIT License. See [LICENSE](LICENSE) for details.
