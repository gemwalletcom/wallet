use std::error::Error;
use std::sync::Arc;

use super::api_clients::{SETUP_DEV_API_CLIENT_NAME, SETUP_DEV_API_CLIENT_SECRET, api_client_access_grants};
use super::database::run_migrations;
use super::production::setup_database;
use crate::Services;
use crate::rewards::{create_username, username_rules};
use chrono::Utc;
use gem_tracing::info_with_fields;
use num_bigint::BigUint;
use primitives::currency::Currency;
use primitives::{
    Asset, AssetAssociation, AssetAssociationType, AssetId, AssetType, Chain, ChartTimeframe, Device, DeviceLocale, FiatAsset, FiatProviderCountry, FiatProviderName, FiatQuoteType, FiatRate, FiatRateProvider, FiatTransaction,
    FiatTransactionStatus, NotificationType, Platform, PlatformStore, PriceAlert, PriceAlertDirection, PriceData, PriceId, PriceProvider, WalletId, WalletSource, WalletType,
    asset_constants::{
        ARBITRUM_USDC_ASSET_ID, ARBITRUM_USDT_ASSET_ID, BASE_USDC_ASSET_ID, ETHEREUM_USDC_ASSET_ID, ETHEREUM_USDT_ASSET_ID, POLYGON_USDC_ASSET_ID, SMARTCHAIN_USDT_ASSET_ID, SOLANA_USDC_ASSET_ID, SOLANA_USDT_ASSET_ID, TON_DUST_ASSET_ID,
        TON_DUST_TOKEN_ID, TON_STON_ASSET_ID, TON_STON_TOKEN_ID, TON_USDT_ASSET_ID, TON_USDT_TOKEN_ID, TRON_USDT_ASSET_ID,
    },
    known_assets::{ARBITRUM_USDC, ARBITRUM_USDT, BASE_USDC, ETHEREUM_USDC, ETHEREUM_USDT, POLYGON_USDC, SMARTCHAIN_USDT, SOLANA_USDC, SOLANA_USDT, TRON_USDT},
};
use rewards::UsernameRules;
use settings::Settings;
use storage::{
    ApiClientsRepository, AssetsRepository, ChartPoint, ChartsRepository, DatabaseClient, DevicesRepository, FiatRepository, NewNotification, NewWallet, NotificationsRepository, PriceAlertsRepository, PriceAsset, PricesRepository,
    WalletsRepository,
};

const DEV_USERNAME: &str = "gemcoder";

pub async fn run_setup_dev(settings: Settings) -> Result<(), Box<dyn Error + Send + Sync>> {
    info_with_fields!("setup_dev", step = "init");

    let services = Services::new(Arc::new(settings.clone()))?;
    let database = services.database();
    run_migrations(&database, "setup_dev").await?;
    setup_database(&database).await?;
    let username_rules = username_rules(&services.config()).await?;

    database
        .run(move |client| -> Result<_, Box<dyn Error + Send + Sync>> {
            setup_dev_currency(client)?;
            setup_dev_api_clients(client)?;
            setup_dev_devices(client, &username_rules)?;
            setup_dev_assets(client)
        })
        .await?;

    info_with_fields!("setup_dev", step = "complete");
    Ok(())
}

fn setup_dev_currency(client: &mut DatabaseClient) -> Result<(), Box<dyn Error + Send + Sync>> {
    info_with_fields!("setup_dev", step = "add currency");

    info_with_fields!("setup_dev", step = "add rate", currency = "USD");
    client.set_fiat_rates(FiatRateProvider::Coingecko, vec![FiatRate { symbol: Currency::USD, rate: 1.0 }])?;
    client.set_fiat_rates_enabled(vec![Currency::USD], true)?;

    Ok(())
}

