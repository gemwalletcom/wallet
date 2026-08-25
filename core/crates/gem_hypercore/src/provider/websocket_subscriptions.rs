use std::collections::HashSet;

use crate::models::websocket::{HyperliquidMethod, HyperliquidRequest, HyperliquidSubscription};

#[derive(Debug, Default)]
pub struct WebSocketSubscriptions {
    requested: HashSet<HyperliquidSubscription>,
    subscribed: Option<HashSet<HyperliquidSubscription>>,
}

impl WebSocketSubscriptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn subscribe(&mut self, subscription: HyperliquidSubscription) -> Vec<HyperliquidRequest> {
        self.requested.insert(subscription.clone());

        match &mut self.subscribed {
            None => Vec::new(),
            Some(subscribed) => match subscribed.insert(subscription.clone()) {
                true => vec![request(HyperliquidMethod::Subscribe, subscription)],
                false => Vec::new(),
            },
        }
    }

    pub fn unsubscribe(&mut self, subscription: &HyperliquidSubscription) -> Vec<HyperliquidRequest> {
        self.requested.remove(subscription);

        match &mut self.subscribed {
            None => Vec::new(),
            Some(subscribed) => match subscribed.remove(subscription) {
                true => vec![request(HyperliquidMethod::Unsubscribe, subscription.clone())],
                false => Vec::new(),
            },
        }
    }

    pub fn connected(&mut self, account_subscriptions: Vec<HyperliquidSubscription>) -> Vec<HyperliquidRequest> {
        let subscribed = self.subscribed.insert(HashSet::new());

        account_subscriptions
            .into_iter()
            .chain(self.requested.iter().cloned())
            .filter(|subscription| subscribed.insert(subscription.clone()))
            .map(|subscription| request(HyperliquidMethod::Subscribe, subscription))
            .collect()
    }

    pub fn disconnected(&mut self) {
        self.subscribed = None;
    }
}

fn request(method: HyperliquidMethod, subscription: HyperliquidSubscription) -> HyperliquidRequest {
    HyperliquidRequest { method, subscription }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candle(symbol: &str) -> HyperliquidSubscription {
        HyperliquidSubscription::Candle {
            symbol: symbol.to_string(),
            interval: "30m".to_string(),
        }
    }

    fn account_state() -> HyperliquidSubscription {
        HyperliquidSubscription::AccountState { address: "0xabc".to_string() }
    }

    fn subscriptions(requests: &[HyperliquidRequest]) -> Vec<(HyperliquidMethod, HyperliquidSubscription)> {
        requests.iter().map(|request| (request.method, request.subscription.clone())).collect()
    }

    #[test]
    fn test_subscribe_before_connect_is_sent_once_on_connect() {
        let mut state = WebSocketSubscriptions::new();

        assert!(state.subscribe(candle("UNI")).is_empty());

        let connected = state.connected(vec![account_state()]);

        assert_eq!(
            subscriptions(&connected),
            vec![(HyperliquidMethod::Subscribe, account_state()), (HyperliquidMethod::Subscribe, candle("UNI")),]
        );
    }

    #[test]
    fn test_subscribe_while_connected_is_sent_immediately_and_not_repeated() {
        let mut state = WebSocketSubscriptions::new();
        state.connected(vec![]);

        assert_eq!(subscriptions(&state.subscribe(candle("UNI"))), vec![(HyperliquidMethod::Subscribe, candle("UNI"))]);
        assert!(state.subscribe(candle("UNI")).is_empty());
    }

    #[test]
    fn test_reconnect_resends_everything_once() {
        let mut state = WebSocketSubscriptions::new();
        state.connected(vec![account_state()]);
        state.subscribe(candle("UNI"));

        state.disconnected();
        let reconnected = state.connected(vec![account_state()]);

        assert_eq!(
            subscriptions(&reconnected),
            vec![(HyperliquidMethod::Subscribe, account_state()), (HyperliquidMethod::Subscribe, candle("UNI")),]
        );
    }

    #[test]
    fn test_unsubscribe_while_disconnected_drops_it_from_the_next_connect() {
        let mut state = WebSocketSubscriptions::new();
        state.connected(vec![]);
        state.subscribe(candle("UNI"));

        state.disconnected();
        assert!(state.unsubscribe(&candle("UNI")).is_empty());

        assert!(state.connected(vec![]).is_empty());
    }

    #[test]
    fn test_unsubscribe_is_sent_once() {
        let mut state = WebSocketSubscriptions::new();
        state.connected(vec![]);
        state.subscribe(candle("UNI"));

        assert_eq!(subscriptions(&state.unsubscribe(&candle("UNI"))), vec![(HyperliquidMethod::Unsubscribe, candle("UNI"))]);
        assert!(state.unsubscribe(&candle("UNI")).is_empty());
    }

    #[test]
    fn test_connected_without_disconnect_resends_for_the_new_connection() {
        let mut state = WebSocketSubscriptions::new();
        state.subscribe(candle("UNI"));
        state.connected(vec![account_state()]);

        let reconnected = state.connected(vec![account_state()]);

        assert_eq!(
            subscriptions(&reconnected),
            vec![(HyperliquidMethod::Subscribe, account_state()), (HyperliquidMethod::Subscribe, candle("UNI"))]
        );
    }

    #[test]
    fn test_account_subscriptions_are_not_kept_across_reconnect() {
        let mut state = WebSocketSubscriptions::new();
        state.connected(vec![account_state()]);

        state.disconnected();

        assert!(state.connected(vec![]).is_empty());
    }
}
