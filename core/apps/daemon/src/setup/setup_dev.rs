use super::api_clients::{SETUP_DEV_API_CLIENT_NAME, SETUP_DEV_API_CLIENT_SECRET, api_client_access_grants};
use super::database::run_migrations;
use chrono::Utc;
use gem_tracing::info_with_fields;
use primitives::{
    Asset, AssetId, Chain, ChartTimeframe, FiatProviderName, FiatQuoteType, FiatTransaction, FiatTransactionStatus, NotificationType, PriceAlert, PriceAlertDirection, PriceId,
    PriceProvider,
};
use settings::Settings;
use storage::models::{ChartRow, FiatAssetRow, FiatProviderCountryRow, FiatRateRow, NewFiatTransactionRow, PriceAssetRow, UpdateDeviceRow, price::NewPriceRow};
use storage::sql_types::{Platform, PlatformStore};
use storage::{
    ApiClientsRepository, AssetsRepository, ChartsRepository, Database, DevicesRepository, NewNotificationRow, NewWalletRow, NotificationsRepository, PriceAlertsRepository,
    PricesRepository, RewardsRepository, WalletSource, WalletType, WalletsRepository,
};

pub async fn run_setup_dev(settings: Settings) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info_with_fields!("setup_dev", step = "init");

    let database = Database::new(&settings.postgres.url, settings.postgres.pool);
    run_migrations(&database, "setup_dev")?;

    setup_dev_currency(&database)?;
    setup_dev_api_clients(&database)?;
    setup_dev_devices(&database)?;
    setup_dev_assets(&database)?;

    info_with_fields!("setup_dev", step = "complete");
    Ok(())
}

fn setup_dev_currency(database: &Database) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info_with_fields!("setup_dev", step = "add currency");

    let fiat_rate = FiatRateRow {
        id: "USD".to_string(),
        name: "US Dollar".to_string(),
        rate: 1.0,
    };

    info_with_fields!("setup_dev", step = "add rate", currency = "USD");
    database.fiat()?.set_fiat_rates(vec![fiat_rate])?;

    Ok(())
}

fn setup_dev_api_clients(database: &Database) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info_with_fields!("setup_dev", step = "api clients");

    let mut api_clients = database.api_clients()?;
    api_clients.add_api_client_grants(api_client_access_grants(SETUP_DEV_API_CLIENT_NAME))?;
    api_clients.set_api_client_secret(SETUP_DEV_API_CLIENT_NAME, SETUP_DEV_API_CLIENT_SECRET)?;

    Ok(())
}

