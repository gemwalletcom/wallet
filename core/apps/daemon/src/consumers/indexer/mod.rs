use std::error::Error;
use std::sync::Arc;

use futures::future;
use primitives::{AssetId, Chain, NFTChain, PriceId, TransactionIdRequest};
use services::Services;
use settings::Settings;
use streamer::{
    ChainAddressPayload, ConsumerConfig, ConsumerStatusReporter, FetchAssetAssociationsPayload, FetchAssetsPayload, FetchBlocksPayload, FetchListPayload, FetchNFTAssetPayload, FetchPricesPayload, QueueName, ShutdownReceiver,
    StreamConnection, StreamProducer, StreamReader, run_consumer,
};

use crate::consumers::runner::ChainConsumerRunner;
use crate::consumers::{consumer_config, consumer_user_agent, reader_config};
use crate::model::{IndexerConsumer, IndexerService};

pub async fn run_consumer_indexer(settings: Settings, service: IndexerService, shutdown_rx: ShutdownReceiver, reporter: Arc<dyn ConsumerStatusReporter>, only: Option<IndexerConsumer>) -> Result<(), Box<dyn Error + Send + Sync>> {
    use IndexerConsumer::*;

    let services = Services::new(Arc::new(settings))?;

    let selected = match only {
        Some(consumer) if service.consumers().contains(&consumer) => vec![consumer],
        Some(consumer) => return Err(format!("Indexer consumer {} does not belong to {}", consumer.as_ref(), service.as_ref()).into()),
        None => service.consumers().to_vec(),
    };

    let handles: Vec<_> = selected
        .into_iter()
        .map(|kind| {
            let services = services.clone();
            let shutdown_rx = shutdown_rx.clone();
            let reporter = reporter.clone();
            tokio::spawn(async move {
                match kind {
                    FetchBlocks => run_fetch_blocks(services, shutdown_rx, reporter).await,
                    FetchAssets => run_fetch_assets(services, shutdown_rx, reporter).await,
                    FetchAssetStatus => run_fetch_asset_status(services, shutdown_rx, reporter).await,
                    FetchAssetAssociations => run_fetch_asset_associations(services, shutdown_rx, reporter).await,
                    FetchLists => run_fetch_lists(services, shutdown_rx, reporter).await,
                    FetchPrices => run_fetch_prices(services, shutdown_rx, reporter).await,
                    FetchPricesMetadata => run_fetch_prices_metadata(services, shutdown_rx, reporter).await,
                    FetchTokenAssociations => run_fetch_token_associations(services, shutdown_rx, reporter).await,
                    FetchCoinAssociations => run_fetch_coin_associations(services, shutdown_rx, reporter).await,
                    FetchNftAssociations => run_fetch_nft_associations(services, shutdown_rx, reporter).await,
                    FetchNftAssets => run_fetch_nft_assets(services, shutdown_rx, reporter).await,
                    FetchAddressTransactions => run_fetch_transaction_associations(services, shutdown_rx, reporter).await,
                    FetchTransactions => run_fetch_transactions(services, shutdown_rx, reporter).await,
                }
            })
        })
        .collect();

    for handle in future::join_all(handles).await {
        handle??;
    }
    Ok(())
}

struct QueueReader {
    name: String,
    connection: StreamConnection,
    reader: StreamReader,
}

async fn queue_reader(settings: &Settings, queue: &QueueName) -> Result<QueueReader, Box<dyn Error + Send + Sync>> {
    let name = queue.to_string();
    let connection = StreamConnection::new(&settings.rabbitmq.url, name.clone()).await?;
    let reader = StreamReader::from_connection(&connection, reader_config(&settings.rabbitmq, name.clone())).await?;
    Ok(QueueReader { name, connection, reader })
}

async fn run_fetch_asset_associations(services: Services, shutdown_rx: ShutdownReceiver, reporter: Arc<dyn ConsumerStatusReporter>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let settings = services.settings();
    let queue = QueueName::FetchAssetAssociations;
    let queue_reader = queue_reader(&settings, &queue).await?;
    let consumer = services.fetch_asset_associations_consumer();
    run_consumer::<FetchAssetAssociationsPayload, _, usize>(&queue_reader.name, queue_reader.reader, queue, None, consumer, consumer_config(&settings.consumer), shutdown_rx, reporter).await
}

