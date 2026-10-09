use super::api_clients::setup_api_client_grants;
use super::repository::{Repository, SetupSeed};
use super::scan_addresses::setup_scan_addresses;
use crate::Services;
use config_keys::{ConfigKey, ConfigParamKey};
use gem_tracing::info_with_fields;
use primitives::{Asset, AssetTag, Chain, FiatProviderName, NFTChain, PlatformStore as PrimitivePlatformStore, PriceProvider, Release};
use search_index::{INDEX_CONFIGS, INDEX_PRIMARY_KEY};
use settings::Settings;
use std::sync::Arc;
use streamer::{ExchangeKind, ExchangeName, QueueName};

pub async fn run_setup(settings: Settings) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info_with_fields!("setup", step = "init");

    let services = Services::new(Arc::new(settings))?;
    let repository = services.setup_repository();
    repository.update_schema().await?;
    info_with_fields!("setup", step = "postgres migrations complete");

    seed_database(repository.as_ref()).await?;
    setup_scan_addresses(repository.as_ref()).await?;
    setup_search_index(&services).await?;
    setup_queues(&services).await?;

    info_with_fields!("setup", step = "complete");
    Ok(())
}

pub(super) async fn seed_database(repository: &dyn Repository) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let chains = Chain::all();
    let seed = SetupSeed {
        assets: chains.iter().map(|chain| Asset::from_chain(*chain).as_basic_primitive()).collect(),
        chains,
        fiat_providers: FiatProviderName::all(),
        api_client_grants: setup_api_client_grants(),
        releases: PrimitivePlatformStore::all().into_iter().map(|store| Release::new(store, "1.0.0".to_string(), false)).collect(),
        tags: AssetTag::all(),
        price_providers: PriceProvider::all(),
        config_keys: ConfigKey::all(),
        config_params: ConfigParamKey::all(),
    };
    repository.set_seed(seed).await?;
    Ok(())
}

async fn setup_search_index(services: &Services) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info_with_fields!("setup", step = "search index", indexes = format!("{:?}", INDEX_CONFIGS.iter().map(|c| c.name).collect::<Vec<_>>()));

    let search_index_client = services.search_index().await?;
    search_index_client.setup(INDEX_CONFIGS, INDEX_PRIMARY_KEY).await.unwrap();

    Ok(())
}

async fn setup_queues(services: &Services) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info_with_fields!("setup", step = "queues");

    let chain_queues = QueueName::chain_queues();
    let non_chain_queues: Vec<_> = QueueName::all().into_iter().filter(|q| !chain_queues.contains(q)).collect();
    let exchanges = ExchangeName::all();
    let chains = Chain::all();

    let stream_producer = services.stream_producer("setup", streamer::no_shutdown()).await?;
    stream_producer.declare_queues(non_chain_queues).await?;
    stream_producer.declare_exchanges(exchanges.clone()).await?;

    info_with_fields!(
        "setup",
        step = "queue exchanges for chain-based consumers",
        queues = format!("{:?}", chain_queues.iter().map(ToString::to_string).collect::<Vec<_>>()),
        chains = format!("{:?}", chains)
    );

    for queue in &chain_queues {
        let exchange_name = format!("{}_exchange", queue);
        stream_producer.declare_exchange(&exchange_name, ExchangeKind::Topic).await?;
        for chain in queue_supported_chains(queue, &chains) {
            stream_producer.bind_queue_routing_key(queue.clone(), chain.as_ref()).await?;
        }
    }

    for exchange in &exchanges {
        let exchange_queues = exchange.queues();
        if exchange_queues.is_empty() {
            continue;
        }
        info_with_fields!(
            "setup",
            step = "exchange bindings",
            exchange = exchange.to_string(),
            queues = format!("{:?}", exchange_queues.iter().map(ToString::to_string).collect::<Vec<_>>())
        );
        for queue in &exchange_queues {
            for chain in queue_supported_chains(queue, &chains) {
                let queue_name = format!("{}.{}", queue, chain.as_ref());
                stream_producer.bind_queue(&queue_name, &exchange.to_string(), chain.as_ref()).await?;
            }
        }
    }

    Ok(())
}

fn queue_supported_chains(queue: &QueueName, all_chains: &[Chain]) -> Vec<Chain> {
    match queue {
        QueueName::FetchNftAssociations => NFTChain::all().into_iter().map(Into::into).collect(),
        _ => all_chains.to_vec(),
    }
}
