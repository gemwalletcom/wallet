use primitives::asset_balance::BalanceMetadata;
use primitives::chart::{ChartCandleStick, ChartCandleUpdate, ChartDateValue};
use primitives::currency::Currency;
use primitives::perpetual::{CancelOrderData, PerpetualModifyConfirmData, PerpetualModifyPositionType, PerpetualReduceData, TPSLOrderData};
use primitives::perpetual::{PerpetualAccountMode, PerpetualBalance, PerpetualData, PerpetualMetadata, PerpetualPositionsSummary};
use primitives::portfolio::{PerpetualAccountSummary, PerpetualPortfolio, PerpetualPortfolioTimeframeData};
use primitives::stake_type::Resource;
use primitives::swap::{ApprovalData, SwapData, SwapPriceImpact, SwapPriceImpactType, SwapProviderData, SwapQuote, SwapQuoteData, SwapQuoteDataType};
use primitives::{
    AccountDataType, ApplicationMetadata, ApplicationMetadataSource, AssetType, ChainAsset, ConnectionComponent, ConnectionStatus, ContractCallData, DelegationBase,
    DelegationState, DelegationValidator, NFTAsset, NFTAttribute, NFTAttributeType, NFTImages, NFTResource, NFTType, Payment, PaymentAmount, PaymentLink, PaymentRequest,
    PerpetualDirection, PerpetualMarginType, PerpetualMarketData, PerpetualOrderType, PerpetualPosition, PerpetualTriggerOrder, ScanAddressTarget, ScanTransaction,
    ScanTransactionPayload, SimulationPayloadField, SimulationResult, SolanaNftStandard, SolanaTokenProgramId, StakeProviderType, StakeValidator, TransactionPerpetualMetadata,
    TransactionState, TransactionType, TransferDataOutputAction, TransferDataOutputType, TronStakeData, TronUnfreeze, TronVote, UTXO,
};
use primitives::{Asset, Delegation, EarnType, PerpetualConfirmData, PerpetualType, Price, StakeType, Transaction};
use primitives::{AssetBasic, AssetFull, AssetMarket, AssetPrice, BannerEvent, BannerState, ConfigResponse, FiatAssets, FiatQuoteType, Markets, SearchResponse};
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
    ApplicationMetadata,
    ApplicationMetadataSource,
    ApprovalData,
    Asset,
    AssetBasic,
    AssetFull,
    AssetMarket,
    AssetPrice,
    AssetType,
    BalanceMetadata,
    BannerEvent,
    BannerState,
    CancelOrderData,
    ChainAsset,
    ChartCandleStick,
    ChartCandleUpdate,
    ChartDateValue,
    ChartPeriod,
    Charts,
    ConfigResponse,
    ConnectionComponent,
    ConnectionStatus,
    ContractCallData,
    Currency,
    Delegation,
    DelegationBase,
    DelegationState,
    DelegationValidator,
    EarnType,
    FiatAssets,
    FiatQuoteType,
    Markets,
    NFTAsset,
    NFTAttribute,
    NFTAttributeType,
    NFTImages,
    NFTResource,
    NFTType,
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
    PerpetualPosition,
    PerpetualPositionsSummary,
    PerpetualReduceData,
    PerpetualTriggerOrder,
    PerpetualType,
    Price,
    Resource,
    ScanAddressTarget,
    ScanTransaction,
    ScanTransactionPayload,
    SearchResponse,
    SimulationPayloadField,
    SimulationResult,
    SolanaNftStandard,
    SolanaTokenProgramId,
    StakeProviderType,
    StakeType,
    StakeValidator,
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
    TransactionState,
    TransactionType,
    TransferDataOutputAction,
    TransferDataOutputType,
    TronStakeData,
    TronUnfreeze,
    TronVote,
    UTXO,
);