fn setup_dev_api_clients(client: &mut DatabaseClient) -> Result<(), Box<dyn Error + Send + Sync>> {
    info_with_fields!("setup_dev", step = "api clients");

    client.add_api_client_grants(api_client_access_grants(SETUP_DEV_API_CLIENT_NAME))?;
    client.set_api_client_secret(SETUP_DEV_API_CLIENT_NAME, SETUP_DEV_API_CLIENT_SECRET)?;

    Ok(())
}

fn setup_dev_devices(client: &mut DatabaseClient, username_rules: &UsernameRules) -> Result<(), Box<dyn Error + Send + Sync>> {
    info_with_fields!("setup_dev", step = "add devices");

    let ios_device_id = "0".repeat(64);
    let android_device_id = "1".repeat(64);

    let ios_device = Device {
        id: ios_device_id.clone(),
        platform: Platform::IOS,
        platform_store: PlatformStore::AppStore,
        token: "test_token".to_string(),
        locale: DeviceLocale::EN,
        currency: Currency::USD,
        is_push_enabled: true,
        is_price_alerts_enabled: Some(true),
        version: "1.0.0".to_string(),
        subscriptions_version: 1,
        os: "iOS 18".to_string(),
        model: "iPhone 16".to_string(),
    };

    let android_device = Device {
        id: android_device_id.clone(),
        platform: Platform::Android,
        platform_store: PlatformStore::GooglePlay,
        token: "test_token_android".to_string(),
        locale: DeviceLocale::EN,
        currency: Currency::USD,
        is_push_enabled: true,
        is_price_alerts_enabled: Some(true),
        version: "1.0.0".to_string(),
        subscriptions_version: 1,
        os: "Android 15".to_string(),
        model: "Pixel 9".to_string(),
    };

    for (device_id, device) in [(ios_device_id.as_str(), ios_device), (android_device_id.as_str(), android_device)] {
        client.add_device(device)?;
        info_with_fields!("setup_dev", step = "device added", device_id = device_id);
    }

    let ios_device_row_id = client.get_device_row_id(&ios_device_id)?;
    let android_device_row_id = client.get_device_row_id(&android_device_id)?;

    let wallet_address = "0xBA4D1d35bCe0e8F28E5a3403e7a0b996c5d50AC4";

    info_with_fields!("setup_dev", step = "add wallet");
    let new_wallet = NewWallet {
        wallet_id: WalletId::Multicoin(wallet_address.to_string()),
        wallet_type: WalletType::Multicoin,
        source: WalletSource::Create,
    };
    let wallet = client.get_or_create_wallet(new_wallet)?;
    info_with_fields!("setup_dev", step = "wallet added", wallet_id = wallet.id);

    info_with_fields!("setup_dev", step = "add wallet subscriptions");
    let solana_address = "8wytzyCBXco7yqgrLDiecpEt452MSuNWRe7xsLgAAX1H";
    let subscriptions = vec![
        (wallet.id, Chain::Ethereum, wallet_address.to_string()),
        (wallet.id, Chain::HyperCore, wallet_address.to_string()),
        (wallet.id, Chain::Solana, solana_address.to_string()),
    ];

    let result = client.add_subscriptions(ios_device_row_id, subscriptions.clone())?;
    info_with_fields!("setup_dev", step = "ios wallet subscription added", count = result);

    let result = client.add_subscriptions(android_device_row_id, subscriptions)?;
    info_with_fields!("setup_dev", step = "android wallet subscription added", count = result);

    setup_dev_fiat_transactions(client, ios_device_row_id, wallet.id)?;

    info_with_fields!("setup_dev", step = "add rewards");
    let devices = client.get_devices_by_wallet_id(wallet.id)?;
    if !devices.is_empty() {
        let result: Result<_, Box<dyn Error + Send + Sync>> = create_username(client, wallet.id, DEV_USERNAME, username_rules).map_err(Into::into).and_then(|outcome| outcome.map_err(Into::into));
        match result {
            Ok((rewards, _)) => info_with_fields!("setup_dev", step = "rewards added", code = rewards.code.unwrap_or_default(), points = rewards.points),
            Err(e) => info_with_fields!("setup_dev", step = "rewards skipped (may already exist)", error = e.to_string()),
        }
    }

    info_with_fields!("setup_dev", step = "add notifications");
    let notifications = vec![
        NewNotification {
            wallet_id: wallet.id,
            asset_id: None,
            notification_type: NotificationType::RewardsEnabled,
            metadata: None,
        },
        NewNotification {
            wallet_id: wallet.id,
            asset_id: None,
            notification_type: NotificationType::ReferralJoined,
            metadata: Some(serde_json::json!({"username": "alice", "points": 100})),
        },
        NewNotification {
            wallet_id: wallet.id,
            asset_id: None,
            notification_type: NotificationType::RewardsCodeDisabled,
            metadata: None,
        },
        NewNotification {
            wallet_id: wallet.id,
            asset_id: None,
            notification_type: NotificationType::RewardsCreateUsername,
            metadata: Some(serde_json::json!({"points": 50})),
        },
        NewNotification {
            wallet_id: wallet.id,
            asset_id: None,
            notification_type: NotificationType::RewardsInvite,
            metadata: Some(serde_json::json!({"username": "bob", "points": 200})),
        },
    ];
    let result = client.create_notifications(notifications)?;
    info_with_fields!("setup_dev", step = "notifications added", count = result);

    info_with_fields!("setup_dev", step = "add price alerts");
    let price_alerts = vec![
        PriceAlert::new_price(AssetId::from_chain(Chain::Ethereum), Currency::USD, 3000.0, PriceAlertDirection::Up),
        PriceAlert::new_price(AssetId::from_chain(Chain::Bitcoin), Currency::USD, 50000.0, PriceAlertDirection::Down),
    ];
    let result = client.add_price_alerts(&ios_device_id, price_alerts)?;
    info_with_fields!("setup_dev", step = "price alerts added", count = result);

    Ok(())
}

