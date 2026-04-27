//! Testing business logic without a mocking library.
//!
//! Article section: "Cheap Testing — and Better Tests"
//!
//! Run: cargo test

use singleton_registry::define_registry;
use std::sync::{Arc, Mutex};

// Isolated registry for this test module — no shared state with the rest of the crate.
// Multiple test modules can each define their own registry and stay independent.
// If you add more tests to this module that share test_app, run them with
// serial_test::serial to avoid registry-state conflicts between parallel tests.
define_registry!(test_app);

trait Formatter: Send + Sync {
    fn format(&self, title: &str, body: &str) -> String;
}

trait Sink: Send + Sync {
    fn write(&self, content: &str);
}

struct PlainFormatter;

impl Formatter for PlainFormatter {
    fn format(&self, title: &str, body: &str) -> String {
        format!("{}: {}", title, body)
    }
}

struct CapturingSink(Mutex<Vec<String>>);

impl CapturingSink {
    fn new() -> Self {
        CapturingSink(Mutex::new(vec![]))
    }

    fn captured(&self) -> Vec<String> {
        self.0.lock().unwrap().clone()
    }
}

impl Sink for CapturingSink {
    fn write(&self, content: &str) {
        self.0.lock().unwrap().push(content.to_string());
    }
}

// Uses test_app so the test controls every capability without touching production registries.
fn generate_report(title: &str, body: &str) {
    let fmt = match test_app::get_cloned::<Arc<dyn Formatter>>() {
        Ok(f) => f,
        Err(_) => {
            eprintln!("warn: no Formatter registered, skipping");
            return;
        }
    };
    match test_app::get_cloned::<Arc<dyn Sink>>() {
        Ok(sink) => sink.write(&fmt.format(title, body)),
        Err(_) => eprintln!("warn: no Sink registered, skipping"),
    }
}

#[test]
fn report_includes_title() {
    // define_registry!(test_app) gives an isolated registry per test module.
    let sink = Arc::new(CapturingSink::new());
    test_app::register(sink.clone() as Arc<dyn Sink>);
    test_app::register(Arc::new(PlainFormatter) as Arc<dyn Formatter>);

    generate_report("Q1", "Revenue up 12%.");

    assert!(sink.captured()[0].contains("Q1"));
}
