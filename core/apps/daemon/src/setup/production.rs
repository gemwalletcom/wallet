use super::api_clients::setup_api_client_grants;
use super::database::run_migrations;
use super::scan_addresses::setup_scan_addresses;
use gem_tracing::info_with_fields;
use primitives::{Asset, AssetTag, Chain, ConfigKey, ConfigParamKey, FiatProviderName, NFTChain, PlatformStore as PrimitivePlatformStore, PriceProvider};
use search_index::{INDEX_CONFIGS, INDEX_PRIMARY_KEY, SearchIndexClient};
use settings::Settings;
use std::collections::HashSet;
use storage::models::ConfigRow;
use storage::{ApiClientsRepository, AssetsRepository, ChainsRepository, ConfigRepository, Database, PricesProvidersRepository, ReleasesRepository, TagRepository};
use streamer::{ExchangeKind, ExchangeName, QueueName, StreamProducer, StreamProducerConfig};

pub async fn run_setup(settings: Settings) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info_with_fields!("setup", step = "init");

    let database = Database::new(&settings.postgres.url, settings.postgres.pool);
    run_migrations(&database, "setup")?;

    setup_database(&database)?;
    setup_scan_addresses(&database)?;
    setup_search_index(&settings).await?;
    setup_queues(&settings).await?;

    info_with_fields!("setup", step = "complete");
    Ok(())
}

fn setup_database(database: &Database) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let chains = Chain::all();
    info_with_fields!("setup", step = "chains", chains = format!("{:?}", chains));

    info_with_fields!("setup", step = "add chains");
    let _ = database.chains()?.add_chains(chains.clone());

    info_with_fields!("setup", step = "parser state");
    for chain in chains.iter().copied() {
        let _ = database.parser_state()?.add_parser_state(chain, chain.block_time() as i32);
    }

    info_with_fields!("setup", step = "assets");
    let assets = chains.into_iter().map(|x| Asset::from_chain(x).as_basic_primitive()).collect::<Vec<_>>();
    let _ = database.assets()?.add_assets(assets);

    info_with_fields!("setup", step = "fiat providers");
    let providers = FiatProviderName::all()
        .into_iter()
        .map(storage::models::FiatProviderRow::from_primitive)
        .collect::<Vec<_>>();
    let _ = database.fiat()?.add_fiat_providers(providers);

    info_with_fields!("setup", step = "api clients");
    let _ = database.api_clients()?.add_api_client_grants(setup_api_client_grants());

    info_with_fields!("setup", step = "releases");
    let releases = PrimitivePlatformStore::all()
        .into_iter()
        .map(|x| storage::models::ReleaseRow {
            platform_store: x.into(),
            version: "1.0.0".to_string(),
            upgrade_required: false,
            update_enabled: true,
        })
        .collect::<Vec<_>>();
    let _ = database.releases()?.add_releases(releases);

    info_with_fields!("setup", step = "assets tags");
    let assets_tags = AssetTag::all().into_iter().map(storage::models::TagRow::from_primitive).collect::<Vec<_>>();
    let _ = database.tag()?.add_tags(assets_tags);

    info_with_fields!("setup", step = "prices providers");
    let providers = PriceProvider::all()
        .into_iter()
        .map(|p| storage::models::PriceProviderConfigRow::new(p, p == PriceProvider::primary()))
        .collect::<Vec<_>>();
    let _ = database.prices_providers()?.add_prices_providers(providers);

    info_with_fields!("setup", step = "config");
    let configs: Vec<ConfigRow> = ConfigKey::all().into_iter().map(ConfigRow::from_primitive).collect();
    let _ = database.client()?.add_config(configs);

    info_with_fields!("setup", step = "param config");
    let param_configs: Vec<ConfigRow> = ConfigParamKey::all().into_iter().map(ConfigRow::from_param).collect();
    let _ = database.client()?.add_config(param_configs);

    info_with_fields!("setup", step = "cleanup stale config keys");
    let valid: HashSet<String> = ConfigKey::all()
        .into_iter()
        .map(|k| k.as_ref().to_string())
        .chain(ConfigParamKey::all().into_iter().map(|k| k.key()))
        .collect();
    let stale: Vec<String> = database.client()?.get_config_keys()?.into_iter().filter(|k| !valid.contains(k)).collect();
    if !stale.is_empty() {
        info_with_fields!("setup", step = "delete stale config keys", count = stale.len(), keys = format!("{:?}", stale));
        let _ = database.client()?.delete_keys(stale);
    }

    Ok(())
}

async fn setup_search_index(settings: &Settings) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info_with_fields!(
        "setup",
        step = "search index",
        indexes = format!("{:?}", INDEX_CONFIGS.iter().map(|c| c.name).collect::<Vec<_>>())
    );

    let search_index_client = SearchIndexClient::new(&settings.meilisearch.url, settings.meilisearch.key.as_str());
    search_index_client.setup(INDEX_CONFIGS, INDEX_PRIMARY_KEY).await.unwrap();

    Ok(())
}

async fn setup_queues(settings: &Settings) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info_with_fields!("setup", step = "queues");

    let chain_queues = QueueName::chain_queues();
    let non_chain_queues: Vec<_> = QueueName::all().into_iter().filter(|q| !chain_queues.contains(q)).collect();
    let exchanges = ExchangeName::all();
    let chains = Chain::all();

    let retry = streamer::Retry::new(settings.rabbitmq.retry.delay, settings.rabbitmq.retry.timeout);
    let rabbitmq_config = StreamProducerConfig::new(settings.rabbitmq.url.clone(), retry);
    let stream_producer = StreamProducer::new(&rabbitmq_config, "setup", streamer::no_shutdown()).await.unwrap();
    let _ = stream_producer.declare_queues(non_chain_queues).await;
    let _ = stream_producer.declare_exchanges(exchanges.clone()).await;

    info_with_fields!(
        "setup",
        step = "queue exchanges for chain-based consumers",
        queues = format!("{:?}", chain_queues.iter().map(|q| q.to_string()).collect::<Vec<_>>()),
        chains = format!("{:?}", chains)
    );

    for queue in &chain_queues {
        let exchange_name = format!("{}_exchange", queue);
        let _ = stream_producer.declare_exchange(&exchange_name, ExchangeKind::Topic).await;
        for chain in queue_supported_chains(queue, &chains) {
            let _ = stream_producer.bind_queue_routing_key(queue.clone(), chain.as_ref()).await;
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
            queues = format!("{:?}", exchange_queues.iter().map(|q| q.to_string()).collect::<Vec<_>>())
        );
        for queue in &exchange_queues {
            for chain in queue_supported_chains(queue, &chains) {
                let queue_name = format!("{}.{}", queue, chain.as_ref());
                let _ = stream_producer.bind_queue(&queue_name, &exchange.to_string(), chain.as_ref()).await;
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
