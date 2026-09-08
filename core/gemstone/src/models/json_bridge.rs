use primitives::Charts;
use primitives::portfolio::{PerpetualAccountSummary, PerpetualPortfolio, PerpetualPortfolioTimeframeData};
use primitives::rewards::{RedemptionRequest, RedemptionResult};
use primitives::{AssetBasic, AssetFull, AssetMarket, ConfigResponse, ConfigVersions, FiatAssets, Markets, SearchResponse, StreamEvent, StreamMessage, SupportTyping};
use primitives::{
    AssetList, ChainAsset, Payment, PaymentAmount, PaymentLink, PaymentRequest, SolanaNftStandard, StakeValidator, TransactionPerpetualMetadata, TronStakeData, TronUnfreeze,
    TronVote, UTXO,
};
use primitives::{
    AuthNonce, AuthPayload, ChartValuePercentage, FiatQuote, FiatQuoteRequest, FiatQuoteUrl, FiatQuotes, FiatTransactionData, InAppNotification, PortfolioData, ReferralCode,
    Rewards, SupportMessage, SupportMessageInput, TransactionsResponse, WalletConfigurationResult, WalletSubscription, WalletSubscriptionChains,
};

macro_rules! json_bridge {
    ($($type:ident),* $(,)?) => {
        $(
            uniffi::custom_type!($type, String, {
                remote,
                lower: |value| match serde_json::to_string(&value) {
                    Ok(json) => json,
                    Err(error) => {
                        debug_assert!(false, concat!("failed to serialize ", stringify!($type), ": {}"), error);
                        String::new()
                    }
                },
                try_lift: |value| serde_json::from_str(&value).map_err(|error| {
                    uniffi::deps::anyhow::Error::msg(format!(concat!("invalid ", stringify!($type), ": {}"), error))
                }),
            });
        )*
    };
}

json_bridge!(
    AssetBasic,
    AssetFull,
    AssetList,
    AssetMarket,
    AuthNonce,
    AuthPayload,
    ChainAsset,
    ChartValuePercentage,
    Charts,
    ConfigResponse,
    ConfigVersions,
    FiatAssets,
    FiatQuote,
    FiatQuoteRequest,
    FiatQuoteUrl,
    FiatQuotes,
    FiatTransactionData,
    InAppNotification,
    Markets,
    Payment,
    PaymentAmount,
    PaymentLink,
    PaymentRequest,
    PerpetualAccountSummary,
    PerpetualPortfolio,
    PerpetualPortfolioTimeframeData,
    PortfolioData,
    RedemptionRequest,
    RedemptionResult,
    ReferralCode,
    Rewards,
    SearchResponse,
    StreamEvent,
    StreamMessage,
    SolanaNftStandard,
    StakeValidator,
    SupportMessage,
    SupportMessageInput,
    SupportTyping,
    TransactionPerpetualMetadata,
    TransactionsResponse,
    TronStakeData,
    TronUnfreeze,
    TronVote,
    UTXO,
    WalletConfigurationResult,
    WalletSubscription,
    WalletSubscriptionChains,
);
