use async_trait::async_trait;

#[async_trait]
pub trait ConsumerStatusReporter: Send + Sync {
    async fn report(&self, name: &str, status: ConsumerStatus);
}

pub enum ConsumerStatus {
    Started { queue_wait_seconds: Option<u64> },
    Success { duration_milliseconds: u64 },
    Skipped { duration_milliseconds: u64 },
    Error { duration_milliseconds: u64 },
}
