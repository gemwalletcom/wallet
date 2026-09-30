use std::time::Duration;

pub async fn sleep(duration: Duration) {
    let (sender, receiver) = futures::channel::oneshot::channel();
    std::thread::spawn(move || {
        std::thread::sleep(duration);
        let _ = sender.send(());
    });
    let _ = receiver.await;
}

pub fn is_outdated(updated_at: Option<i64>, now: i64, interval_seconds: u32) -> bool {
    updated_at.is_none_or(|updated_at| now - updated_at >= i64::from(interval_seconds))
}

pub fn parse_timestamp(value: Option<String>) -> Option<i64> {
    value.and_then(|value| value.trim().parse().ok())
}

pub fn parse_timestamp_or_zero(value: Option<String>) -> u64 {
    parse_timestamp(value).and_then(|timestamp| u64::try_from(timestamp).ok()).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_outdated_when_never_updated_or_past_the_interval() {
        assert!(is_outdated(None, 1_000_000, 3_600), "an asset that was never fully updated cannot tell an empty association list from an unknown one");
        assert!(!is_outdated(Some(1_000_000 - 3_599), 1_000_000, 3_600));
        assert!(is_outdated(Some(1_000_000 - 3_600), 1_000_000, 3_600));
    }

    #[test]
    fn test_parse_timestamp() {
        assert_eq!(parse_timestamp(Some("42".to_string())), Some(42));
        assert_eq!(parse_timestamp(Some(" 7 ".to_string())), Some(7));
        assert_eq!(parse_timestamp(Some("abc".to_string())), None);
        assert_eq!(parse_timestamp(None), None);
        assert_eq!(parse_timestamp_or_zero(Some("42".to_string())), 42);
        assert_eq!(parse_timestamp_or_zero(Some("-1".to_string())), 0);
        assert_eq!(parse_timestamp_or_zero(None), 0);
    }
}
