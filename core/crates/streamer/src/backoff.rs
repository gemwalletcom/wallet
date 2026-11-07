use std::time::Duration;

use crate::Retry;

pub struct Backoff {
    retry: Retry,
    delay: Duration,
    failures: u32,
}

impl Backoff {
    pub fn new(retry: Retry) -> Self {
        Self { delay: retry.delay, retry, failures: 0 }
    }

    pub fn failed(&mut self, ran_for: Duration) -> Duration {
        if ran_for >= self.retry.timeout {
            self.delay = self.retry.delay;
            self.failures = 0;
        }
        self.failures += 1;
        let delay = self.delay;
        self.delay = (delay * 2).min(self.retry.timeout);
        delay
    }

    pub fn failures(&self) -> u32 {
        self.failures
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_failed() {
        let mut backoff = Backoff::new(Retry::new(Duration::from_secs(1), Duration::from_secs(4)));

        let delays: Vec<Duration> = (0..4).map(|_| backoff.failed(Duration::ZERO)).collect();
        assert_eq!(delays, [1, 2, 4, 4].map(Duration::from_secs));
        assert_eq!(backoff.failures(), 4);

        assert_eq!(backoff.failed(Duration::from_secs(3)), Duration::from_secs(4));
        assert_eq!(backoff.failures(), 5);

        assert_eq!(backoff.failed(Duration::from_secs(4)), Duration::from_secs(1));
        assert_eq!(backoff.failures(), 1);
        assert_eq!(backoff.failed(Duration::ZERO), Duration::from_secs(2));
    }
}