fn setup_dev_devices(database: &Database) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info_with_fields!("setup_dev", step = "add devices");

    let ios_device_id = "0".repeat(64);
    let android_device_id = "1".repeat(64);

    let ios_device = UpdateDeviceRow {
        device_id: ios_device_id.clone(),
        platform: Platform::IOS,
        platform_store: PlatformStore::AppStore,
        token: "test_token".to_string(),
        locale: "en".to_string(),
        currency: "USD".to_string(),
        is_push_enabled: true,
        is_price_alerts_enabled: true,
        version: "1.0.0".to_string(),
        subscriptions_version: 1,
        os: "iOS 18".to_string(),
        model: "iPhone 16".to_string(),
    };

    let android_device = UpdateDeviceRow {
        device_id: android_device_id.clone(),
        platform: Platform::Android,
        platform_store: PlatformStore::GooglePlay,
        token: "test_token_android".to_string(),
        locale: "en".to_string(),
        currency: "USD".to_string(),
        is_push_enabled: true,
        is_price_alerts_enabled: true,
        version: "1.0.0".to_string(),
        subscriptions_version: 1,
        os: "Android 15".to_string(),
        model: "Pixel 9".to_string(),
    };

    for (device_id, device) in [(ios_device_id.as_str(), ios_device), (android_device_id.as_str(), android_device)] {
        database.devices()?.add_device(device)?;
        info_with_fields!("setup_dev", step = "device added", device_id = device_id);
    }

    let ios_device_row_id = database.devices()?.get_device_row_id(&ios_device_id)?;
    let android_device_row_id = database.devices()?.get_device_row_id(&android_device_id)?;

    let wallet_address = "0xBA4D1d35bCe0e8F28E5a3403e7a0b996c5d50AC4";

    info_with_fields!("setup_dev", step = "add wallet");
    let wallet_identifier = format!("multicoin_{}", wallet_address);
    let new_wallet = NewWalletRow {
        identifier: wallet_identifier,
        wallet_type: WalletType::Multicoin,
        source: WalletSource::Create,
    };
    let wallet = database.wallets()?.get_or_create_wallet(new_wallet)?;
    info_with_fields!("setup_dev", step = "wallet added", wallet_id = wallet.id);

    info_with_fields!("setup_dev", step = "add wallet subscriptions");
    let solana_address = "8wytzyCBXco7yqgrLDiecpEt452MSuNWRe7xsLgAAX1H";
    let subscriptions = vec![
        (wallet.id, Chain::Ethereum, wallet_address.to_string()),
        (wallet.id, Chain::HyperCore, wallet_address.to_string()),
        (wallet.id, Chain::Solana, solana_address.to_string()),
    ];

    let result = WalletsRepository::add_subscriptions(&mut database.wallets()?, ios_device_row_id, subscriptions.clone())?;
    info_with_fields!("setup_dev", step = "ios wallet subscription added", count = result);

    let result = WalletsRepository::add_subscriptions(&mut database.wallets()?, android_device_row_id, subscriptions)?;
    info_with_fields!("setup_dev", step = "android wallet subscription added", count = result);

    setup_dev_fiat_transactions(database, ios_device_row_id, wallet.id)?;

    info_with_fields!("setup_dev", step = "add rewards");
    let devices = database.wallets()?.get_devices_by_wallet_id(wallet.id)?;
    if !devices.is_empty() {
        let result = database.rewards()?.create_reward(wallet.id, "gemcoder");
        match result {
            Ok((rewards, _)) => info_with_fields!("setup_dev", step = "rewards added", code = rewards.code.unwrap_or_default(), points = rewards.points),
            Err(e) => info_with_fields!("setup_dev", step = "rewards skipped (may already exist)", error = e.to_string()),
        }
    }

    info_with_fields!("setup_dev", step = "add notifications");
    let notifications = vec![
        NewNotificationRow {
            wallet_id: wallet.id,
            asset_id: None,
            notification_type: NotificationType::RewardsEnabled.into(),
            metadata: None,
        },
        NewNotificationRow {
            wallet_id: wallet.id,
            asset_id: None,
            notification_type: NotificationType::ReferralJoined.into(),
            metadata: Some(serde_json::json!({"username": "alice", "points": 100})),
        },
        NewNotificationRow {
            wallet_id: wallet.id,
            asset_id: None,
            notification_type: NotificationType::RewardsCodeDisabled.into(),
            metadata: None,
        },
        NewNotificationRow {
            wallet_id: wallet.id,
            asset_id: None,
            notification_type: NotificationType::RewardsCreateUsername.into(),
            metadata: Some(serde_json::json!({"points": 50})),
        },
        NewNotificationRow {
            wallet_id: wallet.id,
            asset_id: None,
            notification_type: NotificationType::RewardsInvite.into(),
            metadata: Some(serde_json::json!({"username": "bob", "points": 200})),
        },
    ];
    let result = database.notifications()?.create_notifications(notifications)?;
    info_with_fields!("setup_dev", step = "notifications added", count = result);

    info_with_fields!("setup_dev", step = "add price alerts");
    let price_alerts = vec![
        PriceAlert::new_price(AssetId::from_chain(Chain::Ethereum), "USD".to_string(), 3000.0, PriceAlertDirection::Up),
        PriceAlert::new_price(AssetId::from_chain(Chain::Bitcoin), "USD".to_string(), 50000.0, PriceAlertDirection::Down),
    ];
    let result = database.price_alerts()?.add_price_alerts(&ios_device_id, price_alerts)?;
    info_with_fields!("setup_dev", step = "price alerts added", count = result);

    Ok(())
}