async fn run_fetch_blocks(services: Services, shutdown_rx: ShutdownReceiver, reporter: Arc<dyn ConsumerStatusReporter>) -> Result<(), Box<dyn Error + Send + Sync>> {
    ChainConsumerRunner::new(services, QueueName::FetchBlocks, shutdown_rx, reporter)
        .await?
        .run(|runner, chain| async move {
            let queue = QueueName::FetchBlocks;
            let name = format!("{}.{}", queue, chain.as_ref());
            let stream_reader = runner.stream_reader().await?;
            let consumer = runner.services.fetch_blocks_consumer(chain, &consumer_user_agent(&name), runner.stream_producer().await?);
            run_consumer::<FetchBlocksPayload, _, usize>(&name, stream_reader, queue, Some(chain.as_ref()), consumer, runner.config, runner.shutdown_rx, runner.reporter).await
        })
        .await
}

async fn run_fetch_assets(services: Services, shutdown_rx: ShutdownReceiver, reporter: Arc<dyn ConsumerStatusReporter>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let settings = services.settings();
    let queue = QueueName::FetchAssets;
    let queue_reader = queue_reader(&settings, &queue).await?;
    let stream_producer = StreamProducer::from_connection(&queue_reader.connection, shutdown_rx.clone()).await?;
    let consumer = services.fetch_assets_consumer(&consumer_user_agent(&queue_reader.name), stream_producer).await?;
    run_consumer::<FetchAssetsPayload, _, usize>(&queue_reader.name, queue_reader.reader, queue, None, consumer, consumer_config(&settings.consumer), shutdown_rx, reporter).await
}

async fn run_fetch_asset_status(services: Services, shutdown_rx: ShutdownReceiver, reporter: Arc<dyn ConsumerStatusReporter>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let settings = services.settings();
    let queue = QueueName::FetchAssetStatus;
    let queue_reader = queue_reader(&settings, &queue).await?;
    let consumer = services.fetch_asset_status_consumer().await?;
    run_consumer::<AssetId, _, bool>(&queue_reader.name, queue_reader.reader, queue, None, consumer, consumer_config(&settings.consumer), shutdown_rx, reporter).await
}

async fn run_fetch_lists(services: Services, shutdown_rx: ShutdownReceiver, reporter: Arc<dyn ConsumerStatusReporter>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let settings = services.settings();
    let queue = QueueName::FetchLists;
    let queue_reader = queue_reader(&settings, &queue).await?;
    let consumer = services.fetch_list_consumer();
    run_consumer::<FetchListPayload, _, u32>(&queue_reader.name, queue_reader.reader, queue, None, consumer, consumer_config(&settings.consumer), shutdown_rx, reporter).await
}

async fn run_fetch_prices_metadata(services: Services, shutdown_rx: ShutdownReceiver, reporter: Arc<dyn ConsumerStatusReporter>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let settings = services.settings();
    let queue = QueueName::FetchPricesMetadata;
    let queue_reader = queue_reader(&settings, &queue).await?;
    let consumer = services.fetch_prices_metadata_consumer().await?;
    let config = ConsumerConfig {
        skip_on_error: true,
        ..consumer_config(&settings.consumer)
    };
    run_consumer::<PriceId, _, usize>(&queue_reader.name, queue_reader.reader, queue, None, consumer, config, shutdown_rx, reporter).await
}

async fn run_fetch_prices(services: Services, shutdown_rx: ShutdownReceiver, reporter: Arc<dyn ConsumerStatusReporter>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let settings = services.settings();
    let queue = QueueName::FetchPrices;
    let queue_reader = queue_reader(&settings, &queue).await?;
    let consumer = services.fetch_prices_consumer().await?;
    run_consumer::<FetchPricesPayload, _, usize>(&queue_reader.name, queue_reader.reader, queue, None, consumer, consumer_config(&settings.consumer), shutdown_rx, reporter).await
}

async fn run_fetch_token_associations(services: Services, shutdown_rx: ShutdownReceiver, reporter: Arc<dyn ConsumerStatusReporter>) -> Result<(), Box<dyn Error + Send + Sync>> {
    ChainConsumerRunner::new(services, QueueName::FetchTokenAssociations, shutdown_rx, reporter)
        .await?
        .run(|runner, chain| async move {
            let queue = QueueName::FetchTokenAssociations;
            let name = format!("{}.{}", queue, chain.as_ref());
            let stream_reader = runner.stream_reader().await?;
            let consumer = runner.services.fetch_token_addresses_consumer(chain, &consumer_user_agent(&name), runner.stream_producer().await?).await?;
            run_consumer::<ChainAddressPayload, _, usize>(&name, stream_reader, queue, Some(chain.as_ref()), consumer, runner.config, runner.shutdown_rx, runner.reporter).await
        })
        .await
}

