const ASK_NOTIFICATIONS_COOLDOWN_SECONDS: i64 = 30 * 24 * 60 * 60;

pub fn should_ask_notifications(last_asked_at: i64, now: i64) -> bool {
    now - last_asked_at >= ASK_NOTIFICATIONS_COOLDOWN_SECONDS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asking_again_waits_a_month() {
        let now = 1_700_000_000;

        assert!(should_ask_notifications(0, now));
        assert!(should_ask_notifications(now - ASK_NOTIFICATIONS_COOLDOWN_SECONDS, now));
        assert!(!should_ask_notifications(now - ASK_NOTIFICATIONS_COOLDOWN_SECONDS + 1, now));
        assert!(!should_ask_notifications(now, now));
    }
}
