//! Capability contracts and graceful degradation.
//!
//! Article section: "The Idea: Capabilities Instead of Imports"
//!
//! Run: cargo run --example 01_capability_contract

use singleton_registry::define_registry;
use std::sync::Arc;

trait Formatter: Send + Sync {
    fn format(&self, title: &str, body: &str) -> String;
}

define_registry!(app);

// Business logic — knows nothing about which Formatter is registered.
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

fn main() {
    // No Formatter registered yet — degrades gracefully.
    generate_report("Q0", "This will be skipped.");

    // Composition lives in main — the only place that picks concrete types.
    app::register(Arc::new(PlainFormatter) as Arc<dyn Formatter>);
    generate_report("Q1", "Revenue up 12%.");
}
