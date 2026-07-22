pub mod auth_config;
pub(crate) mod body;
pub mod client;
pub mod clients;
pub mod constants;
pub mod error;
pub mod guard;
pub mod signature;
use crate::assets::AssetsClient;
use crate::params::{
    AssetIdParam, ChainParam, ChartPeriodParam, CurrencyParam, FiatProviderIdParam, FiatQuoteTypeParam, NftAssetIdParam, QueryLimitParam, TransactionIdParam, UserAgent,
};
use crate::responders::{ApiError, ApiResponse};
use auth_config::AuthConfig;
use body::DeviceJson;
pub use client::DevicesClient;
pub use clients::{
    AddressNamesClient, FiatQuotesClient, NotificationsClient, PortfolioClient, RewardsClient, RewardsRedemptionClient, ScanClient, ScanProviderFactory, TransactionsClient,
    WalletConfigurationClient, WalletsClient,
};
use defi::DefiClient;
use gem_auth::AuthClient;
use guard::{AuthenticatedDevice, AuthenticatedDeviceWallet, VerifiedDeviceId};
use name_resolver::client::Client as NameClient;
use nft::NFTClient;
use primitives::DeviceToken;
use primitives::device::Device;
use primitives::name::NameRecord;
use primitives::nft::NFTAssetData;
use primitives::rewards::{RedemptionRequest, RedemptionResult, RewardRedemptionOption};
use primitives::{
    AddressName, AssetId, AuthNonce, ChainAddress, DefiPosition, FiatAssets, FiatQuoteRequest, FiatQuoteUrl, FiatQuotes, InAppNotification, NFTData, PortfolioAssets,
    PortfolioAssetsRequest, PriceAlerts, ReportNft, RewardEvent, Rewards, ScanTransaction, ScanTransactionPayload, Transaction, TransactionsResponse, WalletConfigurationResult,
    WalletId, WalletSubscription, WalletSubscriptionChains,
};
use rocket::{FromForm, State, delete, get, post, put};
use std::sync::Arc;
use streamer::{StreamProducer, StreamProducerQueue};

use crate::auth::WalletSigned;

#[derive(FromForm)]
pub struct DeviceTransactionsParams {
    asset_id: Option<AssetIdParam>,
    from_timestamp: Option<u64>,
    limit: QueryLimitParam,
}

#[derive(FromForm)]
pub struct DeviceNotificationsParams {
    from_timestamp: Option<u64>,
    limit: QueryLimitParam,
}

#[post("/devices", format = "json", data = "<device>")]
pub async fn add_device_v2(device_id: VerifiedDeviceId, device: DeviceJson<Device>, client: &State<DevicesClient>) -> Result<ApiResponse<Device>, ApiError> {
    let device = device.into_inner();
    if device.id != device_id.0 {
        return Err(ApiError::BadRequest("Device id mismatch".to_string()));
    }
    Ok(client.add_device(device)?.into())
}

#[get("/devices")]
pub async fn get_device_v2(device: AuthenticatedDevice, client: &State<DevicesClient>) -> Result<ApiResponse<Device>, ApiError> {
    Ok(client.get_device(&device.device_row.device_id)?.into())
}

#[get("/devices/is_registered")]
pub async fn is_device_registered_v2(device_id: VerifiedDeviceId, client: &State<DevicesClient>) -> Result<ApiResponse<bool>, ApiError> {
    Ok(client.is_device_registered(&device_id.0)?.into())
}

#[get("/devices/assets?<from_timestamp>")]
pub async fn get_device_assets_v2(device: AuthenticatedDeviceWallet, from_timestamp: Option<u64>, client: &State<AssetsClient>) -> Result<ApiResponse<Vec<AssetId>>, ApiError> {
    Ok(client.get_assets_by_wallet_id(device.device_row.id, device.wallet_id, from_timestamp)?.into())
}

