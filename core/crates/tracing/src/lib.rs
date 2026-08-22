use std::sync::{Arc, OnceLock};
use std::time::Duration;
use tracing_subscriber::FmtSubscriber;

pub use tracing;

static TRACING_SUBSCRIBER: OnceLock<Arc<FmtSubscriber>> = OnceLock::new();

pub fn get_subscriber() -> Arc<FmtSubscriber> {
    TRACING_SUBSCRIBER.get_or_init(|| Arc::new(tracing_subscriber::fmt().with_target(false).finish())).clone()
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

pub fn normalize_path(path: &str) -> String {
    path.split_once('?')
        .map_or(path, |(path, _)| path)
        .split('/')
        .map(|segment| {
            if !segment.is_empty() && segment.chars().all(|character| character.is_ascii_digit()) {
                ":number"
            } else if segment.len() > 20 {
                ":value"
            } else {
                segment
            }
        })
        .collect::<Vec<_>>()
        .join("/")
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

pub fn info_with_fields_impl(message: &str, fields: &[(&str, &dyn std::fmt::Display)]) {
    let subscriber = get_subscriber();
    tracing::subscriber::with_default(subscriber, || {
        let pairs = format_fields(fields);
        if pairs.is_empty() {
            tracing::info!("{}", message);
        } else {
            tracing::info!("{} {}", message, pairs);
        }
    });
}

pub fn error_fields_impl(message: &str, fields: &[(&str, &dyn std::fmt::Display)]) {
    let subscriber = get_subscriber();
    tracing::subscriber::with_default(subscriber, || {
        let pairs = format_fields(fields);
        if pairs.is_empty() {
            tracing::error!("{}", message);
        } else {
            tracing::error!("{} {}", message, pairs);
        }
    });
}

pub fn warn_fields_impl(message: &str, fields: &[(&str, &dyn std::fmt::Display)]) {
    let subscriber = get_subscriber();
    tracing::subscriber::with_default(subscriber, || {
        let pairs = format_fields(fields);
        if pairs.is_empty() {
            tracing::warn!("{}", message);
        } else {
            tracing::warn!("{} {}", message, pairs);
        }
    });
}

pub fn error_with_fields_impl<E: std::error::Error + ?Sized>(message: &str, error: &E, fields: &[(&str, &dyn std::fmt::Display)]) {
    let subscriber = get_subscriber();
    tracing::subscriber::with_default(subscriber, || {
        let pairs = format_fields(fields);
        if pairs.is_empty() {
            tracing::error!("{} error={}", message, error);
        } else {
            tracing::error!("{} {} error={}", message, pairs, error);
        }
    });
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
    fn normalizes_dynamic_path_segments() {
        let cases = [
            ("/api/v1/verylongsegmentthatisgreaterthan20characters/data", "/api/v1/:value/data"),
            ("/block/12345/transactions", "/block/:number/transactions"),
            ("/block/12345/tx/67890", "/block/:number/tx/:number"),
            ("/api/v1/data", "/api/v1/data"),
            ("/api//data", "/api//data"),
            ("/api/v2/block/5897744?page=1", "/api/v2/block/:number"),
            ("/thorchain/quote/swap?from=X&to=Y", "/thorchain/quote/swap"),
        ];

        for (input, expected) in cases {
            assert_eq!(normalize_path(input), expected, "failed for {input}");
        }
    }
}
