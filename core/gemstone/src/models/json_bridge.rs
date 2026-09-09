use primitives::portfolio::{PerpetualAccountSummary, PerpetualPortfolio, PerpetualPortfolioTimeframeData};
use primitives::rewards::{RedemptionRequest, RedemptionResult};
use primitives::{ConfigResponse, ConfigVersions, FiatAssets, Markets, SearchResponse, StreamEvent, StreamMessage, SupportTyping};
use primitives::{
    TransactionPerpetualMetadata, TronStakeData, TronUnfreeze, UTXO,
};
use primitives::{
    AuthNonce, AuthPayload, FiatQuoteRequest, FiatTransactionData, InAppNotification, ReferralCode,
    Rewards, SupportMessage, SupportMessageInput, TransactionsResponse,
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
    AuthNonce,
    AuthPayload,
    ConfigResponse,
    ConfigVersions,
    FiatAssets,
    FiatQuoteRequest,
    FiatTransactionData,
    InAppNotification,
    Markets,
    PerpetualAccountSummary,
    PerpetualPortfolio,
    PerpetualPortfolioTimeframeData,
    RedemptionRequest,
    RedemptionResult,
    ReferralCode,
    Rewards,
    SearchResponse,
    StreamEvent,
    StreamMessage,
    SupportMessage,
    SupportMessageInput,
    SupportTyping,
    TransactionPerpetualMetadata,
    TransactionsResponse,
    TronStakeData,
    TronUnfreeze,
    UTXO,
);