fn setup_dev_fiat_transactions(database: &Database, device_id: i32, wallet_id: i32) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info_with_fields!("setup_dev", step = "add fiat transactions");

    let mock = || {
        let now = Utc::now();

        FiatTransaction {
            id: "setup-dev-quote-moonpay-pending".to_string(),
            asset_id: AssetId::from_chain(Chain::Ethereum),
            transaction_type: FiatQuoteType::Buy,
            provider: FiatProviderName::MoonPay,
            provider_transaction_id: None,
            status: FiatTransactionStatus::Pending,
            country: Some("US".to_string()),
            fiat_amount: 150.0,
            fiat_currency: "USD".to_string(),
            value: "75000000000000000".to_string(),
            transaction_hash: None,
            created_at: now,
            updated_at: now,
        }
    };

    let transactions = [
        FiatTransaction {
            provider_transaction_id: None,
            ..mock()
        },
        FiatTransaction {
            id: "setup-dev-quote-mercuryo-complete".to_string(),
            provider: FiatProviderName::Mercuryo,
            provider_transaction_id: Some("setup-dev-mercuryo-complete".to_string()),
            status: FiatTransactionStatus::Complete,
            fiat_amount: 320.5,
            value: "160000000000000000".to_string(),
            transaction_hash: Some("0xsetupdevcomplete".to_string()),
            ..mock()
        },
        FiatTransaction {
            id: "setup-dev-quote-transak-failed".to_string(),
            asset_id: AssetId::from_chain(Chain::Solana),
            transaction_type: FiatQuoteType::Sell,
            provider: FiatProviderName::Transak,
            provider_transaction_id: Some("setup-dev-transak-failed".to_string()),
            status: FiatTransactionStatus::Failed,
            fiat_amount: 95.25,
            value: "500000000".to_string(),
            ..mock()
        },
    ];

    let evm_address_id = database.wallets()?.subscriptions_wallet_address_for_chain(device_id, wallet_id, Chain::Ethereum)?.id;
    let solana_address_id = database.wallets()?.subscriptions_wallet_address_for_chain(device_id, wallet_id, Chain::Solana)?.id;

    let mut fiat = database.fiat()?;
    let transaction_rows = vec![
        NewFiatTransactionRow::new(transactions[0].clone(), device_id, wallet_id, evm_address_id),
        NewFiatTransactionRow::new(transactions[1].clone(), device_id, wallet_id, evm_address_id),
        NewFiatTransactionRow::new(transactions[2].clone(), device_id, wallet_id, solana_address_id),
    ];

    let mut count = 0;
    for row in transaction_rows {
        count += fiat.add_fiat_transaction(row)?;
    }

    info_with_fields!("setup_dev", step = "fiat transactions added", count = count);
    Ok(())
}

