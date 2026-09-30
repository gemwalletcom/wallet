use std::error::Error;
use std::sync::Arc;

use primitives::TransactionId;
use services::Services;
use settings::Settings;
use streamer::{ConsumerStatusReporter, PricesPayload, QueueName, ShutdownReceiver, TransactionsPayload, WalletStreamPayload, run_consumer};

use crate::consumers::runner::ChainConsumerRunner;
use crate::consumers::{consumer_config, reader_for_queue};

pub async fn run_consumer_store(settings: Settings, shutdown: ShutdownReceiver, reporter: Arc<dyn ConsumerStatusReporter>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let services = Services::new(Arc::new(settings))?;

    tokio::try_join!(
        run_store_transactions(services.clone(), shutdown.clone(), reporter.clone()),
        run_store_prices(services.clone(), shutdown.clone(), reporter.clone()),
        run_store_pending_transactions(services.clone(), shutdown.clone(), reporter.clone()),
        run_store_transactions_swaps(services.clone(), shutdown.clone(), reporter.clone()),
        run_store_transactions_perpetuals(services.clone(), shutdown.clone(), reporter.clone()),
        run_wallet_stream(services.clone(), shutdown.clone(), reporter.clone()),
    )?;

    Ok(())
}

async fn run_store_transactions(services: Services, shutdown: ShutdownReceiver, reporter: Arc<dyn ConsumerStatusReporter>) -> Result<(), Box<dyn Error + Send + Sync>> {
    ChainConsumerRunner::new(services, QueueName::StoreTransactions, shutdown, reporter)
        .await?
        .run(|runner, chain| async move {
            let queue = QueueName::StoreTransactions;
            let name = format!("{}.{}", queue, chain.as_ref());
            let stream_reader = runner.stream_reader().await?;
            let consumer = runner.services.store_transactions_consumer(runner.stream_producer().await?).await?;
            run_consumer::<TransactionsPayload, _, usize>(&name, stream_reader, queue, Some(chain.as_ref()), consumer, runner.config, runner.shutdown, runner.reporter).await
        })
        .await
}

async fn run_store_prices(services: Services, shutdown: ShutdownReceiver, reporter: Arc<dyn ConsumerStatusReporter>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let settings = services.settings();
    let queue = QueueName::StorePrices;
    let (name, stream_reader) = reader_for_queue(&settings, &queue, &shutdown).await?;
    let consumer = services.store_prices_consumer().await?;
    run_consumer::<PricesPayload, _, usize>(&name, stream_reader, queue, None, consumer, consumer_config(&settings.consumer), shutdown, reporter).await
}

async fn run_wallet_stream(services: Services, shutdown: ShutdownReceiver, reporter: Arc<dyn ConsumerStatusReporter>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let settings = services.settings();
    let queue = QueueName::WalletStreamEvents;
    let (name, stream_reader) = reader_for_queue(&settings, &queue, &shutdown).await?;
    let consumer = services.wallet_stream_consumer().await?;
    run_consumer::<WalletStreamPayload, _, usize>(&name, stream_reader, queue, None, consumer, consumer_config(&settings.consumer), shutdown, reporter).await
}

async fn run_store_pending_transactions(services: Services, shutdown: ShutdownReceiver, reporter: Arc<dyn ConsumerStatusReporter>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let settings = services.settings();
    let queue = QueueName::StorePendingTransactions;
    let (name, stream_reader) = reader_for_queue(&settings, &queue, &shutdown).await?;
    let consumer = services.store_pending_transactions_consumer().await?;
    run_consumer::<TransactionId, _, usize>(&name, stream_reader, queue, None, consumer, consumer_config(&settings.consumer), shutdown, reporter).await
}

async fn run_store_transactions_swaps(services: Services, shutdown: ShutdownReceiver, reporter: Arc<dyn ConsumerStatusReporter>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let settings = services.settings();
    let queue = QueueName::StoreTransactionsSwaps;
    let (name, stream_reader) = reader_for_queue(&settings, &queue, &shutdown).await?;
    let consumer = services.store_transactions_swaps_consumer();
    run_consumer::<TransactionId, _, usize>(&name, stream_reader, queue, None, consumer, consumer_config(&settings.consumer), shutdown, reporter).await
}

async fn run_store_transactions_perpetuals(services: Services, shutdown: ShutdownReceiver, reporter: Arc<dyn ConsumerStatusReporter>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let settings = services.settings();
    let queue = QueueName::StoreTransactionsPerpetuals;
    let (name, stream_reader) = reader_for_queue(&settings, &queue, &shutdown).await?;
    let consumer = services.store_transactions_perpetuals_consumer();
    run_consumer::<TransactionId, _, usize>(&name, stream_reader, queue, None, consumer, consumer_config(&settings.consumer), shutdown, reporter).await
}