fn setup_dev_fiat_transactions(client: &mut DatabaseClient, device_id: i32, wallet_id: i32) -> Result<(), Box<dyn Error + Send + Sync>> {
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
            value: BigUint::from(75000000000000000u64),
            transaction_hash: None,
            created_at: now,
            updated_at: now,
        }
    };

    let transactions = [
        FiatTransaction { provider_transaction_id: None, ..mock() },
        FiatTransaction {
            id: "setup-dev-quote-mercuryo-complete".to_string(),
            provider: FiatProviderName::Mercuryo,
            provider_transaction_id: Some("setup-dev-mercuryo-complete".to_string()),
            status: FiatTransactionStatus::Complete,
            fiat_amount: 320.5,
            value: BigUint::from(160000000000000000u64),
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
            value: BigUint::from(500000000u64),
            ..mock()
        },
    ];

    let evm_address_id = client.subscriptions_wallet_address_for_chain(device_id, wallet_id, Chain::Ethereum)?.id;
    let solana_address_id = client.subscriptions_wallet_address_for_chain(device_id, wallet_id, Chain::Solana)?.id;

    let address_ids = [evm_address_id, evm_address_id, solana_address_id];

    let mut count = 0;
    for (transaction, address_id) in transactions.into_iter().zip(address_ids) {
        count += client.add_fiat_transaction(transaction, device_id, wallet_id, address_id)?;
    }

    info_with_fields!("setup_dev", step = "fiat transactions added", count = count);
    Ok(())
}