fn setup_dev_assets(database: &Database) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info_with_fields!("setup_dev", step = "add assets");

    let assets = Chain::all().into_iter().map(|x| Asset::from_chain(x).as_basic_primitive()).collect::<Vec<_>>();
    let _ = database.assets()?.add_assets(assets);

    info_with_fields!("setup_dev", step = "add fiat assets");

    let bitcoin_asset_id = AssetId::from_chain(Chain::Bitcoin);
    let ethereum_asset_id = AssetId::from_chain(Chain::Ethereum);
    let smartchain_asset_id = AssetId::from_chain(Chain::SmartChain);

    let fiat_asset = |provider: FiatProviderName, code: &str, symbol: &str, network: &str, asset_id: &AssetId| FiatAssetRow {
        id: format!("{}_{}", provider.id(), code).to_lowercase(),
        asset_id: Some(asset_id.into()),
        provider: provider.into(),
        code: code.to_string(),
        symbol: symbol.to_string(),
        network: Some(network.to_string()),
        token_id: None,
        is_enabled: true,
        is_enabled_by_provider: true,
        is_buy_enabled: true,
        is_sell_enabled: true,
        buy_limits: None,
        sell_limits: None,
        unsupported_countries: None,
    };

    let fiat_assets = vec![
        fiat_asset(FiatProviderName::MoonPay, "eth", "ETH", "ethereum", &ethereum_asset_id),
        fiat_asset(FiatProviderName::Mercuryo, "ETH", "ETH", "ETHEREUM", &ethereum_asset_id),
        fiat_asset(FiatProviderName::MoonPay, "bnb_bsc", "BNB", "binance_smart_chain", &smartchain_asset_id),
        fiat_asset(FiatProviderName::Mercuryo, "BNB", "BNB", "BINANCESMARTCHAIN", &smartchain_asset_id),
        fiat_asset(FiatProviderName::Paybis, "ETH", "ETH", "ethereum", &ethereum_asset_id),
    ];

    let result = database.fiat()?.add_fiat_assets(fiat_assets)?;
    info_with_fields!("setup_dev", step = "fiat assets added", count = result);

    info_with_fields!("setup_dev", step = "add fiat provider countries");

    let fiat_countries: Vec<FiatProviderCountryRow> = FiatProviderName::all()
        .into_iter()
        .map(|provider| {
            let id = provider.id();
            FiatProviderCountryRow {
                id: format!("{}_us", id),
                provider: provider.into(),
                alpha2: "US".to_string(),
                is_allowed: true,
            }
        })
        .collect();

    let result = database.fiat()?.add_fiat_providers_countries(fiat_countries)?;
    info_with_fields!("setup_dev", step = "fiat provider countries added", count = result);

    info_with_fields!("setup_dev", step = "add prices and charts");
    let now = chrono::Utc::now().naive_utc();
    let coins: Vec<(&str, &AssetId, f64)> = vec![
        (Chain::Bitcoin.as_ref(), &bitcoin_asset_id, 60000.0),
        (Chain::Ethereum.as_ref(), &ethereum_asset_id, 2000.0),
    ];

    let prices: Vec<NewPriceRow> = coins
        .iter()
        .map(|(coin_id, _, base_price)| NewPriceRow::with_market_data(PriceProvider::primary(), coin_id.to_string(), None, Some(*base_price), None))
        .collect();

    let price_assets: Vec<PriceAssetRow> = coins
        .iter()
        .map(|(coin_id, asset_id, _)| PriceAssetRow::new((*asset_id).clone(), PriceProvider::primary(), coin_id))
        .collect();

    let result = database.prices()?.add_prices(prices)?;
    info_with_fields!("setup_dev", step = "prices added", count = result);

    let result = database.prices()?.set_prices_assets(price_assets)?;
    info_with_fields!("setup_dev", step = "prices_assets added", count = result);

    for (idx, (coin_id, _, base_price)) in coins.iter().enumerate() {
        let seed = (idx + 1) as f64;
        let gen_price = |i: f64, scale: f64| (base_price + ((i * 0.3 + seed * 7.0).sin() + (i * 0.07).cos()) * base_price * scale).max(base_price * 0.1);
        let price_id = PriceId::id_for(PriceProvider::primary(), coin_id);

        let hourly: Vec<ChartRow> = (0i64..720)
            .map(|h| ChartRow::new(price_id.clone(), gen_price(h as f64, 0.1), now - chrono::Duration::hours(h)))
            .collect();

        let daily: Vec<ChartRow> = (30i64..1825)
            .map(|d| ChartRow::new(price_id.clone(), gen_price(d as f64, 0.15), now - chrono::Duration::days(d)))
            .collect();

        database.charts()?.add_charts(ChartTimeframe::Hourly, hourly)?;
        database.charts()?.add_charts(ChartTimeframe::Daily, daily)?;
    }
    info_with_fields!("setup_dev", step = "charts added");

    Ok(())
}
