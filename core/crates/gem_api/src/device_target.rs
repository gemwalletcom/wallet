use gem_client::ClientError;
use primitives::rewards::RedemptionRequest;
use primitives::{
    AuthenticatedRequest, ChainAddress, ChartPeriod, Device, FiatQuoteType, NFTAssetId, PortfolioAssetsRequest, PriceAlert, ReferralCode, ReportNft, ScanTransactionPayload,
    SupportMessageInput, WalletSubscription, WalletSubscriptionChains,
};
use serde::Serialize;

use crate::method::GemApiMethod;

#[derive(Clone, Debug)]
pub enum GemDeviceApiBody {
    Json(Vec<u8>),
    Raw { data: Vec<u8>, content_type: String },
}

#[derive(Clone, Debug)]
pub enum GemDeviceApiTarget {
    GetDevice,
    AddDevice(Device),
    UpdateDevice(Device),
    IsDeviceRegistered,
    GetAuthNonce,

    GetSubscriptions,
    AddSubscriptions(Vec<WalletSubscription>),
    DeleteSubscriptions(Vec<WalletSubscriptionChains>),

    GetPriceAlerts {
        asset_id: Option<String>,
    },
    AddPriceAlerts(Vec<PriceAlert>),
    DeletePriceAlerts(Vec<PriceAlert>),

    GetTransactions {
        wallet_id: String,
        asset_id: Option<String>,
        from_timestamp: u64,
    },
    GetAssetsList {
        wallet_id: String,
        from_timestamp: u64,
    },
    GetWalletConfiguration {
        wallet_id: String,
    },

    ScanTransaction(ScanTransactionPayload),

    GetNftAssets {
        wallet_id: String,
    },
    GetNftAsset(NFTAssetId),
    RefreshNftAsset {
        wallet_id: String,
        asset_id: NFTAssetId,
    },
    ReportNft(ReportNft),

    GetSupportMessages {
        from_timestamp: u64,
    },
    SendSupportMessage(SupportMessageInput),
    SendSupportImage {
        image: Vec<u8>,
        file_name: String,
        mime_type: String,
    },

    GetRewards {
        wallet_id: String,
    },
    CreateReferral {
        wallet_id: String,
        request: AuthenticatedRequest<ReferralCode>,
    },
    UseReferralCode {
        wallet_id: String,
        request: AuthenticatedRequest<ReferralCode>,
    },
    RedeemRewards {
        wallet_id: String,
        request: AuthenticatedRequest<RedemptionRequest>,
    },

    GetNotifications {
        from_timestamp: u64,
    },
    MarkNotificationsRead,

    GetFiatQuotes {
        wallet_id: String,
        quote_type: FiatQuoteType,
        asset_id: String,
        amount: f64,
        currency: String,
    },
    GetFiatQuoteUrl {
        wallet_id: String,
        quote_id: String,
    },
    GetFiatTransactions {
        wallet_id: String,
    },

    GetNameRecord {
        name: String,
        chain: String,
    },
    GetAddressNames(Vec<ChainAddress>),

    GetPortfolioAssets {
        period: ChartPeriod,
        request: PortfolioAssetsRequest,
    },
}

impl GemDeviceApiTarget {
    pub fn method(&self) -> GemApiMethod {
        match self {
            Self::GetDevice
            | Self::IsDeviceRegistered
            | Self::GetAuthNonce
            | Self::GetSubscriptions
            | Self::GetPriceAlerts { .. }
            | Self::GetTransactions { .. }
            | Self::GetAssetsList { .. }
            | Self::GetWalletConfiguration { .. }
            | Self::GetNftAssets { .. }
            | Self::GetNftAsset(_)
            | Self::GetSupportMessages { .. }
            | Self::GetRewards { .. }
            | Self::GetNotifications { .. }
            | Self::GetFiatQuotes { .. }
            | Self::GetFiatQuoteUrl { .. }
            | Self::GetFiatTransactions { .. }
            | Self::GetNameRecord { .. } => GemApiMethod::Get,
            Self::UpdateDevice(_) => GemApiMethod::Put,
            Self::DeleteSubscriptions(_) | Self::DeletePriceAlerts(_) => GemApiMethod::Delete,
            Self::AddDevice(_)
            | Self::AddSubscriptions(_)
            | Self::AddPriceAlerts(_)
            | Self::ScanTransaction(_)
            | Self::RefreshNftAsset { .. }
            | Self::ReportNft(_)
            | Self::SendSupportMessage(_)
            | Self::SendSupportImage { .. }
            | Self::CreateReferral { .. }
            | Self::UseReferralCode { .. }
            | Self::RedeemRewards { .. }
            | Self::MarkNotificationsRead
            | Self::GetAddressNames(_)
            | Self::GetPortfolioAssets { .. } => GemApiMethod::Post,
        }
    }

