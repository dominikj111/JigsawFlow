//! Swapping implementations at runtime.
//!
//! Article section: "Runtime Swappability"
//!
//! Run: cargo run --example 02_runtime_swap

use singleton_registry::define_registry;
use std::sync::Arc;

trait Formatter: Send + Sync {
    fn format(&self, title: &str, body: &str) -> String;
}

define_registry!(app);

fn generate_report(title: &str, body: &str) {
    match app::get_cloned::<Arc<dyn Formatter>>() {
        Ok(fmt) => println!("{}", fmt.format(title, body)),
        Err(_) => eprintln!("warn: no Formatter registered, skipping"),
    }
}

struct PlainFormatter;

impl Formatter for PlainFormatter {
    fn format(&self, title: &str, body: &str) -> String {
        format!("{}: {}", title, body)
    }
}

struct MarkdownFormatter;

impl Formatter for MarkdownFormatter {
    fn format(&self, title: &str, body: &str) -> String {
        format!("# {}\n\n{}", title, body)
    }
}

fn main() {
    // First call: plain text.
    app::register(Arc::new(PlainFormatter) as Arc<dyn Formatter>);
    generate_report("Q1", "Revenue up 12%.");

    // Swap to Markdown — generate_report never changes.
    // Any caller that already holds an Arc<dyn Formatter> keeps the old one;
    // new lookups get the replacement. No race condition.
    app::register(Arc::new(MarkdownFormatter) as Arc<dyn Formatter>);
    generate_report("Q2", "Projections look strong.");
}
