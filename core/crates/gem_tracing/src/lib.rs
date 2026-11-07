use std::fmt::{self, Write as _};
use std::io::Write;
use std::sync::LazyLock;
use std::time::Duration;

pub mod path;

static ANSI: LazyLock<bool> = LazyLock::new(|| std::env::var("NO_COLOR").map_or(true, |value| value.is_empty()));

#[derive(Clone, Copy)]
enum Level {
    Info,
    Warn,
    Error,
}

impl Level {
    fn label(self) -> &'static str {
        match self {
            Self::Info => " INFO",
            Self::Warn => " WARN",
            Self::Error => "ERROR",
        }
    }

    fn color(self) -> &'static str {
        match self {
            Self::Info => "32",
            Self::Warn => "33",
            Self::Error => "31",
        }
    }
}

struct Escaped<'a>(&'a str);

impl fmt::Display for Escaped<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.chars().try_for_each(|character| match character {
            '\x1b' | '\x07' | '\x08' | '\x0c' | '\x7f' => write!(f, "\\x{:02x}", character as u32),
            '\u{80}'..='\u{9f}' => write!(f, "\\u{{{:x}}}", character as u32),
            character => f.write_char(character),
        })
    }
}

fn format_line(level: Level, message: &str, ansi: bool, timestamp: &str) -> String {
    if ansi {
        format!("\x1b[2m{timestamp}\x1b[0m \x1b[{}m{}\x1b[0m {}\n", level.color(), level.label(), Escaped(message))
    } else {
        format!("{timestamp} {} {}\n", level.label(), Escaped(message))
    }
}

fn write_line(level: Level, message: &str) {
    let timestamp = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.6fZ").to_string();
    let _ = std::io::stdout().lock().write_all(format_line(level, message, *ANSI, &timestamp).as_bytes());
}

pub fn human_duration(duration: Duration) -> String {
    if duration.is_zero() {
        return "0s".to_string();
    }

    let mut parts = Vec::new();
    let mut remaining = duration.as_secs();
    const UNITS: [(&str, u64); 4] = [("d", 86_400), ("h", 3_600), ("m", 60), ("s", 1)];

    for (label, unit) in UNITS {
        if remaining >= unit {
            let value = remaining / unit;
            remaining %= unit;
            parts.push(format!("{value}{label}"));
            if parts.len() == 2 {
                break;
            }
        }
    }

    if parts.is_empty() { format!("{}ms", duration.subsec_millis()) } else { parts.join(" ") }
}

pub struct DurationMs(pub Duration);

impl std::fmt::Display for DurationMs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&human_duration(self.0))
    }
}

fn format_fields(fields: &[(&str, &dyn std::fmt::Display)]) -> String {
    fields.iter().map(|(key, value)| format!("{key}={value}")).collect::<Vec<_>>().join(" ")
}

fn with_fields(message: &str, fields: &[(&str, &dyn std::fmt::Display)]) -> String {
    let pairs = format_fields(fields);
    if pairs.is_empty() { message.to_string() } else { format!("{message} {pairs}") }
}

pub fn info_with_fields_impl(message: &str, fields: &[(&str, &dyn std::fmt::Display)]) {
    write_line(Level::Info, &with_fields(message, fields));
}

pub fn error_fields_impl(message: &str, fields: &[(&str, &dyn std::fmt::Display)]) {
    write_line(Level::Error, &with_fields(message, fields));
}

pub fn warn_fields_impl(message: &str, fields: &[(&str, &dyn std::fmt::Display)]) {
    write_line(Level::Warn, &with_fields(message, fields));
}

pub fn error_with_fields_impl<E: std::error::Error + ?Sized>(message: &str, error: &E, fields: &[(&str, &dyn std::fmt::Display)]) {
    write_line(Level::Error, &format!("{} error={error}", with_fields(message, fields)));
}

#[macro_export]
macro_rules! info_with_fields {
    ($message:expr $(, $field:ident = $value:expr)* $(,)?) => {
        {
            let fields: &[(&str, &dyn std::fmt::Display)] = &[
                $((stringify!($field), &$value),)*
            ];
            $crate::info_with_fields_impl($message, fields);
        }
    };
}

#[macro_export]
macro_rules! warn_with_fields {
    ($message:expr $(, $field:ident = $value:expr)* $(,)?) => {
        {
            let fields: &[(&str, &dyn std::fmt::Display)] = &[
                $((stringify!($field), &$value),)*
            ];
            $crate::warn_fields_impl($message, fields);
        }
    };
}

#[macro_export]
macro_rules! error_fields {
    ($message:expr $(, $field:ident = $value:expr)* $(,)?) => {
        {
            let fields: &[(&str, &dyn std::fmt::Display)] = &[
                $((stringify!($field), &$value),)*
            ];
            $crate::error_fields_impl($message, fields);
        }
    };
}

#[macro_export]
macro_rules! error_with_fields {
    ($message:expr, $error:expr $(, $field:ident = $value:expr)* $(,)?) => {
        {
            let fields: &[(&str, &dyn std::fmt::Display)] = &[
                $((stringify!($field), &$value),)*
            ];
            $crate::error_with_fields_impl($message, $error, fields);
        }
    };
}

pub fn error<E: std::error::Error + ?Sized>(message: &str, error: &E) {
    error_with_fields_impl(message, error, &[]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_line() {
        let timestamp = "2026-11-06T21:28:15.934286Z";

        assert_eq!(
            format_line(Level::Info, "started chain=ethereum", true, timestamp),
            "\x1b[2m2026-11-06T21:28:15.934286Z\x1b[0m \x1b[32m INFO\x1b[0m started chain=ethereum\n"
        );
        assert_eq!(format_line(Level::Warn, "slow", true, timestamp), "\x1b[2m2026-11-06T21:28:15.934286Z\x1b[0m \x1b[33m WARN\x1b[0m slow\n");
        assert_eq!(format_line(Level::Error, "failed", true, timestamp), "\x1b[2m2026-11-06T21:28:15.934286Z\x1b[0m \x1b[31mERROR\x1b[0m failed\n");
        assert_eq!(format_line(Level::Info, "plain", false, timestamp), "2026-11-06T21:28:15.934286Z  INFO plain\n");
        assert_eq!(format_line(Level::Error, "failed", false, timestamp), "2026-11-06T21:28:15.934286Z ERROR failed\n");
    }

    #[test]
    fn test_format_line_escapes_terminal_controls() {
        let timestamp = "2026-11-06T21:28:15.934286Z";

        assert_eq!(
            format_line(Level::Error, "id=x\x1b[2Jy \x07\x08\x0c\x7f \u{80}\u{85}\u{9f}\u{a0} tab\tline\nü", false, timestamp),
            "2026-11-06T21:28:15.934286Z ERROR id=x\\x1b[2Jy \\x07\\x08\\x0c\\x7f \\u{80}\\u{85}\\u{9f}\u{a0} tab\tline\nü\n"
        );
    }

    #[test]
    fn test_with_fields() {
        assert_eq!(with_fields("synced", &[]), "synced");
        assert_eq!(with_fields("synced", &[("chain", &"ethereum"), ("count", &3)]), "synced chain=ethereum count=3");
    }
}
