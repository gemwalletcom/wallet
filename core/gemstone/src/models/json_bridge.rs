use primitives::Appearance;
use primitives::Charts;
use primitives::FiatRate;
use primitives::TransactionId;
use primitives::Wallet;
use primitives::asset_balance::BalanceMetadata;
use primitives::chart::{ChartCandleStick, ChartCandleUpdate, ChartDateValue};
use primitives::contact::ContactAddress;
use primitives::name::NameRecord;
use primitives::node::Node;
use primitives::perpetual::{CancelOrderData, PerpetualModifyConfirmData, PerpetualModifyPositionType, PerpetualReduceData, TPSLOrderData};
use primitives::perpetual::{PerpetualBalance, PerpetualData, PerpetualMetadata, PerpetualPositionsSummary};
use primitives::portfolio::{PerpetualAccountSummary, PerpetualPortfolio, PerpetualPortfolioTimeframeData};
use primitives::rewards::{RedemptionRequest, RedemptionResult};
use primitives::swap::{ApprovalData, SwapData, SwapPriceImpact, SwapPriceImpactType, SwapProviderData, SwapQuote, SwapQuoteData, SwapQuoteDataType};
use primitives::{
    AccountDataType, ApplicationMetadata, ApplicationMetadataSource, AssetList, ChainAsset, ContractCallData, DelegationBase, DelegationValidator, NFTAsset, NFTAssetData,
    NFTAttribute, NFTAttributeType, NFTData, NFTImages, NFTResource, NFTType, Payment, PaymentAmount, PaymentLink, PaymentRequest, Perpetual, PerpetualMarketData,
    PerpetualPosition, PerpetualTriggerOrder, ReportNft, ScanAddressTarget, ScanTransaction, ScanTransactionPayload, SimulationHeader, SimulationResult, SimulationWarning,
    SolanaNftStandard, SolanaTokenProgramId, StakeValidator, TransactionPerpetualMetadata, TronStakeData, TronUnfreeze, TronVote, UTXO,
};
use primitives::{
    AddressName, AuthNonce, AuthPayload, ChainAddress, ChartValuePercentage, Device, FiatQuote, FiatQuoteRequest, FiatQuoteUrl, FiatQuotes, FiatTransactionData, InAppNotification,
    PortfolioAsset, PortfolioAssets, PortfolioAssetsRequest, PortfolioData, PriceAlert, ReferralCode, Rewards, SupportMessage, SupportMessageInput, TransactionsResponse,
    WalletConfigurationResult, WalletConnection, WalletConnectionSession, WalletConnectionSessionProposal, WalletSubscription, WalletSubscriptionChains,
};
use primitives::{
    AssetBasic, AssetFull, AssetMarket, AssetPrice, ConfigResponse, ConfigVersions, Contact, FiatAssets, Markets, Release, SearchResponse, StreamEvent, StreamMessage,
    SupportTyping,
};
use primitives::{Delegation, EarnType, PerpetualConfirmData, PerpetualType, Price, StakeType, Transaction, TransactionExtended};
use primitives::{DeviceLocale, Platform};

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
    AccountDataType,
    AddressName,
    Appearance,
    ApplicationMetadata,
    ApplicationMetadataSource,
    ApprovalData,
    AssetBasic,
    AssetFull,
    AssetList,
    AssetMarket,
    AssetPrice,
    AuthNonce,
    AuthPayload,
    BalanceMetadata,
    CancelOrderData,
    ChainAddress,
    ChainAsset,
    ChartCandleStick,
    ChartCandleUpdate,
    ChartDateValue,
    ChartValuePercentage,
    Charts,
    ConfigResponse,
    ConfigVersions,
    Contact,
    ContactAddress,
    ContractCallData,
    Delegation,
    DelegationBase,
    DelegationValidator,
    Device,
    DeviceLocale,
    EarnType,
    FiatAssets,
    FiatQuote,
    FiatRate,
    FiatQuoteRequest,
    FiatQuoteUrl,
    FiatQuotes,
    FiatTransactionData,
    InAppNotification,
    Markets,
    NFTAsset,
    NFTAssetData,
    NFTAttribute,
    NFTAttributeType,
    NFTData,
    Node,
    NFTImages,
    NFTResource,
    NFTType,
    NameRecord,
    Payment,
    PaymentAmount,
    PaymentLink,
    PaymentRequest,
    PerpetualAccountSummary,
    PerpetualBalance,
    PerpetualConfirmData,
    Perpetual,
    PerpetualData,
    PerpetualMarketData,
    PerpetualMetadata,
    PerpetualModifyConfirmData,
    PerpetualModifyPositionType,
    PerpetualPortfolio,
    PerpetualPortfolioTimeframeData,
    Platform,
    Release,
    PerpetualPosition,
    PerpetualPositionsSummary,
    PerpetualReduceData,
    PerpetualTriggerOrder,
    PerpetualType,
    PortfolioAssets,
    PortfolioAsset,
    PortfolioAssetsRequest,
    PortfolioData,
    Price,
    PriceAlert,
    RedemptionRequest,
    RedemptionResult,
    ReferralCode,
    ReportNft,
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
    SolanaTokenProgramId,
    StakeType,
    StakeValidator,
    SupportMessage,
    SupportMessageInput,
    SupportTyping,
    SwapData,
    SwapPriceImpact,
    SwapPriceImpactType,
    SwapProviderData,
    SwapQuote,
    SwapQuoteData,
    SwapQuoteDataType,
    TPSLOrderData,
    Transaction,
    TransactionExtended,
    TransactionPerpetualMetadata,
    TransactionId,
    TransactionsResponse,
    TronStakeData,
    TronUnfreeze,
    TronVote,
    UTXO,
    WalletConfigurationResult,
    WalletConnection,
    WalletConnectionSession,
    WalletConnectionSessionProposal,
    Wallet,
    WalletSubscription,
    WalletSubscriptionChains,
);
