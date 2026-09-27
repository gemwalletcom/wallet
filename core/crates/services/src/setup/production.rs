use super::api_clients::setup_api_client_grants;
use super::database::run_migrations;
use super::scan_addresses::setup_scan_addresses;
use crate::Services;
use config_keys::{ConfigKey, ConfigParamKey};
use gem_tracing::info_with_fields;
use primitives::{Asset, AssetTag, Chain, FiatProviderName, NFTChain, PlatformStore as PrimitivePlatformStore, PriceProvider, Release};
use search_index::{INDEX_CONFIGS, INDEX_PRIMARY_KEY};
use settings::Settings;
use std::collections::HashSet;
use std::sync::Arc;
use storage::{ApiClientsRepository, AssetsRepository, ChainsRepository, ConfigRepository, Database, DatabaseError, FiatRepository, ParserStateRepository, PricesProvidersRepository, ReleasesRepository, TagRepository};
use streamer::{ExchangeKind, ExchangeName, QueueName};

pub async fn run_setup(settings: Settings) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info_with_fields!("setup", step = "init");

    let services = Services::new(Arc::new(settings))?;
    let database = services.database();
    run_migrations(&database, "setup").await?;

    setup_database(&database).await?;
    setup_scan_addresses(&database).await?;
    setup_search_index(&services).await?;
    setup_queues(&services).await?;

    info_with_fields!("setup", step = "complete");
    Ok(())
}

pub(super) async fn setup_database(database: &Database) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    database
        .run(|client| -> Result<_, DatabaseError> {
            let chains = Chain::all();
            info_with_fields!("setup", step = "chains", chains = format!("{:?}", chains));

            info_with_fields!("setup", step = "add chains");
            let _ = client.add_chains(chains.clone());

            info_with_fields!("setup", step = "parser state");
            for chain in chains.iter().copied() {
                let _ = client.add_parser_state(chain, chain.block_time() as i32);
            }

            info_with_fields!("setup", step = "assets");
            let assets = chains.into_iter().map(|x| Asset::from_chain(x).as_basic_primitive()).collect::<Vec<_>>();
            let _ = client.add_assets(assets);

            info_with_fields!("setup", step = "fiat providers");
            let _ = client.add_fiat_providers(FiatProviderName::all());

            info_with_fields!("setup", step = "api clients");
            let _ = client.add_api_client_grants(setup_api_client_grants());

            info_with_fields!("setup", step = "releases");
            let releases = PrimitivePlatformStore::all().into_iter().map(|store| Release::new(store, "1.0.0".to_string(), false)).collect::<Vec<_>>();
            let _ = client.add_releases(releases);

            info_with_fields!("setup", step = "assets tags");
            let _ = client.add_tags(AssetTag::all());

            info_with_fields!("setup", step = "prices providers");
            let _ = client.add_prices_providers(PriceProvider::all());

            info_with_fields!("setup", step = "config");
            let _ = client.add_config_keys(ConfigKey::all());

            info_with_fields!("setup", step = "param config");
            let _ = client.add_config_params(ConfigParamKey::all());

            info_with_fields!("setup", step = "cleanup stale config keys");
            let valid: HashSet<String> = ConfigKey::all().into_iter().map(|k| k.as_ref().to_string()).chain(ConfigParamKey::all().into_iter().map(|k| k.key())).collect();
            let stale: Vec<String> = client.get_config_keys()?.into_iter().filter(|k| !valid.contains(k)).collect();
            if !stale.is_empty() {
                info_with_fields!("setup", step = "delete stale config keys", count = stale.len(), keys = format!("{:?}", stale));
                let _ = client.delete_keys(stale);
            }

            Ok(())
        })
        .await?;
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