async fn run_fetch_coin_associations(services: Services, shutdown_rx: ShutdownReceiver, reporter: Arc<dyn ConsumerStatusReporter>) -> Result<(), Box<dyn Error + Send + Sync>> {
    ChainConsumerRunner::new(services, QueueName::FetchCoinAssociations, shutdown_rx, reporter)
        .await?
        .run(|runner, chain| async move {
            let queue = QueueName::FetchCoinAssociations;
            let name = format!("{}.{}", queue, chain.as_ref());
            let stream_reader = runner.stream_reader().await?;
            let consumer = runner.services.fetch_coin_addresses_consumer(chain, &consumer_user_agent(&name)).await?;
            run_consumer::<ChainAddressPayload, _, String>(&name, stream_reader, queue, Some(chain.as_ref()), consumer, runner.config, runner.shutdown_rx, runner.reporter).await
        })
        .await
}

async fn run_fetch_nft_associations(services: Services, shutdown_rx: ShutdownReceiver, reporter: Arc<dyn ConsumerStatusReporter>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let chains: Vec<Chain> = NFTChain::all().into_iter().map(Into::into).collect();
    ChainConsumerRunner::new(services, QueueName::FetchNftAssociations, shutdown_rx, reporter)
        .await?
        .run_for_chains(chains, |runner, chain| async move {
            let queue = QueueName::FetchNftAssociations;
            let name = format!("{}.{}", queue, chain.as_ref());
            let stream_reader = StreamReader::from_connection(&runner.connection, reader_config(&runner.settings.rabbitmq, name.clone())).await?;
            let consumer = runner.services.fetch_nft_assets_addresses_consumer().await?;
            run_consumer::<ChainAddressPayload, _, usize>(&name, stream_reader, queue, Some(chain.as_ref()), consumer, runner.config, runner.shutdown_rx, runner.reporter).await
        })
        .await
}

async fn run_fetch_nft_assets(services: Services, shutdown_rx: ShutdownReceiver, reporter: Arc<dyn ConsumerStatusReporter>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let settings = services.settings();
    let queue = QueueName::FetchNFTCollectionAssets;
    let queue_reader = queue_reader(&settings, &queue).await?;
    let consumer = services.fetch_nft_asset_consumer().await?;
    run_consumer::<FetchNFTAssetPayload, _, usize>(&queue_reader.name, queue_reader.reader, queue, None, consumer, consumer_config(&settings.consumer), shutdown_rx, reporter).await
}

async fn run_fetch_transaction_associations(services: Services, shutdown_rx: ShutdownReceiver, reporter: Arc<dyn ConsumerStatusReporter>) -> Result<(), Box<dyn Error + Send + Sync>> {
    ChainConsumerRunner::new(services, QueueName::FetchAddressTransactions, shutdown_rx, reporter)
        .await?
        .run(|runner, chain| async move {
            let queue = QueueName::FetchAddressTransactions;
            let name = format!("{}.{}", queue, chain.as_ref());
            let stream_reader = runner.stream_reader().await?;
            let consumer = runner.services.fetch_address_transactions_consumer(chain, &consumer_user_agent(&name), runner.stream_producer().await?).await?;
            run_consumer::<ChainAddressPayload, _, usize>(&name, stream_reader, queue, Some(chain.as_ref()), consumer, runner.config, runner.shutdown_rx, runner.reporter).await
        })
        .await
}

async fn run_fetch_transactions(services: Services, shutdown_rx: ShutdownReceiver, reporter: Arc<dyn ConsumerStatusReporter>) -> Result<(), Box<dyn Error + Send + Sync>> {
    ChainConsumerRunner::new(services, QueueName::FetchTransactions, shutdown_rx, reporter)
        .await?
        .run(|runner, chain| async move {
            let queue = QueueName::FetchTransactions;
            let name = format!("{}.{}", queue, chain.as_ref());
            let stream_reader = runner.stream_reader().await?;
            let consumer = runner.services.fetch_transaction_consumer(chain, &consumer_user_agent(&name), runner.stream_producer().await?).await?;
            run_consumer::<TransactionIdRequest, _, usize>(&name, stream_reader, queue, Some(chain.as_ref()), consumer, runner.config, runner.shutdown_rx, runner.reporter).await
        })
        .await
}
