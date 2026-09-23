use colored::{ColoredString, Colorize};
use std::io::{self, Write};
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

/// Returns text colored in signature electric blue (#00a2ff)
pub fn electric_blue(text: &str) -> ColoredString {
    text.truecolor(0, 162, 255)
}

/// Returns a standardized high-contrast red Cancel option for interactive menus
pub fn cancel_option() -> String {
    format!("{}", "Cancel".bright_red().bold())
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
        "│  {} {}  v0.2.0 • Developer Productivity Engine       │",
        "⚡".bold(),
        electric_blue("run").bold()
    );
    let bottom = "╰────────────────────────────────────────────────────────────╯";
    println!("{}", electric_blue(top));
    println!("{}", title);
    println!("{}", electric_blue(bottom));
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
                format!("{}", electric_blue(c).dimmed())
            }
        })
        .collect();
    let arrow = format!(" {} ", electric_blue("›").bold());
    println!(" {} {}", electric_blue("›").bold(), formatted.join(&arrow));
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
    println!("{}", electric_blue(&top_bar));

    for (label, val) in rows {
        let label_vis = visible_len(label);
        let pad_label = max_label_len.saturating_sub(label_vis);
        let left = format!("  {}{}: ", label.bold(), " ".repeat(pad_label));
        let left_vis = visible_len(&left);
        let val_vis = visible_len(val);
        let right_pad = content_width.saturating_sub(left_vis + val_vis + 1);

        println!(
            "{} {}{}{}{}",
            electric_blue("│"),
            left,
            val,
            " ".repeat(right_pad),
            electric_blue("│")
        );
    }

    let bottom_bar = format!("╰{}╯", "─".repeat(content_width));
    println!("{}", electric_blue(&bottom_bar));
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
                let _ = write!(stderr, "\r{} {} ", electric_blue(frame).bold(), message);
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
