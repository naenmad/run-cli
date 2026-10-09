# Contributing to `run-cli`

Thank you for your interest in contributing to `run-cli`! This project is an open source, community-driven productivity CLI for macOS built with Rust.

Please review this guide before submitting issues or pull requests.

---

## Code of Conduct

Everyone participating in this project is expected to follow our [Code of Conduct](CODE_OF_CONDUCT.md). Please report any unacceptable behavior to the project maintainer.

---

## Getting Started

### Prerequisites

* **Operating System**: macOS (Apple Silicon `aarch64` or Intel `x86_64`)
* **Rust**: Toolchain version 1.85 or newer (Rust 2024 edition support)
* **Git**: Installed and configured

Check your Rust version:

```bash
rustc --version
cargo --version
```

If you need to update Rust:

```bash
rustup update stable
```

### Fork & Clone

1. Fork the repository on GitHub: [https://github.com/naenmad/run-cli](https://github.com/naenmad/run-cli)
2. Clone your fork locally:

```bash
git clone https://github.com/<your-username>/run-cli.git
cd run-cli
```

3. Add the upstream remote:

```bash
git remote add upstream https://github.com/naenmad/run-cli.git
```

---

## Local Development Workflow

### Building and Running

Compile the project in debug mode:

```bash
cargo build
```

Run commands directly via `cargo run`:

```bash
# Run interactive menu
cargo run

# Run specific command
cargo run -- port 8080
cargo run -- wifi
cargo run -- help
```

### Running Tests

We maintain comprehensive unit and integration tests covering CLI parsing, Levenshtein distance resolution, theme parsing, and Clap-metadata symmetry:

```bash
cargo test
```

To run a specific test by name:

```bash
cargo test test_dynamic_all_commands_clap_and_metadata_symmetry
```

You can also run the full end-to-end command runner script:

```bash
./scripts/test_all_commands.sh
```

### Code Formatting and Linting

Before opening a pull request, your code must pass formatting checks and Clippy warnings. CI will reject PRs that fail these checks.

1. **Check formatting**:
   ```bash
   cargo fmt --all -- --check
   ```
   To automatically format files:
   ```bash
   cargo fmt --all
   ```

2. **Run Clippy**:
   ```bash
   cargo clippy --all-targets -- -D warnings
   ```

---

## Design Principles & Command Rules

Every command in `run-cli` adheres to the standards defined in [DESIGN.md](DESIGN.md):

1. **Human-Friendly Verb**: Use clear, standard English verbs or nouns (`make`, `open`, `copy`, `port`).
2. **Mandatory 3-Letter Alias**: Every command must provide a unique 3-letter alias (`mak`, `opn`, `cpy`, `prt`).
3. **Dual-Mode Execution**:
   - **Direct Mode**: When arguments are supplied (e.g., `run port 8080`), execute immediately.
   - **Interactive Mode**: When arguments are omitted (e.g., `run port`), provide an interactive terminal prompt (using `dialoguer`).
4. **Safety Defaults**: Destructive commands must require user confirmation before taking action.
5. **Clear Error Output**: Return user-friendly error messages without panics or raw stack traces.

### Adding a New Command Checklist

When introducing a new sub-command:

1. **Clap Definition**: Add the variant with its docs, arguments, and aliases to `enum Commands` in `src/main.rs`.
2. **Metadata Registration**: Add the corresponding `CommandMeta` struct to the `ALL_COMMANDS` array in `src/main.rs`. Ensure `name`, `alias_3`, `aliases`, `summary`, and `category` are populated.
3. **Command Implementation**: Place execution logic in `src/commands.rs` or `src/mac.rs`.
4. **Interactive Routing**: Wire up interactive and direct execution in `execute_command` in `src/main.rs`.
5. **Help Guide**: Add a descriptive help section to `run help <command>` in `src/main.rs`.
6. **Documentation**: Document the command and its aliases in both `README.md` and `COMMANDS.md`.
7. **Symmetry Test**: Run `cargo test` to verify that `test_dynamic_all_commands_clap_and_metadata_symmetry` passes.

---

## Git Workflow & Pull Requests

### Branch Naming

Create a feature branch off `main`:

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/issue-description
# or
git checkout -b docs/update-readme
```

### Commit Messages

Use clear, imperative commit messages:

* `feat: add lan device ping latency inspector`
* `fix: prevent crash when scanning empty keychain entries`
* `docs: update CONTRIBUTING.md with command symmetry guide`
* `refactor: extract network helpers into dedicated module`

### Submitting a Pull Request

1. Push your branch to your fork:
   ```bash
   git push origin feature/your-feature-name
   ```
2. Open a Pull Request against the `main` branch of `naenmad/run-cli`.
3. Fill out the PR template describing the changes and referencing any related issues.
4. Verify that all automated GitHub Actions checks pass.

---

## Questions and Support

If you have questions or need guidance, feel free to open a discussion or file an issue on GitHub. Thank you for helping build a better terminal experience for macOS!
