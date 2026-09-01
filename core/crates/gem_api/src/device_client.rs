use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use gem_auth::build_device_auth_header;
use gem_client::{ClientError, ContentType, build_request_url, deserialize_response};
use gem_jsonrpc::{RpcClientError, RpcProvider, Target};
use primitives::name::NameRecord;
use primitives::rewards::{RedemptionRequest, RedemptionResult};
use primitives::{
    AddressName, AuthNonce, AuthenticatedRequest, ChainAddress, ChartPeriod, Device, FiatQuoteType, FiatQuoteUrl, FiatQuotes, FiatTransactionData, InAppNotification, NFTAssetData,
    NFTAssetId, NFTData, PortfolioAssets, PortfolioAssetsRequest, PriceAlert, ReferralCode, ReportNft, Rewards, ScanTransaction, ScanTransactionPayload, SupportMessage,
    SupportMessageInput, TransactionsResponse, WalletConfigurationResult, WalletSubscription, WalletSubscriptionChains,
};
use serde::de::DeserializeOwned;

use crate::device_target::{GemDeviceApiBody, GemDeviceApiTarget};

/// Runs before any request scoped to a wallet, so the app can make sure the
/// backend already knows that wallet before the call goes out.
#[async_trait::async_trait]
pub trait WalletRequestPreflight: Send + Sync + std::fmt::Debug {
    async fn prepare(&self) -> Result<(), ClientError>;
}

/// Signs `/v2/devices/*` requests with the device Ed25519 key.
///
/// The key is passed in once at construction and is expected to stay in memory for
/// the life of the client. It is a device-scoped credential, unrelated to wallet keys
/// or recovery material, and is held this way deliberately so signing does not read
/// secure storage on every request.
#[derive(Debug, Clone)]
pub struct GemDeviceApiClient<E: RpcClientError> {
    base_url: String,
    provider: Arc<dyn RpcProvider<Error = E>>,
    device_private_key: Vec<u8>,
    preflight: std::sync::OnceLock<Arc<dyn WalletRequestPreflight>>,
}

impl<E: RpcClientError> GemDeviceApiClient<E> {
    pub fn new(base_url: String, provider: Arc<dyn RpcProvider<Error = E>>, device_private_key: Vec<u8>) -> Self {
        Self {
            base_url,
            provider,
            device_private_key,
            preflight: std::sync::OnceLock::new(),
        }
    }

    /// Installed after construction because the preflight needs the service that owns this client.
    pub fn set_preflight(&self, preflight: Arc<dyn WalletRequestPreflight>) {
        let _ = self.preflight.set(preflight);
    }

    pub async fn get_device(&self) -> Result<Option<Device>, ClientError> {
        self.send(GemDeviceApiTarget::GetDevice).await
    }

    pub async fn add_device(&self, device: Device) -> Result<Device, ClientError> {
        self.send(GemDeviceApiTarget::AddDevice(device)).await
    }

    pub async fn update_device(&self, device: Device) -> Result<Device, ClientError> {
        self.send(GemDeviceApiTarget::UpdateDevice(device)).await
    }

    pub async fn is_device_registered(&self) -> Result<bool, ClientError> {
        self.send(GemDeviceApiTarget::IsDeviceRegistered).await
    }

    pub async fn get_auth_nonce(&self) -> Result<AuthNonce, ClientError> {
        self.send(GemDeviceApiTarget::GetAuthNonce).await
    }

    pub async fn get_subscriptions(&self) -> Result<Vec<WalletSubscriptionChains>, ClientError> {
        let subscriptions: Option<Vec<WalletSubscriptionChains>> = self.send(GemDeviceApiTarget::GetSubscriptions).await?;
        Ok(subscriptions.unwrap_or_default())
    }

    pub async fn add_subscriptions(&self, subscriptions: Vec<WalletSubscription>) -> Result<(), ClientError> {
        self.send::<serde_json::Value>(GemDeviceApiTarget::AddSubscriptions(subscriptions)).await?;
        Ok(())
    }

    pub async fn delete_subscriptions(&self, subscriptions: Vec<WalletSubscriptionChains>) -> Result<(), ClientError> {
        self.send::<serde_json::Value>(GemDeviceApiTarget::DeleteSubscriptions(subscriptions)).await?;
        Ok(())
    }

