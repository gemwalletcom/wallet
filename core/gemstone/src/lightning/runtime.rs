use std::future::Future;
use std::sync::LazyLock;

use gem_spark::LightningError;
use tokio::runtime::{Builder, Runtime};

const RUNTIME_WORKER_THREADS: usize = 2;

static RUNTIME: LazyLock<Result<Runtime, String>> = LazyLock::new(|| {
    Builder::new_multi_thread()
        .worker_threads(RUNTIME_WORKER_THREADS)
        .thread_name("gem-lightning")
        .enable_all()
        .build()
        .map_err(|error| error.to_string())
});

pub(crate) fn run<T: Send + 'static>(future: impl Future<Output = Result<T, LightningError>> + Send + 'static) -> impl Future<Output = Result<T, LightningError>> {
    let handle = RUNTIME.as_ref().map(|runtime| runtime.spawn(future)).map_err(|error| LightningError::Wallet(error.clone()));
    async move { handle?.await.map_err(|error| LightningError::Wallet(error.to_string()))? }
}
