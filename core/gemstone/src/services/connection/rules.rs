use std::time::Duration;

use primitives::ConnectionStatus;

use super::{GemReconnection, GemRefreshKind};
use crate::constants::PING_INTERVAL;
use crate::models::GemConnectionComponent;

const RECONNECT_MULTIPLIER_MILLISECONDS: f64 = 300.0;
const RECONNECT_MAXIMUM_MILLISECONDS: f64 = 30_000.0;
const MARKET_REFRESH: Duration = Duration::from_secs(60);
const WALLET_REFRESH: Duration = Duration::from_secs(300);
const STREAMING_REFRESH: Duration = Duration::from_secs(900);

pub fn reconnection(attempt: u32, connected: Duration) -> GemReconnection {
    let attempt = if connected >= PING_INTERVAL { 0 } else { attempt };
    GemReconnection {
        next_attempt: attempt.saturating_add(1),
        delay: reconnect_delay(attempt),
    }
}

fn reconnect_delay(attempt: u32) -> Duration {
    Duration::from_millis((RECONNECT_MULTIPLIER_MILLISECONDS * f64::from(attempt).exp()).min(RECONNECT_MAXIMUM_MILLISECONDS) as u64)
}

pub fn refresh_interval(kind: GemRefreshKind, status: ConnectionStatus) -> Duration {
    match (kind, status) {
        (GemRefreshKind::Chart | GemRefreshKind::Confirm, _) => MARKET_REFRESH,
        (GemRefreshKind::Market | GemRefreshKind::Wallet, ConnectionStatus::Online) => STREAMING_REFRESH,
        (GemRefreshKind::Market, ConnectionStatus::NoInternet | ConnectionStatus::NoService) => MARKET_REFRESH,
        (GemRefreshKind::Wallet, ConnectionStatus::NoInternet | ConnectionStatus::NoService) => WALLET_REFRESH,
    }
}

pub fn resets_component_health(component: GemConnectionComponent, is_healthy: bool, was_healthy: Option<bool>) -> bool {
    component == GemConnectionComponent::Internet && is_healthy && was_healthy == Some(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_charts_and_confirmation_refresh_even_when_prices_are_streaming() {
        for status in [ConnectionStatus::Online, ConnectionStatus::NoService, ConnectionStatus::NoInternet] {
            for kind in [GemRefreshKind::Chart, GemRefreshKind::Confirm] {
                assert_eq!(refresh_interval(kind, status), Duration::from_secs(60));
            }
        }
    }

    #[test]
    fn test_keepalive_pings_before_the_reconnect_backoff_caps_out() {
        assert!(PING_INTERVAL >= reconnect_delay(0), "a keepalive that fires faster than the first reconnect would ping a socket that is still coming up");
    }

    #[test]
    fn test_reconnect_delay_grows_exponentially_and_caps() {
        assert_eq!(reconnect_delay(0), Duration::from_millis(300));
        assert_eq!(reconnect_delay(1), Duration::from_millis(815));
        assert_eq!(reconnect_delay(3), Duration::from_millis(6_025));
        assert_eq!(reconnect_delay(4), Duration::from_millis(16_379));
        assert_eq!(
            reconnect_delay(5),
            Duration::from_millis(RECONNECT_MAXIMUM_MILLISECONDS as u64),
            "the curve is capped from the attempt it first exceeds the maximum"
        );
        assert_eq!(reconnect_delay(u32::MAX), Duration::from_millis(RECONNECT_MAXIMUM_MILLISECONDS as u64), "an overflowing exponent still yields the cap");
    }

    #[test]
    fn test_reconnection() {
        let ping_interval = PING_INTERVAL;
        assert_eq!(
            reconnection(0, Duration::ZERO),
            GemReconnection {
                next_attempt: 1,
                delay: Duration::from_millis(300)
            }
        );
        assert_eq!(
            reconnection(3, Duration::ZERO),
            GemReconnection {
                next_attempt: 4,
                delay: Duration::from_millis(6_025)
            },
            "a connection that never opened keeps backing off"
        );
        assert_eq!(
            reconnection(3, ping_interval - Duration::from_millis(1)),
            GemReconnection {
                next_attempt: 4,
                delay: Duration::from_millis(6_025)
            },
            "a connection the server drops right away keeps backing off"
        );
        assert_eq!(
            reconnection(3, ping_interval),
            GemReconnection {
                next_attempt: 1,
                delay: Duration::from_millis(300)
            },
            "a connection that outlived a keepalive restarts the backoff"
        );
    }

    #[test]
    fn test_reconnect_delay_never_decreases() {
        let delays: Vec<Duration> = (0..10).map(reconnect_delay).collect();
        assert!(delays.windows(2).all(|pair| pair[0] <= pair[1]));
    }

    #[test]
    fn test_a_delivering_socket_slows_every_screen_to_the_same_safety_net() {
        assert_eq!(refresh_interval(GemRefreshKind::Market, ConnectionStatus::Online), Duration::from_secs(900));
        assert_eq!(refresh_interval(GemRefreshKind::Wallet, ConnectionStatus::Online), Duration::from_secs(900));
        assert!(
            refresh_interval(GemRefreshKind::Market, ConnectionStatus::Online) > refresh_interval(GemRefreshKind::Market, ConnectionStatus::NoService),
            "a screen polls harder when nothing is pushing to it"
        );
    }

    #[test]
    fn test_market_data_refreshes_faster_than_wallet_data_when_nothing_is_pushing() {
        assert_eq!(refresh_interval(GemRefreshKind::Market, ConnectionStatus::NoService), Duration::from_secs(60));
        assert_eq!(refresh_interval(GemRefreshKind::Wallet, ConnectionStatus::NoService), Duration::from_secs(300));
        assert_eq!(
            refresh_interval(GemRefreshKind::Wallet, ConnectionStatus::NoInternet),
            refresh_interval(GemRefreshKind::Wallet, ConnectionStatus::NoService),
            "a screen cannot tell a dead socket from a dead network, and polls the same either way"
        );
    }

    #[test]
    fn test_only_recovering_internet_resets_component_health() {
        assert!(resets_component_health(GemConnectionComponent::Internet, true, Some(false)));
        assert!(!resets_component_health(GemConnectionComponent::Internet, true, Some(true)), "internet that never dropped leaves the other components alone");
        assert!(!resets_component_health(GemConnectionComponent::Internet, true, None), "a first reading is not a recovery");
        assert!(!resets_component_health(GemConnectionComponent::Internet, false, Some(false)), "losing internet keeps what is known");
        assert!(!resets_component_health(GemConnectionComponent::Stream, true, Some(false)), "only internet recovery invalidates the other components");
    }
}