    pub async fn get_price_alerts(&self, asset_id: Option<String>) -> Result<Vec<PriceAlert>, ClientError> {
        self.send(GemDeviceApiTarget::GetPriceAlerts { asset_id }).await
    }

    pub async fn add_price_alerts(&self, alerts: Vec<PriceAlert>) -> Result<(), ClientError> {
        self.send::<serde_json::Value>(GemDeviceApiTarget::AddPriceAlerts(alerts)).await?;
        Ok(())
    }

    pub async fn delete_price_alerts(&self, alerts: Vec<PriceAlert>) -> Result<(), ClientError> {
        self.send::<serde_json::Value>(GemDeviceApiTarget::DeletePriceAlerts(alerts)).await?;
        Ok(())
    }

    pub async fn get_transactions(&self, wallet_id: String, asset_id: Option<String>, from_timestamp: u64) -> Result<TransactionsResponse, ClientError> {
        self.send(GemDeviceApiTarget::GetTransactions {
            wallet_id,
            asset_id,
            from_timestamp,
        })
        .await
    }

    pub async fn get_assets_list(&self, wallet_id: String, from_timestamp: u64) -> Result<Vec<String>, ClientError> {
        self.send(GemDeviceApiTarget::GetAssetsList { wallet_id, from_timestamp }).await
    }

    pub async fn get_wallet_configuration(&self, wallet_id: String) -> Result<WalletConfigurationResult, ClientError> {
        self.send(GemDeviceApiTarget::GetWalletConfiguration { wallet_id }).await
    }

    pub async fn scan_transaction(&self, payload: ScanTransactionPayload) -> Result<ScanTransaction, ClientError> {
        self.send(GemDeviceApiTarget::ScanTransaction(payload)).await
    }

    pub async fn get_nft_assets(&self, wallet_id: String) -> Result<Vec<NFTData>, ClientError> {
        let assets: Option<Vec<NFTData>> = self.send(GemDeviceApiTarget::GetNftAssets { wallet_id }).await?;
        Ok(assets.unwrap_or_default())
    }

    pub async fn get_nft_asset(&self, asset_id: NFTAssetId) -> Result<NFTAssetData, ClientError> {
        self.send(GemDeviceApiTarget::GetNftAsset(asset_id)).await
    }

    pub async fn refresh_nft_asset(&self, wallet_id: String, asset_id: NFTAssetId) -> Result<bool, ClientError> {
        self.send(GemDeviceApiTarget::RefreshNftAsset { wallet_id, asset_id }).await
    }

    pub async fn report_nft(&self, report: ReportNft) -> Result<bool, ClientError> {
        self.send(GemDeviceApiTarget::ReportNft(report)).await
    }

    pub async fn get_support_messages(&self, from_timestamp: u64) -> Result<Vec<SupportMessage>, ClientError> {
        self.send(GemDeviceApiTarget::GetSupportMessages { from_timestamp }).await
    }

    pub async fn send_support_message(&self, input: SupportMessageInput) -> Result<SupportMessage, ClientError> {
        self.send(GemDeviceApiTarget::SendSupportMessage(input)).await
    }

    pub async fn send_support_image(&self, image: Vec<u8>, file_name: String, mime_type: String) -> Result<SupportMessage, ClientError> {
        self.send(GemDeviceApiTarget::SendSupportImage { image, file_name, mime_type }).await
    }

    pub async fn get_rewards(&self, wallet_id: String) -> Result<Rewards, ClientError> {
        self.send(GemDeviceApiTarget::GetRewards { wallet_id }).await
    }

    pub async fn create_referral(&self, wallet_id: String, request: AuthenticatedRequest<ReferralCode>) -> Result<Rewards, ClientError> {
        self.send(GemDeviceApiTarget::CreateReferral { wallet_id, request }).await
    }

    pub async fn use_referral_code(&self, wallet_id: String, request: AuthenticatedRequest<ReferralCode>) -> Result<bool, ClientError> {
        self.send(GemDeviceApiTarget::UseReferralCode { wallet_id, request }).await
    }

    pub async fn redeem_rewards(&self, wallet_id: String, request: AuthenticatedRequest<RedemptionRequest>) -> Result<RedemptionResult, ClientError> {
        self.send(GemDeviceApiTarget::RedeemRewards { wallet_id, request }).await
    }

