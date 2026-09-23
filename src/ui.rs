use std::fmt;
use std::io::{self, Write};
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use colored::{ColoredString, Colorize};
use dialoguer::theme::{ColorfulTheme, Theme};

/// Returns text colored in the active configured primary color (defaults to electric blue #00a2ff)
pub fn primary_colored(text: &str) -> ColoredString {
    crate::config::primary_colored(text)
}

/// Backward compatibility alias for primary_colored
pub fn electric_blue(text: &str) -> ColoredString {
    primary_colored(text)
}

/// Returns a standardized high-contrast red Cancel option for interactive menus
pub fn cancel_option() -> String {
    format!("{}", "Cancel".bright_red().bold())
}

/// Clears terminal screen if user configured auto_clear = true
pub fn maybe_auto_clear() {
    let cfg = crate::config::load_config();
    if cfg.auto_clear.unwrap_or(false) {
        let _ = std::process::Command::new("clear").status();
    }
}

/// Custom theme wrapping dialoguer::ColorfulTheme with active primary color
/// and guaranteed persistent Bold Bright Red styling for "Cancel" items.
pub struct RunTheme {
    pub inner: ColorfulTheme,
}

impl Clone for RunTheme {
    fn clone(&self) -> Self {
        custom_theme()
    }
}

impl Theme for RunTheme {
    fn format_prompt(&self, f: &mut dyn fmt::Write, prompt: &str) -> fmt::Result {
        self.inner.format_prompt(f, prompt)
    }

    fn format_error(&self, f: &mut dyn fmt::Write, err: &str) -> fmt::Result {
        self.inner.format_error(f, err)
    }

    fn format_confirm_prompt(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        default: Option<bool>,
    ) -> fmt::Result {
        self.inner.format_confirm_prompt(f, prompt, default)
    }

    fn format_confirm_prompt_selection(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        selection: Option<bool>,
    ) -> fmt::Result {
        self.inner
            .format_confirm_prompt_selection(f, prompt, selection)
    }

    fn format_input_prompt(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        default: Option<&str>,
    ) -> fmt::Result {
        self.inner.format_input_prompt(f, prompt, default)
    }

    fn format_input_prompt_selection(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        sel: &str,
    ) -> fmt::Result {
        self.inner.format_input_prompt_selection(f, prompt, sel)
    }

    fn format_password_prompt(&self, f: &mut dyn fmt::Write, prompt: &str) -> fmt::Result {
        self.inner.format_password_prompt(f, prompt)
    }

    fn format_password_prompt_selection(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
    ) -> fmt::Result {
        self.inner.format_password_prompt_selection(f, prompt)
    }

    fn format_select_prompt(&self, f: &mut dyn fmt::Write, prompt: &str) -> fmt::Result {
        self.inner.format_select_prompt(f, prompt)
    }

    fn format_select_prompt_selection(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        sel: &str,
    ) -> fmt::Result {
        self.inner.format_select_prompt_selection(f, prompt, sel)
    }

    fn format_select_prompt_item(
        &self,
        f: &mut dyn fmt::Write,
        text: &str,
        active: bool,
    ) -> fmt::Result {
        let is_cancel = text.contains("Cancel");
        if active {
            if is_cancel {
                write!(
                    f,
                    "{} {}",
                    "❯".bright_red().bold(),
                    "Cancel".bright_red().bold()
                )
            } else {
                self.inner.format_select_prompt_item(f, text, active)
            }
        } else if is_cancel {
            write!(f, "  {}", "Cancel".bright_red().bold())
        } else {
            self.inner.format_select_prompt_item(f, text, active)
        }
    }

    fn format_fuzzy_select_prompt(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        search_term: &str,
        cursor_pos: usize,
    ) -> fmt::Result {
        self.inner
            .format_fuzzy_select_prompt(f, prompt, search_term, cursor_pos)
    }

    fn format_fuzzy_select_prompt_item(
        &self,
        f: &mut dyn fmt::Write,
        text: &str,
        active: bool,
        highlight_matches: bool,
        matcher: &fuzzy_matcher::skim::SkimMatcherV2,
        search_term: &str,
    ) -> fmt::Result {
        let is_cancel = text.contains("Cancel");
        if active {
            if is_cancel {
                write!(
                    f,
                    "{} {}",
                    "❯".bright_red().bold(),
                    "Cancel".bright_red().bold()
                )
            } else {
                self.inner.format_fuzzy_select_prompt_item(
                    f,
                    text,
                    active,
                    highlight_matches,
                    matcher,
                    search_term,
                )
            }
        } else if is_cancel {
            write!(f, "  {}", "Cancel".bright_red().bold())
        } else {
            self.inner.format_fuzzy_select_prompt_item(
                f,
                text,
                active,
                highlight_matches,
                matcher,
                search_term,
            )
        }
    }
}

/// Creates a customized dialoguer theme respecting user's configured primary color
/// and guaranteeing persistent bold red styling for Cancel options.
pub fn custom_theme() -> RunTheme {
    let mut inner = ColorfulTheme::default();
    let col256 = crate::config::get_primary_color256();

    let primary_style = dialoguer::console::Style::new()
        .for_stderr()
        .color256(col256)
        .bold();
    let primary_symbol = dialoguer::console::style("❯".to_string())
        .for_stderr()
        .color256(col256)
        .bold();

    inner.prompt_style = dialoguer::console::Style::new().for_stderr().bold();
    inner.prompt_prefix = dialoguer::console::style("?".to_string())
        .for_stderr()
        .color256(col256)
        .bold();
    inner.values_style = primary_style.clone();
    inner.active_item_style = primary_style;
    inner.active_item_prefix = primary_symbol.clone();
    inner.picked_item_prefix = primary_symbol;

    RunTheme { inner }
}

