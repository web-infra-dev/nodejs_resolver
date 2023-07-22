use tracing_subscriber::prelude::*;

use crate::context::Depth;

pub fn enable_by_env() {
    let is_enabled = std::env::var("RESOLVER_TRACE")
        .map_or(false, |var| matches!(var.as_str(), "TRACE" | "DEBUG" | "INFO" | "WARN" | "ERROR"));
    if !is_enabled {
        return;
    }
    let formatter = Formatter::default();
    tracing_subscriber::Registry::default()
        .with(formatter)
        .with(tracing_subscriber::EnvFilter::from_env("RESOLVER_TRACE"))
        .init();
}

#[derive(Default)]
struct Formatter {}

impl<S> tracing_subscriber::Layer<S> for Formatter
where
    S: tracing::Subscriber + std::fmt::Debug,
{
    fn on_event(&self, event: &tracing::Event<'_>, _: tracing_subscriber::layer::Context<'_, S>) {
        event.record(&mut Data);
    }
}

struct Data;

impl tracing::field::Visit for Data {
    fn record_debug(&mut self, _field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        eprintln!("{value:?}");
    }
}

/// TODO: use marco
pub mod color {
    const BOLD: &str = "\u{001b}[1m";
    const RED: &str = "\u{001b}[31m";
    const GREEN: &str = "\u{001b}[32m";
    const BLUE: &str = "\u{001b}[34m";
    const CYAN: &str = "\u{001b}[36m";
    const RESET: &str = "\u{001b}[0m";

    pub fn bold<T: core::fmt::Display>(s: &T) -> String {
        format!("{BOLD}{s}{RESET}")
    }

    pub fn red<T: core::fmt::Display>(s: &T) -> String {
        format!("{RED}{s}{RESET}")
    }

    pub fn green<T: core::fmt::Display>(s: &T) -> String {
        format!("{GREEN}{s}{RESET}")
    }

    pub fn blue<T: core::fmt::Display>(s: &T) -> String {
        format!("{BLUE}{s}{RESET}")
    }

    pub fn cyan<T: core::fmt::Display>(s: &T) -> String {
        format!("{CYAN}{s}{RESET}")
    }
}

pub fn depth(depth: &Depth) -> String {
    format!("Depth: {}", color::bold(&depth.value()))
}
