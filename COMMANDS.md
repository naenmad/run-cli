# Complete Command Reference: `run` CLI

`run` is a clean, human-friendly macOS productivity CLI built in Rust. Every command provides a full semantic English verb/noun, an exact 3-letter alias, and standard Unix alias compatibility for smooth workflow integration.

---

## Command Reference Matrix (77 Commands)

### 🍏 macOS Native Interaction Suite

| Semantic Command | 3-Letter Alias | Additional Aliases | Purpose |
| :--- | :--- | :--- | :--- |
| `wifi` | `wif` | - | Manage Wi-Fi connections, scan networks, toggle power, show Keychain passwords |
| `bluetooth` | `blt` | `bt`, `blue` | Inspect Bluetooth controller state & connected devices |
| `airpods` | `pod` | `pods` | Quick connect paired AirPods or Bluetooth headphones |
| `airdrop` | `drp` | `drop` | Open AirDrop finder window or share files via native Share Sheet |
| `music` | `msc` | `spotify` | Apple Music & Spotify player control (`play`, `pause`, `next`, `prev`) and track HUD |
| `volume` | `vol` | `sound` | Audio volume HUD & slider (0-100%, mute/unmute) |
| `note` | `not` | `notes` | Quick scratchpad saver directly into Apple Notes |
| `fixapp` | `fix` | `xattr` | Fix Gatekeeper quarantine ("App is damaged and can't be opened") |
| `battery` | `bat` | `batt` | Detailed battery health, cycle count, max capacity %, and charger wattage |
| `awake` | `caf` | `caffeinate` | Keep Mac awake and prevent display/system sleep with countdown timer |
| `peek` | `pek` | `ql`, `quicklook` | Launch native macOS QuickLook preview popup for any document or image |
| `trash` | `tsh` | - | Inspect Trash disk usage or safely empty trash |
| `shot` | `snt` | `snip`, `screenshot` | Capture screen selection or window directly to clipboard |
| `notify` | `ntf` | `alert` | Dispatch native macOS notification banner with sound |
| `dark` | `drk` | - | Toggle or set macOS Dark Mode |
| `light` | `lit` | - | Set macOS Light Mode |
| `lock` | `lok` | - | Lock macOS screen immediately |
| `desktop` | `dkt` | `desk` | Hide or show desktop icons for clean presentations |
| `dns` | `fls` | `flush` | Flush macOS DNS cache in 1 click (`dscacheutil` + `mDNSResponder`) |
| `voice` | `say` | `voc` | Native macOS Text-to-Speech synthesis with fun voices |
| `ocr` | `txt` | `vision`, `scan-text` | Extract text from screen selection or image file via Apple Neural Vision OCR |

### 🛠️ Developer & Workspace Suite

| Semantic Command | 3-Letter Alias | Additional Aliases | Purpose |
| :--- | :--- | :--- | :--- |
| `browse` | `brw` | `web`, `surf`, `google` | Search web or open URLs in default browser with dev bangs (`run web`, `run web gh nextjs`, `run web localhost:3000`) |
| `install` | `ins` | `deps`, `setup`, `i` | Auto-detect stack & lockfiles to install dependencies or add packages (`run ins`, `run i pkg -D`) |
| `color` | `hex` | `rgb`, `picker` | Color inspector, converter (HEX, RGB, HSL, Flutter), and macOS magnifying loupe eyedropper |
| `img` | `pic` | `view`, `photo` | Render images directly inside terminal with 24-bit TrueColor ANSI half-blocks |
| `encrypt` | `enc` | `crypt` | Encrypt files using military-grade authenticated AES-256-GCM and password |
| `decrypt` | `dec` | `uncrypt` | Decrypt files previously encrypted with `run encrypt` |
| `mock` | `fak` | `fake`, `dummy` | Instant developer mock & dummy data generator (users, products) in JSON/CSV |
| `project` | `prj` | - | Scan workspace hubs and launch in Antigravity, VS Code, Cursor, Xcode, or terminal |
| `dev` | `dev` | `develop` | Auto-detect stack (Rust, Node, Flutter, Go) and start development server |
| `build` | `bld` | - | Compile active project in release mode |
| `test` | `tst` | - | Polyglot automated test runner (`cargo test`, `npm test`, `pytest`, `flutter test`, `go test`) |
| `clean` | `cln` | - | Scan and clean disposable build artifacts (`target/`, `node_modules/`, `.next/`, `__pycache__/`, `DerivedData/`) with size confirmation |
| `sync` | `snc` | `git` | 1-step Git pull, status review, commit message prompt, and push |
| `docker` | `dck` | - | Inspect and manage Docker/OrbStack containers, view logs, start or stop |
| `secret` | `sec` | `dotenv` | Audit local `.env` variables against `.env.example` and generate sanitized templates (`--fix`) |
| `config` | `cfg` | - | Manage CLI settings, custom primary colors, editor & auto-clear (`run cfg`) |
| `alias` | `als` | `shortcut` | Define custom command shortcuts (`run alias add c "cargo check"`) with execution fallback |
| `stats` | `sts` | `analytics` | Terminal usage analytics dashboard with visual frequency bars (`run stats`) |
| `timer` | `tmr` | `pomo` | Focus Pomodoro / countdown timer with live progress bar and completion chime alert |
| `uuid` | `uid` | - | Generate UUID v4 and automatically copy to system clipboard |
| `pass` | `pas` | `password` | Generate cryptographically secure random password/API token and copy to clipboard |
| `update` | `upd` | `upgrade` | 1-step updater for Homebrew, Rust toolchain, and global Node packages |
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
| `qr` | `qrc` | - | Enhanced QR suite: terminal generator, PNG export, Wi-Fi share, Vision scan, and local drop |
| `lan` | `rad` | `radar`, `subnet` | Scan local Wi-Fi / LAN network devices, IP addresses, and MACs via ARP |
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