#[get("/devices/transactions?<params..>")]
pub async fn get_device_transactions_v2(
    device: AuthenticatedDeviceWallet,
    params: DeviceTransactionsParams,
    client: &State<TransactionsClient>,
) -> Result<ApiResponse<TransactionsResponse>, ApiError> {
    Ok(client
        .get_transactions_by_wallet_id(
            &device.device_row.device_id,
            device.device_row.id,
            device.wallet_id,
            params.asset_id.map(|x| x.0),
            params.from_timestamp,
            params.limit.0,
        )
        .await?
        .into())
}

#[get("/devices/transactions/<id>")]
pub async fn get_device_transaction_by_id_v2(
    _device: AuthenticatedDevice,
    id: TransactionIdParam,
    client: &State<TransactionsClient>,
) -> Result<ApiResponse<Transaction>, ApiError> {
    get_device_transaction(id, client)
}

#[get("/devices/transaction/<id>")]
pub async fn get_device_transaction_v2(_device: AuthenticatedDevice, id: TransactionIdParam, client: &State<TransactionsClient>) -> Result<ApiResponse<Transaction>, ApiError> {
    get_device_transaction(id, client)
}

fn get_device_transaction(id: TransactionIdParam, client: &State<TransactionsClient>) -> Result<ApiResponse<Transaction>, ApiError> {
    Ok(client.get_transaction_by_id(&id.0)?.into())
}

#[post("/devices/address_names", format = "json", data = "<requests>")]
pub async fn get_device_address_names_v2(
    _device: AuthenticatedDevice,
    requests: DeviceJson<Vec<ChainAddress>>,
    client: &State<AddressNamesClient>,
) -> Result<ApiResponse<Vec<AddressName>>, ApiError> {
    Ok(client.get_address_names(requests.into_inner())?.into())
}

#[get("/devices/nft_assets")]
pub async fn get_device_nft_assets_v2(device: AuthenticatedDeviceWallet, client: &State<NFTClient>) -> Result<ApiResponse<Vec<NFTData>>, ApiError> {
    Ok(client.get_nft_assets_by_wallet_id(device.device_row.id, device.wallet_id).await?.into())
}

#[get("/devices/nft_assets/<asset_id>")]
pub async fn get_device_nft_asset_v2(_device: AuthenticatedDevice, asset_id: NftAssetIdParam, client: &State<NFTClient>) -> Result<ApiResponse<NFTAssetData>, ApiError> {
    Ok(client.get_nft_asset_data(asset_id.0)?.into())
}

#[post("/devices/nft_assets/<asset_id>/refresh")]
pub async fn refresh_device_nft_asset_v2(
    _device: AuthenticatedDeviceWallet,
    asset_id: NftAssetIdParam,
    stream_producer: &State<StreamProducer>,
) -> Result<ApiResponse<bool>, ApiError> {
    Ok(stream_producer.publish_fetch_nft_asset(asset_id.0).await?.into())
}

#[get("/devices/defi/positions")]
pub async fn get_device_defi_positions_v2(device: AuthenticatedDeviceWallet, client: &State<DefiClient>) -> Result<ApiResponse<Vec<DefiPosition>>, ApiError> {
    Ok(client.get_positions_by_wallet_id(device.device_row.id, device.wallet_id).await?.into())
}

#[get("/devices/rewards")]
pub async fn get_device_rewards_v2(device: AuthenticatedDeviceWallet, client: &State<RewardsClient>) -> Result<ApiResponse<Rewards>, ApiError> {
    Ok(client.get_rewards_by_wallet_id(device.wallet_id)?.into())
}

#[get("/devices/rewards/events")]
pub async fn get_device_rewards_events_v2(device: AuthenticatedDeviceWallet, client: &State<RewardsClient>) -> Result<ApiResponse<Vec<RewardEvent>>, ApiError> {
    Ok(client.get_rewards_events_by_wallet_id(device.wallet_id)?.into())
}

#[get("/devices/rewards/redemptions/<code>")]
pub async fn get_device_rewards_redemption_v2(_device: AuthenticatedDevice, code: &str, client: &State<RewardsClient>) -> Result<ApiResponse<RewardRedemptionOption>, ApiError> {
    Ok(client.get_rewards_redemption_option(code)?.into())
}

