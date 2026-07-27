use std::time::Duration;

use strum::AsRefStr;
use tokio::time::{Instant, sleep};

use super::evaluator::NodeHealthEvaluator;
use super::request_failure::RequestFailureSignal;

#[derive(Clone, Copy, AsRefStr)]
#[strum(serialize_all = "snake_case")]
pub(super) enum MonitorCycleSource {
    Scheduled,
    FailureTrigger,
}

pub(super) struct ChainMonitor {
    evaluator: NodeHealthEvaluator,
    interval: Duration,
    initial_delay: Duration,
    signal: RequestFailureSignal,
}

impl ChainMonitor {
    pub(super) fn new(evaluator: NodeHealthEvaluator, interval: Duration, initial_delay: Duration, signal: RequestFailureSignal) -> Self {
        Self {
            evaluator,
            interval,
            initial_delay,
            signal,
        }
    }

    pub(super) async fn run(self) {
        sleep(self.initial_delay).await;
        self.check(MonitorCycleSource::Scheduled).await;

        let scheduled_check = sleep(self.interval);
        tokio::pin!(scheduled_check);
        loop {
            tokio::select! {
                _ = &mut scheduled_check => {
                    self.check(MonitorCycleSource::Scheduled).await;
                    scheduled_check.as_mut().reset(Instant::now() + self.interval);
                }
                url = self.signal.wait() => {
                    if !self.evaluator.is_current(&url).await {
                        continue;
                    }

                    self.check(MonitorCycleSource::FailureTrigger).await;
                    scheduled_check.as_mut().reset(Instant::now() + self.interval);
                }
            }
        }
    }

    async fn check(&self, source: MonitorCycleSource) {
        if let Some(url) = self.evaluator.check(source).await {
            self.signal.check_completed(&url);
        }
    }
}
