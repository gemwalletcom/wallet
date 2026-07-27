use std::sync::{Arc, Mutex, MutexGuard};

use primitives::Chain;
use tokio::sync::Notify;
use tokio::time::Instant;

use super::failure_tracker::FailureTracker;
use super::telemetry::NodeTelemetry;
use crate::config::{FailureTriggerConfig, Url};

struct RequestFailureState {
    active_url: Url,
    tracker: FailureTracker,
    pending: bool,
}

struct RequestFailureSignalInner {
    chain: Chain,
    config: FailureTriggerConfig,
    state: Mutex<RequestFailureState>,
    notify: Notify,
}

#[derive(Clone)]
pub(super) struct RequestFailureSignal {
    inner: Arc<RequestFailureSignalInner>,
}

impl RequestFailureSignal {
    pub(super) fn new(chain: Chain, active_url: Url, config: FailureTriggerConfig) -> Self {
        Self {
            inner: Arc::new(RequestFailureSignalInner {
                chain,
                config,
                state: Mutex::new(RequestFailureState {
                    active_url,
                    tracker: FailureTracker::default(),
                    pending: false,
                }),
                notify: Notify::new(),
            }),
        }
    }

    pub(super) fn report(&self, url: &Url, failed: bool) {
        let should_notify = {
            let mut state = self.state();
            if state.active_url != *url {
                return;
            }
            if state.tracker.record(failed, Instant::now(), &self.inner.config) {
                state.pending = true;
                true
            } else {
                false
            }
        };
        if should_notify {
            NodeTelemetry::log_failure_trigger(self.inner.chain, url);
            self.inner.notify.notify_one();
        }
    }

    pub(super) async fn wait(&self) -> Url {
        loop {
            let notified = self.inner.notify.notified();
            if let Some(url) = self.take_pending() {
                return url;
            }
            notified.await;
        }
    }

    pub(super) fn check_completed(&self, url: &Url) {
        let mut state = self.state();
        if state.active_url != *url {
            state.active_url = url.clone();
            state.tracker = FailureTracker::default();
        } else {
            state.tracker.clear_samples();
        }
        state.pending = false;
    }

    fn state(&self) -> MutexGuard<'_, RequestFailureState> {
        self.inner.state.lock().unwrap()
    }

    fn take_pending(&self) -> Option<Url> {
        let mut state = self.state();
        if !state.pending {
            return None;
        }
        state.pending = false;
        Some(state.active_url.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::sync::url;
    use primitives::MINUTE;

    #[tokio::test]
    async fn test_tracks_only_the_active_url() {
        let first = url("https://first");
        let fallback = url("https://fallback");
        let signal = RequestFailureSignal::new(
            Chain::Ethereum,
            first.clone(),
            FailureTriggerConfig {
                failures: 2,
                rate: 100,
                window: MINUTE,
            },
        );

        signal.report(&fallback, true);
        signal.report(&fallback, true);
        assert!(!signal.state().pending);

        signal.report(&first, true);
        signal.check_completed(&first);
        signal.report(&first, true);
        assert!(!signal.state().pending);
        signal.report(&first, true);
        assert_eq!(signal.wait().await, first);

        signal.check_completed(&fallback);
        signal.report(&first, true);
        signal.report(&first, true);
        assert!(!signal.state().pending);

        signal.report(&fallback, true);
        signal.report(&fallback, true);
        assert_eq!(signal.wait().await, fallback);
    }
}