    pub fn path(&self) -> String {
        match self {
            Self::GetDevice | Self::AddDevice(_) | Self::UpdateDevice(_) => "/v2/devices".to_string(),
            Self::IsDeviceRegistered => "/v2/devices/is_registered".to_string(),
            Self::GetAuthNonce => "/v2/devices/auth/nonce".to_string(),
            Self::GetSubscriptions | Self::AddSubscriptions(_) | Self::DeleteSubscriptions(_) => "/v2/devices/subscriptions".to_string(),
            Self::GetPriceAlerts { asset_id } => match asset_id {
                Some(asset_id) => format!("/v2/devices/price_alerts?asset_id={asset_id}"),
                None => "/v2/devices/price_alerts".to_string(),
            },
            Self::AddPriceAlerts(_) | Self::DeletePriceAlerts(_) => "/v2/devices/price_alerts".to_string(),
            Self::GetTransactions { asset_id, from_timestamp, .. } => match asset_id {
                Some(asset_id) => format!("/v2/devices/transactions?from_timestamp={from_timestamp}&asset_id={asset_id}"),
                None => format!("/v2/devices/transactions?from_timestamp={from_timestamp}"),
            },
            Self::GetAssetsList { from_timestamp, .. } => format!("/v2/devices/assets?from_timestamp={from_timestamp}"),
            Self::GetWalletConfiguration { .. } => "/v2/devices/wallet_configuration".to_string(),
            Self::ScanTransaction(_) => "/v2/devices/scan/transaction".to_string(),
            Self::GetNftAssets { .. } => "/v2/devices/nft_assets".to_string(),
            Self::GetNftAsset(asset_id) => format!("/v2/devices/nft_assets/{asset_id}"),
            Self::RefreshNftAsset { asset_id, .. } => format!("/v2/devices/nft_assets/{asset_id}/refresh"),
            Self::ReportNft(_) => "/v2/devices/nft/report".to_string(),
            Self::GetSupportMessages { from_timestamp } => format!("/v2/devices/support/messages?from_timestamp={from_timestamp}"),
            Self::SendSupportMessage(_) => "/v2/devices/support/messages".to_string(),
            Self::SendSupportImage { file_name, .. } => format!(
                "/v2/devices/support/messages/images?file_name={}",
                percent_encoding::utf8_percent_encode(file_name, percent_encoding::NON_ALPHANUMERIC)
            ),
            Self::GetRewards { .. } => "/v2/devices/rewards".to_string(),
            Self::CreateReferral { .. } => "/v2/devices/rewards/referrals/create".to_string(),
            Self::UseReferralCode { .. } => "/v2/devices/rewards/referrals/use".to_string(),
            Self::RedeemRewards { .. } => "/v2/devices/rewards/redeem".to_string(),
            Self::GetNotifications { from_timestamp } => format!("/v2/devices/notifications?from_timestamp={from_timestamp}"),
            Self::MarkNotificationsRead => "/v2/devices/notifications/read".to_string(),
            Self::GetFiatQuotes {
                quote_type,
                asset_id,
                amount,
                currency,
                ..
            } => format!("/v2/devices/fiat/quotes/{}/{asset_id}?amount={amount}&currency={currency}", quote_type.as_ref()),
            Self::GetFiatQuoteUrl { quote_id, .. } => format!("/v2/devices/fiat/quotes/{quote_id}/url"),
            Self::GetFiatTransactions { .. } => "/v2/devices/fiat/transactions".to_string(),
            Self::GetNameRecord { name, chain } => format!("/v2/devices/name/resolve/{name}?chain={chain}"),
            Self::GetAddressNames(_) => "/v2/devices/address_names".to_string(),
            Self::GetPortfolioAssets { period, .. } => format!("/v2/devices/portfolio/assets?period={}", period.as_ref()),
        }
    }

    pub fn wallet_id(&self) -> &str {
        match self {
            Self::GetTransactions { wallet_id, .. }
            | Self::GetAssetsList { wallet_id, .. }
            | Self::GetWalletConfiguration { wallet_id }
            | Self::GetNftAssets { wallet_id }
            | Self::RefreshNftAsset { wallet_id, .. }
            | Self::GetRewards { wallet_id }
            | Self::CreateReferral { wallet_id, .. }
            | Self::UseReferralCode { wallet_id, .. }
            | Self::RedeemRewards { wallet_id, .. }
            | Self::GetFiatQuotes { wallet_id, .. }
            | Self::GetFiatQuoteUrl { wallet_id, .. }
            | Self::GetFiatTransactions { wallet_id } => wallet_id.as_str(),
            _ => "",
        }
    }

    pub fn body(&self) -> Result<Option<GemDeviceApiBody>, ClientError> {
        match self {
            Self::AddDevice(device) | Self::UpdateDevice(device) => json(device),
            Self::AddSubscriptions(subscriptions) => json(subscriptions),
            Self::DeleteSubscriptions(subscriptions) => json(subscriptions),
            Self::AddPriceAlerts(alerts) | Self::DeletePriceAlerts(alerts) => json(alerts),
            Self::ScanTransaction(payload) => json(payload),
            Self::ReportNft(report) => json(report),
            Self::SendSupportMessage(input) => json(input),
            Self::SendSupportImage { image, mime_type, .. } => Ok(Some(GemDeviceApiBody::Raw {
                data: image.clone(),
                content_type: mime_type.clone(),
            })),
            Self::CreateReferral { request, .. } | Self::UseReferralCode { request, .. } => json(request),
            Self::RedeemRewards { request, .. } => json(request),
            Self::GetAddressNames(requests) => json(requests),
            Self::GetPortfolioAssets { request, .. } => json(request),
            _ => Ok(None),
        }
    }
}

fn json<T: Serialize>(value: &T) -> Result<Option<GemDeviceApiBody>, ClientError> {
    Ok(Some(GemDeviceApiBody::Json(serde_json::to_vec(value)?)))
}