    pub async fn get_notifications(&self, from_timestamp: u64) -> Result<Vec<InAppNotification>, ClientError> {
        self.send(GemDeviceApiTarget::GetNotifications { from_timestamp }).await
    }

    pub async fn mark_notifications_read(&self) -> Result<(), ClientError> {
        self.send_ignoring_body(GemDeviceApiTarget::MarkNotificationsRead).await
    }

    pub async fn get_fiat_quotes(&self, wallet_id: String, quote_type: FiatQuoteType, asset_id: String, amount: f64, currency: String) -> Result<FiatQuotes, ClientError> {
        self.send(GemDeviceApiTarget::GetFiatQuotes {
            wallet_id,
            quote_type,
            asset_id,
            amount,
            currency,
        })
        .await
    }

    pub async fn get_fiat_quote_url(&self, wallet_id: String, quote_id: String) -> Result<FiatQuoteUrl, ClientError> {
        self.send(GemDeviceApiTarget::GetFiatQuoteUrl { wallet_id, quote_id }).await
    }

    pub async fn get_fiat_transactions(&self, wallet_id: String) -> Result<Vec<FiatTransactionData>, ClientError> {
        self.send(GemDeviceApiTarget::GetFiatTransactions { wallet_id }).await
    }

    pub async fn get_name_record(&self, name: String, chain: String) -> Result<Option<NameRecord>, ClientError> {
        self.send(GemDeviceApiTarget::GetNameRecord { name, chain }).await
    }

    pub async fn get_address_names(&self, requests: Vec<ChainAddress>) -> Result<Vec<AddressName>, ClientError> {
        self.send(GemDeviceApiTarget::GetAddressNames(requests)).await
    }

    pub async fn get_portfolio_assets(&self, period: ChartPeriod, request: PortfolioAssetsRequest) -> Result<PortfolioAssets, ClientError> {
        self.send(GemDeviceApiTarget::GetPortfolioAssets { period, request }).await
    }

    async fn send<R: DeserializeOwned>(&self, target: GemDeviceApiTarget) -> Result<R, ClientError> {
        let response = self.request(target).await?;
        deserialize_response(&response)
    }

    async fn send_ignoring_body(&self, target: GemDeviceApiTarget) -> Result<(), ClientError> {
        self.request(target).await.map(|_| ())
    }

    async fn request(&self, target: GemDeviceApiTarget) -> Result<gem_client::Response, ClientError> {
        if let Some(preflight) = self.preflight.get().filter(|_| !target.wallet_id().is_empty()) {
            preflight.prepare().await?;
        }
        let path = target.path();
        let body = target.body()?;
        let (bytes, content_type) = match &body {
            Some(GemDeviceApiBody::Json(bytes)) => (bytes.as_slice(), ContentType::ApplicationJson.as_str().to_string()),
            Some(GemDeviceApiBody::Raw { data, content_type }) => (data.as_slice(), content_type.clone()),
            None => (&[][..], ContentType::ApplicationJson.as_str().to_string()),
        };

        let mut headers = self.authorization(&target, bytes)?;
        headers.insert("Content-Type".to_string(), content_type);

        let request = Target {
            url: build_request_url(&self.base_url, &path),
            method: target.method().into(),
            headers: Some(headers),
            body: body.map(|body| match body {
                GemDeviceApiBody::Json(bytes) => bytes,
                GemDeviceApiBody::Raw { data, .. } => data,
            }),
        };
        let response = self.provider.request(request).await.map_err(RpcClientError::into_client_error)?;
        if let Some(status) = response.status.filter(|status| !(200..300).contains(status)) {
            return Err(ClientError::Http { status, body: response.data });
        }
        Ok(response)
    }

    fn authorization(&self, target: &GemDeviceApiTarget, body: &[u8]) -> Result<HashMap<String, String>, ClientError> {
        let path = target.path();
        let signed_path = path.split('?').next().unwrap_or(&path);
        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| ClientError::Serialization(error.to_string()))?
            .as_millis() as u64;

        let header = build_device_auth_header(&self.device_private_key, target.method().as_ref(), signed_path, target.wallet_id(), body, timestamp_ms)
            .map_err(|error| ClientError::Serialization(error.to_string()))?;

        Ok(HashMap::from([("Authorization".to_string(), header)]))
    }
}
