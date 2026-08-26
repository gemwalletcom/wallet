use primitives::FiatRate;
use primitives::contact::ContactAddress;
use primitives::TransactionId;
use primitives::Wallet;
use primitives::asset_balance::BalanceMetadata;
use primitives::chart::{ChartCandleStick, ChartCandleUpdate, ChartDateValue};
use primitives::currency::Currency;
use primitives::name::NameRecord;
use primitives::node::Node;
use primitives::perpetual::{CancelOrderData, PerpetualModifyConfirmData, PerpetualModifyPositionType, PerpetualReduceData, TPSLOrderData};
use primitives::perpetual::{PerpetualAccountMode, PerpetualBalance, PerpetualData, PerpetualMetadata, PerpetualPositionsSummary};
use primitives::portfolio::{PerpetualAccountSummary, PerpetualPortfolio, PerpetualPortfolioTimeframeData};
use primitives::rewards::{RedemptionRequest, RedemptionResult};
use primitives::stake_type::Resource;
use primitives::swap::{ApprovalData, SwapData, SwapPriceImpact, SwapPriceImpactType, SwapProviderData, SwapQuote, SwapQuoteData, SwapQuoteDataType};
use primitives::{
    AccountDataType, ApplicationMetadata, ApplicationMetadataSource, AssetType, ChainAsset, ConnectionComponent, ConnectionStatus, ContractCallData, DelegationBase,
    DelegationState, DelegationValidator, NFTAsset, NFTAssetData, NFTAttribute, NFTAttributeType, NFTData, NFTImages, NFTResource, NFTType, Payment, PaymentAmount, PaymentLink,
    PaymentRequest, PerpetualDirection, PerpetualMarginType, PerpetualMarketData, PerpetualOrderType, PerpetualPosition, PerpetualTriggerOrder, ReportNft, ScanAddressTarget,
    ScanTransaction, ScanTransactionPayload, SimulationPayloadField, SimulationResult, SolanaNftStandard, SolanaTokenProgramId, StakeProviderType, StakeValidator,
    TransactionPerpetualMetadata, TransactionState, TransactionType, TransferDataOutputAction, TransferDataOutputType, TronStakeData, TronUnfreeze, TronVote, UTXO,
};
use primitives::{
    AddressName, AuthNonce, AuthPayload, ChainAddress, Device, DeviceToken, FiatQuote, FiatQuoteRequest, FiatQuoteUrl, FiatQuotes, FiatTransactionData, InAppNotification,
    PortfolioAssets, PortfolioAssetsRequest, PriceAlert, ReferralCode, Rewards, SupportMessage, SupportMessageInput, TransactionsResponse, WalletConfigurationResult,
    WalletSubscription, WalletSubscriptionChains,
};
use primitives::{Asset, Delegation, EarnType, PerpetualConfirmData, PerpetualType, Price, StakeType, Transaction};
use primitives::{AssetBasic, AssetFull, AssetMarket, AssetPrice, BannerEvent, BannerState, ConfigResponse, ConfigVersions, Contact, FiatAssets, FiatQuoteType, Markets, PlatformStore, Release, SearchResponse, StreamEvent};
use primitives::{ChartPeriod, Charts};

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
    ApplicationMetadata,
    ApplicationMetadataSource,
    ApprovalData,
    Asset,
    AssetBasic,
    AssetFull,
    AssetMarket,
    AssetPrice,
    AssetType,
    AuthNonce,
    AuthPayload,
    BalanceMetadata,
    BannerEvent,
    BannerState,
    CancelOrderData,
    ChainAddress,
    ChainAsset,
    ChartCandleStick,
    ChartCandleUpdate,
    ChartDateValue,
    ChartPeriod,
    Charts,
    ConfigResponse,
    ConfigVersions,
    ConnectionComponent,
    ConnectionStatus,
    Contact,
    ContactAddress,
    ContractCallData,
    Currency,
    Delegation,
    DelegationBase,
    DelegationState,
    DelegationValidator,
    Device,
    DeviceToken,
    EarnType,
    FiatAssets,
    FiatQuote,
    FiatRate,
    FiatQuoteRequest,
    FiatQuoteType,
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
    PerpetualAccountMode,
    PerpetualAccountSummary,
    PerpetualBalance,
    PerpetualConfirmData,
    PerpetualData,
    PerpetualDirection,
    PerpetualMarginType,
    PerpetualMarketData,
    PerpetualMetadata,
    PerpetualModifyConfirmData,
    PerpetualModifyPositionType,
    PerpetualOrderType,
    PerpetualPortfolio,
    PerpetualPortfolioTimeframeData,
    PlatformStore,
    Release,
    PerpetualPosition,
    PerpetualPositionsSummary,
    PerpetualReduceData,
    PerpetualTriggerOrder,
    PerpetualType,
    PortfolioAssets,
    PortfolioAssetsRequest,
    Price,
    PriceAlert,
    RedemptionRequest,
    RedemptionResult,
    ReferralCode,
    ReportNft,
    Resource,
    Rewards,
    ScanAddressTarget,
    ScanTransaction,
    ScanTransactionPayload,
    SearchResponse,
    StreamEvent,
    SimulationPayloadField,
    SimulationResult,
    SolanaNftStandard,
    SolanaTokenProgramId,
    StakeProviderType,
    StakeType,
    StakeValidator,
    SupportMessage,
    SupportMessageInput,
    SwapData,
    SwapPriceImpact,
    SwapPriceImpactType,
    SwapProviderData,
    SwapQuote,
    SwapQuoteData,
    SwapQuoteDataType,
    TPSLOrderData,
    Transaction,
    TransactionPerpetualMetadata,
    TransactionId,
    TransactionState,
    TransactionType,
    TransactionsResponse,
    TransferDataOutputAction,
    TransferDataOutputType,
    TronStakeData,
    TronUnfreeze,
    TronVote,
    UTXO,
    WalletConfigurationResult,
    Wallet,
    WalletSubscription,
    WalletSubscriptionChains,
);
