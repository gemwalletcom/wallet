use chrono::{NaiveDateTime, Utc};
use num_bigint::BigUint;
use primitives::currency::Currency;
use primitives::{
    Asset, AssetAssociation, AssetAssociationType, AssetBasic, AssetId, AssetType, Chain, Device, DeviceLocale, FiatAsset, FiatProviderCountry, FiatProviderName, FiatQuoteType, FiatTransaction, FiatTransactionStatus, NotificationType,
    Platform, PlatformStore, PriceAlert, PriceAlertDirection, PriceData, PriceId, PriceProvider, WalletId, WalletSource, WalletType,
    asset_constants::{
        ARBITRUM_USDC_ASSET_ID, ARBITRUM_USDT_ASSET_ID, BASE_USDC_ASSET_ID, ETHEREUM_USDC_ASSET_ID, ETHEREUM_USDT_ASSET_ID, POLYGON_USDC_ASSET_ID, SMARTCHAIN_USDT_ASSET_ID, SOLANA_USDC_ASSET_ID, SOLANA_USDT_ASSET_ID, TON_DUST_ASSET_ID,
        TON_DUST_TOKEN_ID, TON_STON_ASSET_ID, TON_STON_TOKEN_ID, TON_USDT_ASSET_ID, TON_USDT_TOKEN_ID, TRON_USDT_ASSET_ID,
    },
    known_assets::{ARBITRUM_USDC, ARBITRUM_USDT, BASE_USDC, ETHEREUM_USDC, ETHEREUM_USDT, POLYGON_USDC, SMARTCHAIN_USDT, SOLANA_USDC, SOLANA_USDT, TRON_USDT},
};
use storage::{ChartPoint, NewNotification, NewWallet, PriceAsset};

const WALLET_ADDRESS: &str = "0xBA4D1d35bCe0e8F28E5a3403e7a0b996c5d50AC4";
const SOLANA_ADDRESS: &str = "8wytzyCBXco7yqgrLDiecpEt452MSuNWRe7xsLgAAX1H";

pub(super) struct DevPriceSeries {
    pub(super) prices: Vec<PriceData>,
    pub(super) price_assets: Vec<PriceAsset>,
    pub(super) charts: Vec<(Vec<ChartPoint>, Vec<ChartPoint>)>,
}

pub(super) fn ios_device_id() -> String {
    "0".repeat(64)
}

pub(super) fn android_device_id() -> String {
    "1".repeat(64)
}

pub(super) fn devices() -> Vec<Device> {
    vec![
        Device {
            id: ios_device_id(),
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
        },
        Device {
            id: android_device_id(),
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
        },
    ]
}

pub(super) fn wallet() -> NewWallet {
    NewWallet {
        wallet_id: WalletId::Multicoin(WALLET_ADDRESS.to_string()),
        wallet_type: WalletType::Multicoin,
        source: WalletSource::Create,
    }
}

pub(super) fn subscriptions(wallet_id: i32) -> Vec<(i32, Chain, String)> {
    vec![
        (wallet_id, Chain::Ethereum, WALLET_ADDRESS.to_string()),
        (wallet_id, Chain::HyperCore, WALLET_ADDRESS.to_string()),
        (wallet_id, Chain::Solana, SOLANA_ADDRESS.to_string()),
    ]
}

pub(super) fn fiat_transactions() -> Vec<(FiatTransaction, Chain)> {
    let now = Utc::now();
    let pending = FiatTransaction {
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
    };
    vec![
        (pending.clone(), Chain::Ethereum),
        (
            FiatTransaction {
                id: "setup-dev-quote-mercuryo-complete".to_string(),
                provider: FiatProviderName::Mercuryo,
                provider_transaction_id: Some("setup-dev-mercuryo-complete".to_string()),
                status: FiatTransactionStatus::Complete,
                fiat_amount: 320.5,
                value: BigUint::from(160000000000000000u64),
                transaction_hash: Some("0xsetupdevcomplete".to_string()),
                ..pending.clone()
            },
            Chain::Ethereum,
        ),
        (
            FiatTransaction {
                id: "setup-dev-quote-transak-failed".to_string(),
                asset_id: AssetId::from_chain(Chain::Solana),
                transaction_type: FiatQuoteType::Sell,
                provider: FiatProviderName::Transak,
                provider_transaction_id: Some("setup-dev-transak-failed".to_string()),
                status: FiatTransactionStatus::Failed,
                fiat_amount: 95.25,
                value: BigUint::from(500000000u64),
                ..pending
            },
            Chain::Solana,
        ),
    ]
}

pub(super) fn notifications(wallet_id: i32) -> Vec<NewNotification> {
    [
        (NotificationType::RewardsEnabled, None),
        (NotificationType::ReferralJoined, Some(serde_json::json!({"username": "alice", "points": 100}))),
        (NotificationType::RewardsCodeDisabled, None),
        (NotificationType::RewardsCreateUsername, Some(serde_json::json!({"points": 50}))),
        (NotificationType::RewardsInvite, Some(serde_json::json!({"username": "bob", "points": 200}))),
    ]
    .into_iter()
    .map(|(notification_type, metadata)| NewNotification {
        wallet_id,
        asset_id: None,
        notification_type,
        metadata,
    })
    .collect()
}

pub(super) fn price_alerts() -> Vec<PriceAlert> {
    vec![
        PriceAlert::new_price(AssetId::from_chain(Chain::Ethereum), Currency::USD, 3000.0, PriceAlertDirection::Up),
        PriceAlert::new_price(AssetId::from_chain(Chain::Bitcoin), Currency::USD, 50000.0, PriceAlertDirection::Down),
    ]
}