`run` is designed for fast, ergonomic terminal interactions:

1. **Bare `run` Fallback Menu:**
   - Running `run` without any arguments automatically displays an interactive selection menu listing all available commands and their 3-letter aliases with descriptions.
   - Selecting any command launches directly into its interactive mode.
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
   - If an unrecognized command is typed (e.g., `run fethc` or `run pak`), `run` calculates edit distances and prompts you with likely matches. If only one command is close, it corrects automatically.

---

## Detailed Usage Examples

### 1. Developer Workflows

```bash
# Scan all projects in ~/Developer, ~/Projects, etc. and choose IDE:
run project
# Or with 3-letter alias:
run prj

# Auto-detect stack, lockfile, or .venv and install all dependencies:
run install
run ins
run deps
run setup

# Add a specific package to the detected stack (auto-dispatches to pnpm/yarn/bun/npm, pip, cargo, etc.):
run ins express
run ins react -D           # Add as dev dependency
run ins tokio --dev        # Add dev dependency in Cargo.toml
run i pytest -D            # Python development requirement

# Search the web, developer hubs, or open URLs in your default browser:
run web rust iter map vs filter_map
run web localhost:3000     # Open local dev server or URLs directly
run web gh nextjs          # Search directly on GitHub
run web so borrow checker  # Search directly on StackOverflow
run web crate serde        # Search crates.io for Rust crates
run web npm tailwindcss    # Search npmjs.com for Node packages
run web mdn flexbox        # Search MDN Web Docs
run web ai "how to optimize rust cli"  # Ask AI (Perplexity)
run web                    # Auto-detects clipboard error or presents interactive menu

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

# Toggle compact mode (minimalist layout without large banners):
run cfg set compact_mode true
run cfg get compact_mode

# Print config file path or open in system editor:
run cfg path
run cfg edit

# Reset configuration to factory defaults:
run cfg reset
```

### 8. Port Inspector & Conflict Resolver

```bash
# List all active listening TCP ports interactively:
run port
run prt

# Inspect specific port:
run port 8000
run port 3000

# Terminate process blocking a port directly:
run port 8000 -k
run killport 8000
```

### 9. Custom Developer Aliases & Shortcuts

```bash
# Open interactive alias manager:
run alias
run als

# Add custom alias:
run alias add c "cargo check"
run alias add gs "git status"
run alias add devs "pnpm run dev"

# Execute registered custom alias:
run c
run gs

# List or remove aliases:
run alias list
run alias rm c
```

### 10. CLI Usage Analytics & Statistics

```bash
# View command execution stats and visual frequency bar chart:
run stats
run sts
```

### 11. 🍏 macOS Native Interaction Suite (Dual-Mode: Direct & Interactive)

Every command in the macOS Native Suite works in **Dual-Mode**: run directly with arguments for instant execution, or omit arguments to trigger an interactive fuzzy menu.

#### 1. Wi-Fi Manager (`run wifi`, `run wif`)
```bash
# Interactive Wi-Fi dashboard (status, scan, saved passwords, toggle):
run wifi
run wif

# Direct Wi-Fi status:
run wifi status

# Scan nearby Wi-Fi networks interactively:
run wifi scan

# Connect to Wi-Fi network directly:
run wifi connect "Office-5G" "SuperSecret123"

# Show saved Wi-Fi password from macOS Keychain:
run wifi pass "HomeNetwork"

# Toggle Wi-Fi power:
run wifi on
run wifi off
```

