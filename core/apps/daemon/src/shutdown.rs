use std::time::Duration;

pub use http_server::{ShutdownReceiver, ShutdownSender, shutdown_channel as channel, spawn_signal_handler};
pub use job_runner::sleep_or_shutdown;

pub async fn wait_with_timeout(handles: Vec<tokio::task::JoinHandle<()>>, timeout: Duration) -> bool {
    tokio::time::timeout(timeout, futures::future::join_all(handles)).await.is_ok()
}