/// Calculate visible length of a string ignoring ANSI terminal escape codes
pub fn visible_len(s: &str) -> usize {
    let mut len = 0;
    let mut in_escape = false;
    for c in s.chars() {
        if c == '\x1B' {
            in_escape = true;
        } else if in_escape {
            if c == 'm' {
                in_escape = false;
            }
        } else {
            len += 1;
        }
    }
    len
}

/// Print modern minimalist header banner for run CLI
pub fn print_banner() {
    println!();
    let top = "╭────────────────────────────────────────────────────────────╮";
    let title = format!(
        "│  {} {}  v0.3.0 • Developer Productivity Engine       │",
        "⚡".bold(),
        primary_colored("run").bold()
    );
    let bottom = "╰────────────────────────────────────────────────────────────╯";
    println!("{}", primary_colored(top));
    println!("{}", title);
    println!("{}", primary_colored(bottom));
    println!();
}

/// Render navigation breadcrumbs
pub fn render_breadcrumbs(crumbs: &[&str]) {
    let formatted: Vec<String> = crumbs
        .iter()
        .enumerate()
        .map(|(i, c)| {
            if i == crumbs.len() - 1 {
                format!("{}", c.bold().white())
            } else {
                format!("{}", primary_colored(c).dimmed())
            }
        })
        .collect();
    let arrow = format!(" {} ", primary_colored("›").bold());
    println!(
        " {} {}",
        primary_colored("›").bold(),
        formatted.join(&arrow)
    );
    println!();
}

/// Print keyboard navigation hints for interactive selection
pub fn print_key_hints() {
    println!(
        "{}",
        "  [ ↑/↓ Navigate • Type to Filter • Enter Select • Esc/Cancel Abort ]".dimmed()
    );
}

/// Print a stylish rounded card panel with key-value information
pub fn print_card(title: &str, rows: &[(&str, String)]) {
    let max_label_len = rows
        .iter()
        .map(|(k, _)| visible_len(k))
        .max()
        .unwrap_or(12)
        .max(12);

    let max_val_len = rows.iter().map(|(_, v)| visible_len(v)).max().unwrap_or(20);

    let content_width = (max_label_len + max_val_len + 6)
        .max(visible_len(title) + 6)
        .max(52);

    let title_prefix = format!("╭─ {} ", title);
    let title_prefix_len = visible_len(&title_prefix);
    let top_pad = content_width.saturating_sub(title_prefix_len);
    let top_bar = format!("{}{}{}", title_prefix, "─".repeat(top_pad), "╮");
    println!("{}", primary_colored(&top_bar));

    for (label, val) in rows {
        let label_vis = visible_len(label);
        let pad_label = max_label_len.saturating_sub(label_vis);
        let left = format!("  {}{}: ", label.bold(), " ".repeat(pad_label));
        let left_vis = visible_len(&left);
        let val_vis = visible_len(val);
        let right_pad = content_width.saturating_sub(left_vis + val_vis + 1);

        println!(
            "{} {}{}{}{}",
            primary_colored("│"),
            left,
            val,
            " ".repeat(right_pad),
            primary_colored("│")
        );
    }

    let bottom_bar = format!("╰{}╯", "─".repeat(content_width));
    println!("{}", primary_colored(&bottom_bar));
    println!();
}

/// Detect project tech stack and return a colorized pill badge
pub fn detect_stack_badge(path: &Path) -> String {
    if path.join("Cargo.toml").exists() {
        format!("{}", " 🦀 [Rust] ".bold().bright_yellow().on_black())
    } else if path.join("package.json").exists() {
        format!("{}", " ⚡ [Node] ".bold().bright_green().on_black())
    } else if path.join("pubspec.yaml").exists() {
        format!("{}", " 🎯 [Flutter] ".bold().bright_blue().on_black())
    } else if path.join("go.mod").exists() {
        format!("{}", " 🐹 [Go] ".bold().bright_cyan().on_black())
    } else if path.join("pyproject.toml").exists()
        || path.join("requirements.txt").exists()
        || path.join("Pipfile").exists()
    {
        format!("{}", " 🐍 [Python] ".bold().bright_yellow().on_black())
    } else if path.join("Makefile").exists() {
        format!("{}", " 🛠️  [Make] ".bold().white().on_black())
    } else {
        format!("{}", " 📁 [Project] ".dimmed())
    }
}

/// Non-blocking terminal micro-spinner running on a background thread
pub struct Spinner {
    running: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}

impl Spinner {
    pub fn start(message: &'static str) -> Self {
        let running = Arc::new(AtomicBool::new(true));
        let r_clone = running.clone();

        let handle = thread::spawn(move || {
            let frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
            let mut i = 0;
            let mut stderr = io::stderr();
            while r_clone.load(Ordering::Relaxed) {
                let frame = frames[i % frames.len()];
                let _ = write!(stderr, "\r{} {} ", primary_colored(frame).bold(), message);
                let _ = stderr.flush();
                thread::sleep(Duration::from_millis(80));
                i += 1;
            }
            let _ = write!(stderr, "\r\x1B[2K");
            let _ = stderr.flush();
        });

        Self {
            running,
            handle: Some(handle),
        }
    }

    pub fn stop(&mut self) {
        if let Some(handle) = self.handle.take() {
            self.running.store(false, Ordering::Relaxed);
            let _ = handle.join();
        }
    }
}

impl Drop for Spinner {
    fn drop(&mut self) {
        self.stop();
    }
}