#### 2. Bluetooth & AirPods (`run bluetooth`, `run bt`, `run airpods`, `run pods`)
```bash
# Inspect Bluetooth status & connected devices:
run bt
run bt status

# One-click connect paired AirPods or Bluetooth headphones:
run pods
run airpods
```

#### 3. AirDrop & Share Sheet (`run airdrop`, `run drop`)
```bash
# Open native AirDrop window in Finder:
run drop
run airdrop

# Share specific document or screenshot via native macOS Share Sheet:
run drop report.pdf
run airdrop ~/Downloads/photo.png
```

#### 4. Music & Media HUD (`run music`, `run msc`)
```bash
# Interactive player dashboard with track info HUD and controls:
run music
run msc

# Direct playback controls (Apple Music & Spotify):
run music play
run music pause
run music next
run music prev
```

#### 5. Volume Controller (`run volume`, `run vol`)
```bash
# Interactive volume slider and presets:
run vol
run volume

# Direct volume adjustment (0-100%):
run vol 50
run vol 80
run vol mute
run vol unmute
```

#### 6. Apple Notes Scratchpad (`run note`, `run not`)
```bash
# Direct note quick-saver into Apple Notes:
run note "Meeting action items: review PR #42 and deploy to staging"

# Interactive multi-line note creator:
run note
```

#### 7. Gatekeeper & Quarantine Fixer (`run fixapp`, `run fix`)
```bash
# Fix Gatekeeper quarantine ("App is damaged and can't be opened"):
run fixapp Figma
run fix VSCode

# Interactive fuzzy search across /Applications and ~/Applications:
run fixapp
run fix
```

#### 8. Battery Health Inspector (`run battery`, `run bat`)
```bash
# View charge %, cycle count, max capacity %, health condition, and charger wattage:
run battery
run bat
```

#### 9. Screen & System Sleeplessness (`run awake`, `run caf`)
```bash
# Keep Mac awake with active timer (caffeinate wrapper):
run awake 30       # Keep awake for 30 minutes
run awake 120      # Keep awake for 2 hours

# Interactive duration picker:
run awake
run caf
```

#### 10. QuickLook Preview (`run peek`, `run pek`)
```bash
# Launch native macOS QuickLook preview popup:
run peek design_mockup.png
run peek README.md
run peek data.pdf

# Interactive file picker when omitted:
run peek
```

#### 11. Trash Manager (`run trash`, `run tsh`)
```bash
# Interactive Trash storage inspector and confirmation dialog:
run trash
run tsh

# List trash items or empty trash directly:
run trash list
run trash empty
```

#### 12. Screenshot & Snip (`run shot`, `run snt`, `run snip`)
```bash
# Interactive capture mode selector (selection, window, full screen):
run shot
run snip

# Direct capture to clipboard:
run shot selection
run shot window
run shot full
```

#### 13. System Notifications (`run notify`, `run ntf`)
```bash
# Dispatch native macOS notification banner with sound:
run notify "Deploy Complete" "Production v1.4.0 deployed successfully"

# Interactive prompt if omitted:
run notify
```

#### 14. Appearance, Lock & Desktop
```bash
# Toggle or set macOS Dark Mode:
run dark
run drk on
run drk off

# Set Light Mode:
run light
run lit

# Immediately lock macOS screen:
run lock
run lok

# Hide or show desktop icons for clean screenshares & presentations:
run desk hide
run desk show
run desk
```

#### 15. DNS Cache Flusher (`run dns`, `run flush`, `run fls`)
```bash
# Instant 1-click macOS DNS flush:
run dns
run flush
run fls
```

#### 16. Native Voice Text-to-Speech (`run voice`, `run say`, `run voc`)
```bash
# Direct speech synthesis:
run voice "Build finished successfully"
run say "Welcome back master"

# Custom voice personality (Samantha, Zarvox, Whisper, Fred, Bad News, etc.):
run voice "Alert! Deploy error" -v "Bad News"
run say "System online" --voice Zarvox

# Interactive voice picker & text prompt:
run voice
run say
```

### 12. ⚡ Productivity Essentials (Timer, UUID, Pass, QR, Update)

#### 1. Focus & Pomodoro Timer (`run timer`, `run pomo`, `run tmr`)
```bash
# Direct 25-minute Pomodoro timer with live progress bar & chime alert:
run timer 25
run pomo
run tmr 45

# Interactive duration presets (5m, 15m, 25m, 45m, 60m, or Custom):
run timer
```

#### 2. Instant UUID Generator (`run uuid`, `run uid`)
```bash
# Generate UUID v4 and copy to clipboard automatically:
run uuid
run uid
```