fn setup_dev_assets(client: &mut DatabaseClient) -> Result<(), Box<dyn Error + Send + Sync>> {
    info_with_fields!("setup_dev", step = "add assets");

    let assets = [
        Asset::new(TON_USDT_ASSET_ID.clone(), "Tether USD".to_string(), "USD₮".to_string(), 6, AssetType::JETTON),
        Asset::new(TON_STON_ASSET_ID.clone(), "STON".to_string(), "STON".to_string(), 9, AssetType::JETTON),
        Asset::new(TON_DUST_ASSET_ID.clone(), "DeDust".to_string(), "DUST".to_string(), 9, AssetType::JETTON),
        (*ETHEREUM_USDC).clone(),
        (*ARBITRUM_USDC).clone(),
        (*BASE_USDC).clone(),
        (*POLYGON_USDC).clone(),
        (*SOLANA_USDC).clone(),
        (*ETHEREUM_USDT).clone(),
        (*ARBITRUM_USDT).clone(),
        (*SMARTCHAIN_USDT).clone(),
        (*SOLANA_USDT).clone(),
        (*TRON_USDT).clone(),
    ]
    .into_iter()
    .map(|asset| asset.as_basic_primitive())
    .collect::<Vec<_>>();
    client.add_assets(assets)?;

    setup_dev_asset_associations(
        client,
        "usdc",
        &[
            (ETHEREUM_USDC_ASSET_ID.clone(), AssetAssociationType::Official),
            (ARBITRUM_USDC_ASSET_ID.clone(), AssetAssociationType::Official),
            (BASE_USDC_ASSET_ID.clone(), AssetAssociationType::Official),
            (POLYGON_USDC_ASSET_ID.clone(), AssetAssociationType::Official),
            (SOLANA_USDC_ASSET_ID.clone(), AssetAssociationType::Official),
        ],
    )?;
    setup_dev_asset_associations(
        client,
        "usdt",
        &[
            (ETHEREUM_USDT_ASSET_ID.clone(), AssetAssociationType::Official),
            (TRON_USDT_ASSET_ID.clone(), AssetAssociationType::Official),
            (SOLANA_USDT_ASSET_ID.clone(), AssetAssociationType::Official),
            (SMARTCHAIN_USDT_ASSET_ID.clone(), AssetAssociationType::Official),
            (ARBITRUM_USDT_ASSET_ID.clone(), AssetAssociationType::Bridged),
        ],
    )?;

    info_with_fields!("setup_dev", step = "add fiat assets");

    let bitcoin_asset_id = AssetId::from_chain(Chain::Bitcoin);
    let ethereum_asset_id = AssetId::from_chain(Chain::Ethereum);
    let smartchain_asset_id = AssetId::from_chain(Chain::SmartChain);

    let fiat_asset = |provider: FiatProviderName, code: &str, symbol: &str, network: &str, asset_id: &AssetId| FiatAsset {
        id: code.to_string(),
        asset_id: Some(asset_id.clone()),
        provider,
        symbol: symbol.to_string(),
        network: Some(network.to_string()),
        token_id: None,
        enabled: true,
        is_buy_enabled: true,
        is_sell_enabled: true,
        unsupported_countries: Default::default(),
        buy_limits: vec![],
        sell_limits: vec![],
    };

    let fiat_assets = vec![
        fiat_asset(FiatProviderName::MoonPay, "eth", "ETH", "ethereum", &ethereum_asset_id),
        fiat_asset(FiatProviderName::Mercuryo, "ETH", "ETH", "ETHEREUM", &ethereum_asset_id),
        fiat_asset(FiatProviderName::MoonPay, "bnb_bsc", "BNB", "binance_smart_chain", &smartchain_asset_id),
        fiat_asset(FiatProviderName::Mercuryo, "BNB", "BNB", "BINANCESMARTCHAIN", &smartchain_asset_id),
        fiat_asset(FiatProviderName::Paybis, "ETH", "ETH", "ethereum", &ethereum_asset_id),
    ];

    let result = client.add_fiat_assets(fiat_assets)?;
    info_with_fields!("setup_dev", step = "fiat assets added", count = result);

    info_with_fields!("setup_dev", step = "add fiat provider countries");

    let fiat_countries: Vec<FiatProviderCountry> = FiatProviderName::all()
        .into_iter()
        .map(|provider| FiatProviderCountry {
            provider,
            alpha2: "US".to_string(),
            is_allowed: true,
        })
        .collect();

    let result = client.add_fiat_providers_countries(fiat_countries)?;
    info_with_fields!("setup_dev", step = "fiat provider countries added", count = result);

    info_with_fields!("setup_dev", step = "add prices and charts");
    let now = chrono::Utc::now().naive_utc();
    let ton_native_price_id = Asset::from_chain(Chain::Ton).symbol.to_lowercase();
    let coins = [
        (PriceProvider::primary(), Chain::Bitcoin.as_ref(), bitcoin_asset_id, 60000.0),
        (PriceProvider::primary(), Chain::Ethereum.as_ref(), ethereum_asset_id, 2000.0),
        (PriceProvider::TonApi, ton_native_price_id.as_str(), AssetId::from_chain(Chain::Ton), 1.42),
        (PriceProvider::TonApi, TON_USDT_TOKEN_ID, TON_USDT_ASSET_ID.clone(), 1.0),
        (PriceProvider::TonApi, TON_STON_TOKEN_ID, TON_STON_ASSET_ID.clone(), 0.47),
        (PriceProvider::TonApi, TON_DUST_TOKEN_ID, TON_DUST_ASSET_ID.clone(), 0.64),
    ];

    let prices: Vec<PriceData> = coins
        .iter()
        .map(|(provider, coin_id, _, base_price)| PriceData {
            id: PriceId::new(*provider, coin_id.to_string()),
            provider: *provider,
            provider_price_id: coin_id.to_string(),
            price: *base_price,
            price_change_percentage_24h: 0.0,
            all_time_high: 0.0,
            all_time_high_date: None,
            all_time_low: 0.0,
            all_time_low_date: None,
            market_cap_rank: None,
            total_volume: None,
            last_updated_at: Utc::now(),
        })
        .collect();

    let price_assets: Vec<PriceAsset> = coins
        .iter()
        .map(|(provider, coin_id, asset_id, _)| PriceAsset {
            asset_id: asset_id.clone(),
            price_id: PriceId::new(*provider, coin_id.to_string()),
        })
        .collect();

    let result = client.add_prices(prices)?;
    info_with_fields!("setup_dev", step = "prices added", count = result);

    let result = client.set_prices_assets(price_assets)?;
    info_with_fields!("setup_dev", step = "prices_assets added", count = result);

    for (idx, (provider, coin_id, _, base_price)) in coins.iter().enumerate() {
        let seed = (idx + 1) as f64;
        let gen_price = |i: f64, scale: f64| (base_price + ((i * 0.3 + seed * 7.0).sin() + (i * 0.07).cos()) * base_price * scale).max(base_price * 0.1);
        let price_id = PriceId::id_for(*provider, coin_id);

        let point = |price: f64, created_at| ChartPoint {
            price_id: price_id.clone(),
            price,
            created_at,
        };
        let hourly: Vec<ChartPoint> = (0i64..720).map(|h| point(gen_price(h as f64, 0.1), now - chrono::Duration::hours(h))).collect();
        let daily: Vec<ChartPoint> = (30i64..1825).map(|d| point(gen_price(d as f64, 0.15), now - chrono::Duration::days(d))).collect();

        client.add_charts(ChartTimeframe::Hourly, hourly)?;
        client.add_charts(ChartTimeframe::Daily, daily)?;
    }
    info_with_fields!("setup_dev", step = "charts added");

    Ok(())
}

fn setup_dev_asset_associations(client: &mut DatabaseClient, id: &str, assets: &[(AssetId, AssetAssociationType)]) -> Result<(), Box<dyn Error + Send + Sync>> {
    let associations = assets
        .iter()
        .map(|(asset_id, association_type)| AssetAssociation {
            asset_id: asset_id.clone(),
            association_type: association_type.clone(),
        })
        .collect();

    client.upsert_asset_associations(id, associations)?;
    Ok(())
}
