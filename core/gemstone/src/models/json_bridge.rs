use primitives::Charts;
use primitives::name::NameRecord;
use primitives::perpetual::{CancelOrderData, PerpetualModifyConfirmData, PerpetualModifyPositionType, PerpetualReduceData, TPSLOrderData};
use primitives::portfolio::{PerpetualAccountSummary, PerpetualPortfolio, PerpetualPortfolioTimeframeData};
use primitives::rewards::{RedemptionRequest, RedemptionResult};
use primitives::swap::{ApprovalData, SwapData, SwapProviderData, SwapQuote, SwapQuoteData};
use primitives::{AssetBasic, AssetFull, AssetMarket, ConfigResponse, ConfigVersions, FiatAssets, Markets, SearchResponse, StreamEvent, StreamMessage, SupportTyping};
use primitives::{
    AssetList, ChainAsset, ContractCallData, DelegationBase, NFTAsset, NFTAssetData, NFTAttribute, NFTData, NFTImages, NFTResource, Payment, PaymentAmount, PaymentLink,
    PaymentRequest, ScanAddressTarget, ScanTransaction, ScanTransactionPayload, SimulationHeader, SimulationResult, SimulationWarning, SolanaNftStandard, StakeValidator,
    TransactionPerpetualMetadata, TronStakeData, TronUnfreeze, TronVote, UTXO,
};
use primitives::{
    AuthNonce, AuthPayload, ChartValuePercentage, Device, FiatQuote, FiatQuoteRequest, FiatQuoteUrl, FiatQuotes, FiatTransactionData, InAppNotification, PortfolioAsset,
    PortfolioAssets, PortfolioAssetsRequest, PortfolioData, ReferralCode, Rewards, SupportMessage, SupportMessageInput, TransactionsResponse, WalletConfigurationResult,
    WalletConnection, WalletConnectionSession, WalletConnectionSessionProposal, WalletSubscription, WalletSubscriptionChains,
};
use primitives::{Delegation, EarnType, PerpetualConfirmData, PerpetualType, Price, StakeType, Transaction, TransactionExtended};

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
    ApprovalData,
    AssetBasic,
    AssetFull,
    AssetList,
    AssetMarket,
    AuthNonce,
    AuthPayload,
    CancelOrderData,
    ChainAsset,
    ChartValuePercentage,
    Charts,
    ConfigResponse,
    ConfigVersions,
    ContractCallData,
    Delegation,
    DelegationBase,
    Device,
    EarnType,
    FiatAssets,
    FiatQuote,
    FiatQuoteRequest,
    FiatQuoteUrl,
    FiatQuotes,
    FiatTransactionData,
    InAppNotification,
    Markets,
    NFTAsset,
    NFTAssetData,
    NFTAttribute,
    NFTData,
    NFTImages,
    NFTResource,
    NameRecord,
    Payment,
    PaymentAmount,
    PaymentLink,
    PaymentRequest,
    PerpetualAccountSummary,
    PerpetualConfirmData,
    PerpetualModifyConfirmData,
    PerpetualModifyPositionType,
    PerpetualPortfolio,
    PerpetualPortfolioTimeframeData,
    PerpetualReduceData,
    PerpetualType,
    PortfolioAssets,
    PortfolioAsset,
    PortfolioAssetsRequest,
    PortfolioData,
    Price,
    RedemptionRequest,
    RedemptionResult,
    ReferralCode,
    Rewards,
    ScanAddressTarget,
    ScanTransaction,
    ScanTransactionPayload,
    SearchResponse,
    StreamEvent,
    StreamMessage,
    SimulationHeader,
    SimulationResult,
    SimulationWarning,
    SolanaNftStandard,
    StakeType,
    StakeValidator,
    SupportMessage,
    SupportMessageInput,
    SupportTyping,
    SwapData,
    SwapProviderData,
    SwapQuote,
    SwapQuoteData,
    TPSLOrderData,
    Transaction,
    TransactionExtended,
    TransactionPerpetualMetadata,
    TransactionsResponse,
    TronStakeData,
    TronUnfreeze,
    TronVote,
    UTXO,
    WalletConfigurationResult,
    WalletConnection,
    WalletConnectionSession,
    WalletConnectionSessionProposal,
    WalletSubscription,
    WalletSubscriptionChains,
);