pub(super) fn assets() -> Vec<AssetBasic> {
    [
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
    .collect()
}

pub(super) fn asset_associations() -> Vec<(&'static str, Vec<AssetAssociation>)> {
    [
        (
            "usdc",
            vec![
                (ETHEREUM_USDC_ASSET_ID.clone(), AssetAssociationType::Official),
                (ARBITRUM_USDC_ASSET_ID.clone(), AssetAssociationType::Official),
                (BASE_USDC_ASSET_ID.clone(), AssetAssociationType::Official),
                (POLYGON_USDC_ASSET_ID.clone(), AssetAssociationType::Official),
                (SOLANA_USDC_ASSET_ID.clone(), AssetAssociationType::Official),
            ],
        ),
        (
            "usdt",
            vec![
                (ETHEREUM_USDT_ASSET_ID.clone(), AssetAssociationType::Official),
                (TRON_USDT_ASSET_ID.clone(), AssetAssociationType::Official),
                (SOLANA_USDT_ASSET_ID.clone(), AssetAssociationType::Official),
                (SMARTCHAIN_USDT_ASSET_ID.clone(), AssetAssociationType::Official),
                (ARBITRUM_USDT_ASSET_ID.clone(), AssetAssociationType::Bridged),
            ],
        ),
    ]
    .into_iter()
    .map(|(id, assets)| (id, assets.into_iter().map(|(asset_id, association_type)| AssetAssociation { asset_id, association_type }).collect()))
    .collect()
}

pub(super) fn fiat_assets() -> Vec<FiatAsset> {
    let ethereum_asset_id = AssetId::from_chain(Chain::Ethereum);
    let smartchain_asset_id = AssetId::from_chain(Chain::SmartChain);
    [
        (FiatProviderName::MoonPay, "eth", "ETH", "ethereum", &ethereum_asset_id),
        (FiatProviderName::Mercuryo, "ETH", "ETH", "ETHEREUM", &ethereum_asset_id),
        (FiatProviderName::MoonPay, "bnb_bsc", "BNB", "binance_smart_chain", &smartchain_asset_id),
        (FiatProviderName::Mercuryo, "BNB", "BNB", "BINANCESMARTCHAIN", &smartchain_asset_id),
        (FiatProviderName::Paybis, "ETH", "ETH", "ethereum", &ethereum_asset_id),
    ]
    .into_iter()
    .map(|(provider, code, symbol, network, asset_id)| FiatAsset {
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
    })
    .collect()
}

pub(super) fn fiat_countries() -> Vec<FiatProviderCountry> {
    FiatProviderName::all()
        .into_iter()
        .map(|provider| FiatProviderCountry {
            provider,
            alpha2: "US".to_string(),
            is_allowed: true,
        })
        .collect()
}

pub(super) fn price_series(now: NaiveDateTime) -> DevPriceSeries {
    let ton_native_price_id = Asset::from_chain(Chain::Ton).symbol.to_lowercase();
    let coins = [
        (PriceProvider::primary(), Chain::Bitcoin.as_ref().to_string(), AssetId::from_chain(Chain::Bitcoin), 60000.0),
        (PriceProvider::primary(), Chain::Ethereum.as_ref().to_string(), AssetId::from_chain(Chain::Ethereum), 2000.0),
        (PriceProvider::TonApi, ton_native_price_id, AssetId::from_chain(Chain::Ton), 1.42),
        (PriceProvider::TonApi, TON_USDT_TOKEN_ID.to_string(), TON_USDT_ASSET_ID.clone(), 1.0),
        (PriceProvider::TonApi, TON_STON_TOKEN_ID.to_string(), TON_STON_ASSET_ID.clone(), 0.47),
        (PriceProvider::TonApi, TON_DUST_TOKEN_ID.to_string(), TON_DUST_ASSET_ID.clone(), 0.64),
    ];

    let prices = coins
        .iter()
        .map(|(provider, coin_id, _, base_price)| PriceData {
            id: PriceId::new(*provider, coin_id.clone()),
            provider: *provider,
            provider_price_id: coin_id.clone(),
            price: *base_price,
            price_change_percentage_24h: None,
            all_time_high: 0.0,
            all_time_high_date: None,
            all_time_low: 0.0,
            all_time_low_date: None,
            market_cap: None,
            market_cap_fdv: None,
            market_cap_rank: None,
            total_volume: None,
            circulating_supply: None,
            total_supply: None,
            max_supply: None,
            last_updated_at: Utc::now(),
        })
        .collect();

    let price_assets = coins
        .iter()
        .map(|(provider, coin_id, asset_id, _)| PriceAsset {
            asset_id: asset_id.clone(),
            price_id: PriceId::new(*provider, coin_id.clone()),
        })
        .collect();

    let charts = coins
        .iter()
        .enumerate()
        .map(|(idx, (provider, coin_id, _, base_price))| {
            let seed = (idx + 1) as f64;
            let gen_price = |i: f64, scale: f64| (base_price + ((i * 0.3 + seed * 7.0).sin() + (i * 0.07).cos()) * base_price * scale).max(base_price * 0.1);
            let price_id = PriceId::id_for(*provider, coin_id);
            let point = |price: f64, created_at| ChartPoint {
                price_id: price_id.clone(),
                price,
                created_at,
            };
            let hourly = (0i64..720).map(|h| point(gen_price(h as f64, 0.1), now - chrono::Duration::hours(h))).collect();
            let daily = (30i64..1825).map(|d| point(gen_price(d as f64, 0.15), now - chrono::Duration::days(d))).collect();
            (hourly, daily)
        })
        .collect();

    DevPriceSeries { prices, price_assets, charts }
}