#### 3. Cryptographic Password & Token Generator (`run pass`, `run pas`)
```bash
# Generate secure password with automatic clipboard copy:
run pass 24
run pass 32
run pas

# Interactive presets (16-char strong, 24-char extra secure, 32-char token, alphanumeric):
run pass
```

#### 4. Enhanced QR Code Suite (`run qr`, `run qrc`)
```bash
# Generate visual Unicode QR code in terminal:
run qr https://github.com/naenmad/run-cli
run qr "Hello Antigravity!"

# Automatically uses URL from clipboard if omitted, or prompts interactively:
run qr
run qrc

# Export high-resolution QR code as PNG image file:
run qr https://github.com -o qrcode.png

# Copy QR code image directly to macOS clipboard (ready to paste in Figma, Slack, etc.):
run qr "https://github.com" -c

# Auto-detect active macOS Wi-Fi SSID and generate instant camera-scannable join QR:
run qr wifi
run qr wifi "Office-5G" "SecretPassword"

# Scan and decode QR code from screen (launches crosshair selector) or image file:
run qr scan
run qr scan /path/to/qr_image.png

# Ephemeral local Wi-Fi file sharing (starts micro-server & prints QR code for phones on the same Wi-Fi):
run qr share document.pdf
```

#### 5. One-Step System & Stack Updater (`run update`, `run upd`)
```bash
# Update Homebrew, Rust toolchain, and global Node packages in 1 step:
run update
run upd
```

#### 6. macOS Native Neural Vision OCR (`run ocr`, `run txt`, `run vision`)
```bash
# Drag and select any screen area with macOS crosshairs to extract text & copy to clipboard:
run ocr
run txt
run vision

# Extract text directly from an image file:
run ocr /path/to/screenshot.png
run ocr document.jpg
```

#### 7. Terminal TrueColor Image Viewer (`run img`, `run pic`, `run view`)
```bash
# Render any image (.png, .jpg, .webp, etc.) directly in terminal with 24-bit TrueColor ANSI half-blocks:
run img logo.png
run pic photo.jpg
run view banner.webp

# Limit rendering width to custom terminal column width:
run img logo.png -w 60
```

#### 8. Color Inspector, Converter & macOS Eyedropper (`run color`, `run hex`, `run picker`)
```bash
# Inspect hex color, show live TrueColor swatch, and convert to RGB, HSL, Flutter, Android, CSS:
run color #3B82F6
run hex "#ff007f"
run color "rgb(59, 130, 246)"
run color blue

# Launch native macOS magnifying glass eyedropper loupe to pick any pixel from screen:
run color --pick
run hex -p
run picker
```

#### 9. Authenticated File Encryption & Decryption (`run encrypt`, `run decrypt`, `run enc`, `run dec`)
```bash
# Encrypt sensitive files with military-grade AES-256-GCM authenticated encryption:
run encrypt .env
run enc secret_notes.txt

# Decrypt previously encrypted file:
run decrypt .env.enc
run dec secret_notes.txt.enc

# Pass password directly via flag:
run enc credentials.json -p "MyMasterPassword"
run dec credentials.json.enc -p "MyMasterPassword"
```

#### 10. Local Network & Wi-Fi Radar (`run lan`, `run rad`, `run radar`)
```bash
# Scan and discover all active devices on your local Wi-Fi/subnet (IP, MAC, Hostname, Role):
run lan
run rad
run radar
```

#### 11. Developer Mock & Dummy Data Generator (`run mock`, `run fak`, `run fake`)
```bash
# Generate 5 realistic mock user profiles in formatted JSON:
run mock
run mock user 5

# Generate 10 mock product records in CSV:
run mock product -n 10 --csv

# Copy generated mock data directly to macOS clipboard:
run mock user 20 -c
```

---

## ⚙️ Configuration File (`~/.config/run/config.toml`)

`run` automatically creates a configuration template at `~/.config/run/config.toml` upon first run.

```toml
# Primary theme accent color:
# Presets: "electric-blue" (default), "violet", "emerald", "amber", "rose", "cyan"
# Or use any custom HEX color code: "#ff007f", "#8b5cf6", "#10b981", "#00a2ff"
primary_color = "electric-blue"

# Automatically clear terminal screen before running commands and interactive menus:
auto_clear = false

# Compact mode (sleek minimalist layout without large ASCII banners):
compact_mode = false

# Default editor to open projects directly without prompting:
# Options: "antigravity", "cursor", "vscode", "xcode", "terminal", "ask" (default)
default_ide = "ask"

# Additional custom directory hubs to scan for projects in `run project`:
custom_hubs = [
    "~/Developer/Summit",
    "~/Work/Projects"
]
```
