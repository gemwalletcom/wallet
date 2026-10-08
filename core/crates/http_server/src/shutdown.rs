use gem_tracing::info_with_fields;
use tokio::sync::watch;
use tokio::task::JoinHandle;

pub type ShutdownSender = watch::Sender<bool>;
pub type ShutdownReceiver = watch::Receiver<bool>;

pub fn shutdown_channel() -> (ShutdownSender, ShutdownReceiver) {
    watch::channel(false)
}

pub fn spawn_signal_handler(sender: ShutdownSender) -> JoinHandle<()> {
    tokio::spawn(async move {
        let signal = wait_for_signal().await;
        info_with_fields!("shutdown signal received", signal = signal, status = "ok");
        let _ = sender.send(true);
    })
}

async fn wait_for_signal() -> &'static str {
    let ctrl_c = tokio::signal::ctrl_c();

    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(_) => std::future::pending::<()>().await,
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => "SIGINT",
        _ = terminate => "SIGTERM",
    }
}