#[post("/devices/rewards/referrals/create", format = "json", data = "<request>")]
pub async fn create_device_referral_v2(
    device: AuthenticatedDevice,
    request: WalletSigned<primitives::ReferralCode>,
    ip: std::net::IpAddr,
    client: &State<RewardsClient>,
) -> Result<ApiResponse<Rewards>, ApiError> {
    let wallet_identifier = primitives::WalletId::Multicoin(request.address.clone()).id();
    Ok(client
        .create_username(
            &wallet_identifier,
            &request.data.code,
            device.device_row.id,
            &ip.to_string(),
            device.device_row.locale.as_str(),
        )
        .await?
        .into())
}

#[post("/devices/rewards/referrals/use", format = "json", data = "<request>")]
pub async fn use_device_referral_code_v2(
    device: AuthenticatedDevice,
    request: WalletSigned<primitives::ReferralCode>,
    ip: std::net::IpAddr,
    user_agent: UserAgent,
    client: &State<RewardsClient>,
) -> Result<ApiResponse<bool>, ApiError> {
    client
        .use_referral_code(&device.device_row, &request.address, &request.data.code, &ip.to_string(), &user_agent.0)
        .await?;
    Ok(true.into())
}

#[post("/devices/rewards/redeem", format = "json", data = "<request>")]
pub async fn redeem_device_rewards_v2(
    device: AuthenticatedDeviceWallet,
    request: WalletSigned<RedemptionRequest>,
    client: &State<RewardsRedemptionClient>,
) -> Result<ApiResponse<RedemptionResult>, ApiError> {
    if WalletId::Multicoin(request.address.clone()) != device.wallet_identifier {
        return Err(ApiError::BadRequest("Wallet signature mismatch".to_string()));
    }

    Ok(client.redeem_by_wallet_id(device.wallet_id, &request.data.id, device.device_row.id).await?.into())
}

#[put("/devices", format = "json", data = "<device_input>")]
pub async fn update_device_v2(device: AuthenticatedDevice, device_input: DeviceJson<Device>, client: &State<DevicesClient>) -> Result<ApiResponse<Device>, ApiError> {
    let device_input = device_input.into_inner();
    if device_input.id != device.device_row.device_id {
        return Err(ApiError::BadRequest("Device id mismatch".to_string()));
    }
    Ok(client.update_device(device_input)?.into())
}

#[post("/devices/push-notification")]
pub async fn send_push_notification_device_v2(device: AuthenticatedDevice, client: &State<DevicesClient>) -> Result<ApiResponse<bool>, ApiError> {
    Ok(ApiResponse::from(
        client.send_push_notification_device(&device.device_row.device_id).await.map_err(ApiError::from)?,
    ))
}

#[post("/devices/nft/report", format = "json", data = "<request>")]
pub async fn report_device_nft_v2(device: AuthenticatedDevice, request: DeviceJson<ReportNft>, client: &State<NFTClient>) -> Result<ApiResponse<bool>, ApiError> {
    let request = request.into_inner();
    let asset_id = request
        .asset_id
        .as_deref()
        .map(|asset_id| AssetId::new(asset_id).ok_or_else(|| ApiError::BadRequest(format!("Invalid asset_id: {asset_id}"))))
        .transpose()?;

    Ok(client
        .report_nft(&device.device_row.device_id, request.collection_id.clone(), asset_id, request.reason.clone())?
        .into())
}

#[get("/devices/name/resolve/<name>?<chain>")]
pub async fn get_device_name_resolve_v2(
    _device: AuthenticatedDevice,
    name: &str,
    chain: ChainParam,
    client: &State<NameClient>,
) -> Result<ApiResponse<Option<NameRecord>>, ApiError> {
    let result = client.resolve(name, chain.0).await;
    match result {
        Ok(record) => Ok(Some(record).into()),
        Err(_) => Ok(None.into()),
    }
}

