use primitives::asset_balance::BalanceMetadata;
use primitives::chart::{ChartCandleStick, ChartCandleUpdate, ChartDateValue};
use primitives::perpetual::{CancelOrderData, PerpetualModifyConfirmData, PerpetualModifyPositionType, PerpetualReduceData, TPSLOrderData};
use primitives::perpetual::{PerpetualAccountMode, PerpetualBalance, PerpetualData, PerpetualMetadata, PerpetualPositionsSummary};
use primitives::portfolio::{PerpetualAccountSummary, PerpetualPortfolio, PerpetualPortfolioTimeframeData};
use primitives::stake_type::Resource;
use primitives::swap::{ApprovalData, SwapData, SwapPriceImpact, SwapPriceImpactType, SwapProviderData, SwapQuote, SwapQuoteData, SwapQuoteDataType};
use primitives::{
    AccountDataType, ApplicationMetadata, ApplicationMetadataSource, AssetType, ChainAsset, ConnectionComponent, ConnectionStatus, ContractCallData, DelegationBase,
    DelegationState, DelegationValidator, NFTAsset, NFTAttribute, NFTAttributeType, NFTImages, NFTResource, NFTType, Payment, PaymentAmount, PaymentLink, PaymentRequest,
    PerpetualDirection, PerpetualMarginType, PerpetualMarketData, PerpetualOrderType, PerpetualPosition, PerpetualTriggerOrder, ScanAddressTarget, ScanTransaction,
    ScanTransactionPayload, SimulationPayloadField, SimulationResult, SolanaNftStandard, SolanaTokenProgramId, StakeProviderType, TransactionPerpetualMetadata, TransactionState,
    TransactionType, TransferDataOutputAction, TransferDataOutputType, TronStakeData, TronUnfreeze, TronVote, UTXO,
};
use primitives::{Asset, Delegation, EarnType, PerpetualConfirmData, PerpetualType, Price, StakeType};

/// Bridges `primitives` types across the FFI as JSON strings.
///
/// The platform side decodes into its typeshare-generated counterpart, which is
/// produced from the same `primitives` type. That keeps `primitives` the single
/// source of truth: no uniffi mirror record here, and no hand-written mapper on
/// Swift or Kotlin.
///
/// Use this for records and enums. Newtype-ish values that already have a lossless
/// string form (`AssetId`, `Chain`, `BigInt`) belong in `custom_types` instead —
/// a bare string is cheaper than JSON and is equally free of duplication.
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

// Every `primitives` type that crosses the FFI as JSON. Add a type here and the
// platform side decodes it into its typeshare-generated counterpart — no uniffi
// mirror record, no hand-written mapper on either platform.
json_bridge!(
    AccountDataType,
    ApplicationMetadata,
    ApplicationMetadataSource,
    ApprovalData,
    Asset,
    AssetType,
    BalanceMetadata,
    CancelOrderData,
    ChainAsset,
    ChartCandleStick,
    ChartCandleUpdate,
    ChartDateValue,
    ConnectionComponent,
    ConnectionStatus,
    ContractCallData,
    Delegation,
    DelegationBase,
    DelegationState,
    DelegationValidator,
    EarnType,
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
    SimulationPayloadField,
    SimulationResult,
    SolanaNftStandard,
    SolanaTokenProgramId,
    StakeProviderType,
    StakeType,
    SwapData,
    SwapPriceImpact,
    SwapPriceImpactType,
    SwapProviderData,
    SwapQuote,
    SwapQuoteData,
    SwapQuoteDataType,
    TPSLOrderData,
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
