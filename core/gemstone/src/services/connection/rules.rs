use crate::models::GemConnectionComponent;

const RECONNECT_MULTIPLIER_MILLISECONDS: f64 = 300.0;
const RECONNECT_MAXIMUM_MILLISECONDS: f64 = 30_000.0;
const OFFLINE_DEBOUNCE_MILLISECONDS: u64 = 500;
const PING_INTERVAL_MILLISECONDS: u64 = 30_000;

pub fn reconnect_delay_milliseconds(attempt: u32) -> u64 {
    (RECONNECT_MULTIPLIER_MILLISECONDS * f64::from(attempt).exp()).min(RECONNECT_MAXIMUM_MILLISECONDS) as u64
}

pub fn offline_debounce_milliseconds() -> u64 {
    OFFLINE_DEBOUNCE_MILLISECONDS
}

pub fn ping_interval_milliseconds() -> u64 {
    PING_INTERVAL_MILLISECONDS
}

pub fn resets_component_health(component: GemConnectionComponent, is_healthy: bool, was_healthy: Option<bool>) -> bool {
    component == GemConnectionComponent::Internet && is_healthy && was_healthy == Some(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keepalive_pings_before_the_reconnect_backoff_caps_out() {
        assert_eq!(ping_interval_milliseconds(), 30_000);
        assert!(
            ping_interval_milliseconds() >= reconnect_delay_milliseconds(0),
            "a keepalive that fires faster than the first reconnect would ping a socket that is still coming up"
        );
    }

    #[test]
    fn test_reconnect_delay_grows_exponentially_and_caps() {
        assert_eq!(reconnect_delay_milliseconds(0), 300);
        assert_eq!(reconnect_delay_milliseconds(1), 815);
        assert_eq!(reconnect_delay_milliseconds(3), 6_025);
        assert_eq!(reconnect_delay_milliseconds(4), 16_379);
        assert_eq!(
            reconnect_delay_milliseconds(5),
            RECONNECT_MAXIMUM_MILLISECONDS as u64,
            "the curve is capped from the attempt it first exceeds the maximum"
        );
        assert_eq!(
            reconnect_delay_milliseconds(u32::MAX),
            RECONNECT_MAXIMUM_MILLISECONDS as u64,
            "an overflowing exponent still yields the cap"
        );
    }

    #[test]
    fn test_reconnect_delay_never_decreases() {
        let delays: Vec<u64> = (0..10).map(reconnect_delay_milliseconds).collect();
        assert!(delays.windows(2).all(|pair| pair[0] <= pair[1]));
    }

    #[test]
    fn test_offline_debounce_holds_a_drop_before_reporting_it() {
        assert_eq!(offline_debounce_milliseconds(), 500);
    }

    #[test]
    fn test_only_recovering_internet_resets_component_health() {
        assert!(resets_component_health(GemConnectionComponent::Internet, true, Some(false)));
        assert!(
            !resets_component_health(GemConnectionComponent::Internet, true, Some(true)),
            "internet that never dropped leaves the other components alone"
        );
        assert!(!resets_component_health(GemConnectionComponent::Internet, true, None), "a first reading is not a recovery");
        assert!(
            !resets_component_health(GemConnectionComponent::Internet, false, Some(false)),
            "losing internet keeps what is known"
        );
        assert!(
            !resets_component_health(GemConnectionComponent::Api, true, Some(false)),
            "only internet recovery invalidates the other components"
        );
    }
}