#[post("/devices/scan/transaction", data = "<request>")]
pub async fn scan_device_transaction_v2(
    _device: AuthenticatedDevice,
    request: DeviceJson<ScanTransactionPayload>,
    client: &State<ScanClient>,
) -> Result<ApiResponse<ScanTransaction>, ApiError> {
    Ok(client.get_scan_transaction(request.into_inner()).await?.into())
}

#[get("/devices/wallet_configuration")]
pub async fn get_device_wallet_configuration_v2(
    device: AuthenticatedDeviceWallet,
    client: &State<WalletConfigurationClient>,
) -> Result<ApiResponse<WalletConfigurationResult>, ApiError> {
    Ok(client.get_configuration(device.device_row.id, device.wallet_id, device.wallet_identifier).await?.into())
}

#[get("/devices/notifications?<params..>")]
pub async fn get_device_notifications_v2(
    device: AuthenticatedDevice,
    params: DeviceNotificationsParams,
    client: &State<NotificationsClient>,
) -> Result<ApiResponse<Vec<InAppNotification>>, ApiError> {
    Ok(client.get_notifications(&device.device_row.device_id, params.from_timestamp, params.limit.0)?.into())
}

#[post("/devices/notifications/read")]
pub async fn mark_device_notifications_read_v2(device: AuthenticatedDevice, client: &State<NotificationsClient>) -> Result<ApiResponse<usize>, ApiError> {
    Ok(client.mark_all_as_read(&device.device_row.device_id)?.into())
}

#[get("/devices/subscriptions")]
pub async fn get_device_subscriptions_v2(device: AuthenticatedDevice, client: &State<WalletsClient>) -> Result<ApiResponse<Vec<WalletSubscriptionChains>>, ApiError> {
    Ok(client.get_subscriptions(device.device_row.id)?.into())
}

#[post("/devices/subscriptions", format = "json", data = "<subscriptions>")]
pub async fn add_device_subscriptions_v2(
    device: AuthenticatedDevice,
    subscriptions: DeviceJson<Vec<WalletSubscription>>,
    client: &State<WalletsClient>,
) -> Result<ApiResponse<usize>, ApiError> {
    Ok(client.add_subscriptions(device.device_row.id, subscriptions.into_inner()).await?.into())
}

#[delete("/devices/subscriptions", format = "json", data = "<subscriptions>")]
pub async fn delete_device_subscriptions_v2(
    device: AuthenticatedDevice,
    subscriptions: DeviceJson<Vec<WalletSubscriptionChains>>,
    client: &State<WalletsClient>,
) -> Result<ApiResponse<usize>, ApiError> {
    Ok(client.delete_subscriptions(device.device_row.id, subscriptions.into_inner()).await?.into())
}

#[get("/devices/auth/nonce")]
pub async fn get_auth_nonce_v2(device: AuthenticatedDevice, client: &State<Arc<AuthClient>>) -> Result<ApiResponse<AuthNonce>, ApiError> {
    Ok(client.get_nonce(&device.device_row.device_id).await?.into())
}

#[get("/devices/token")]
pub async fn get_device_token_v2(device: AuthenticatedDevice, config: &State<AuthConfig>, client: &State<Arc<AuthClient>>) -> Result<ApiResponse<DeviceToken>, ApiError> {
    Ok(client.create_device_token(&device.device_row.device_id, &config.jwt.secret, config.jwt.expiry)?.into())
}

#[get("/devices/price_alerts?<asset_id>")]
pub async fn get_device_price_alerts_v2(
    device: AuthenticatedDevice,
    asset_id: Option<AssetIdParam>,
    client: &State<pricer::PriceAlertClient>,
) -> Result<ApiResponse<PriceAlerts>, ApiError> {
    Ok(client.get_price_alerts(&device.device_row.device_id, asset_id.as_ref().map(|x| &x.0)).await?.into())
}

#[post("/devices/price_alerts", format = "json", data = "<price_alerts>")]
pub async fn add_device_price_alerts_v2(
    device: AuthenticatedDevice,
    price_alerts: DeviceJson<PriceAlerts>,
    client: &State<pricer::PriceAlertClient>,
) -> Result<ApiResponse<usize>, ApiError> {
    Ok(client.add_price_alerts(&device.device_row.device_id, price_alerts.into_inner()).await?.into())
}

#[delete("/devices/price_alerts", format = "json", data = "<price_alerts>")]
pub async fn delete_device_price_alerts_v2(
    device: AuthenticatedDevice,
    price_alerts: DeviceJson<PriceAlerts>,
    client: &State<pricer::PriceAlertClient>,
) -> Result<ApiResponse<usize>, ApiError> {
    Ok(client.delete_price_alerts(&device.device_row.device_id, price_alerts.into_inner()).await?.into())
}

#[get("/devices/fiat/transactions")]
pub async fn get_device_fiat_transactions_v2(
    device: AuthenticatedDeviceWallet,
    client: &State<FiatQuotesClient>,
) -> Result<ApiResponse<Vec<primitives::FiatTransactionData>>, ApiError> {
    Ok(client.get_transactions_by_device_wallet_id(device.device_row.id, device.wallet_id)?.into())
}

#[get("/fiat/assets/<quote_type>")]
pub async fn get_fiat_assets(quote_type: FiatQuoteTypeParam, client: &State<FiatQuotesClient>) -> Result<ApiResponse<FiatAssets>, ApiError> {
    Ok(client.get_assets(quote_type.0).await?.into())
}

#[get("/devices/fiat/assets/<quote_type>")]
pub async fn get_device_fiat_assets_v2(
    _device: AuthenticatedDevice,
    quote_type: FiatQuoteTypeParam,
    client: &State<FiatQuotesClient>,
) -> Result<ApiResponse<FiatAssets>, ApiError> {
    Ok(client.get_assets(quote_type.0).await?.into())
}

#[get("/devices/fiat/quotes/<quote_type>/<asset_id>?<amount>&<currency>&<provider>")]
pub async fn get_fiat_quotes_v2(
    _device: AuthenticatedDeviceWallet,
    quote_type: FiatQuoteTypeParam,
    asset_id: AssetIdParam,
    amount: f64,
    currency: CurrencyParam,
    provider: Option<FiatProviderIdParam>,
    ip: std::net::IpAddr,
    client: &State<FiatQuotesClient>,
) -> Result<ApiResponse<FiatQuotes>, ApiError> {
    let quote_request = FiatQuoteRequest {
        asset_id: asset_id.0,
        quote_type: quote_type.0,
        amount,
        currency: currency.0.as_ref().to_string(),
        provider_id: provider.map(|p| p.0.id().to_string()),
        ip_address: ip.to_string(),
    };
    let quotes = client.get_quotes(quote_request).await?;
    Ok(quotes.into())
}

#[get("/devices/fiat/quotes/<quote_id>/url")]
pub async fn get_fiat_quote_url_v2(
    device: AuthenticatedDeviceWallet,
    quote_id: &str,
    ip: std::net::IpAddr,
    client: &State<FiatQuotesClient>,
) -> Result<ApiResponse<FiatQuoteUrl>, ApiError> {
    let locale = device.device_row.locale.as_str();
    let ip_address = ip.to_string();
    let url = client.get_quote_url(quote_id, device.wallet_id, device.device_row.id, &ip_address, locale).await?;
    Ok(url.into())
}

#[post("/devices/portfolio/assets?<period>", format = "json", data = "<request>")]
pub async fn get_device_portfolio_assets_v2(
    _device: AuthenticatedDevice,
    period: ChartPeriodParam,
    request: DeviceJson<PortfolioAssetsRequest>,
    portfolio_client: &State<PortfolioClient>,
) -> Result<ApiResponse<PortfolioAssets>, ApiError> {
    Ok(portfolio_client.get_portfolio_charts(request.into_inner().assets, period.0)?.into())
}
