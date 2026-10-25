use super::permit2_data::Permit2Data;
use crate::{
    SwapperError, SwapperProvider, SwapperQuoteAsset, SwapperSlippage,
    config::DEFAULT_SLIPPAGE_BPS,
    fees::{ReferralFee, on_chain_referral_fees, reserved_transaction_fees},
};
use num_bigint::BigUint;
use num_traits::ToPrimitive;
pub use primitives::swap::Permit2ApprovalData;
pub use primitives::swap::{SwapResult, SwapResultRequest};
use primitives::{
    AssetId, Chain,
    known_assets::STABLECOIN_ASSET_IDS,
    swap::{SlippageMode, SwapProviderMode},
};
use serde::{Serialize, de::DeserializeOwned};
use serde_serializers::serialize_display;
use serde_serializers::{serialize_biguint, serialize_option_biguint};
use std::fmt::Debug;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProviderType {
    pub id: SwapperProvider,
    pub name: String,
    pub protocol: String,
    pub protocol_id: String,
    pub mode: SwapProviderMode,
    pub slippage_mode: SlippageMode,
}

impl ProviderType {
    pub fn new(id: SwapperProvider) -> Self {
        Self {
            id,
            name: id.name().to_string(),
            protocol: id.name().to_string(),
            protocol_id: id.id().to_string(),
            mode: ProviderType::mode(id),
            slippage_mode: ProviderType::slippage_mode(id),
        }
    }

    pub fn mode(id: SwapperProvider) -> SwapProviderMode {
        match id {
            SwapperProvider::UniswapV3
            | SwapperProvider::UniswapV4
            | SwapperProvider::PancakeswapV3
            | SwapperProvider::Panora
            | SwapperProvider::Jupiter
            | SwapperProvider::Oku
            | SwapperProvider::Wagmi
            | SwapperProvider::CetusClmm
            | SwapperProvider::StonfiV2
            | SwapperProvider::Aerodrome
            | SwapperProvider::Orca
            | SwapperProvider::RaydiumClmm
            | SwapperProvider::Hyperion
            | SwapperProvider::Dedust
            | SwapperProvider::PumpSwap
            | SwapperProvider::MeteoraDammV2
            | SwapperProvider::SunswapV3
            | SwapperProvider::StellarDex
            | SwapperProvider::DeepbookV3
            | SwapperProvider::Aero
            | SwapperProvider::Okx => SwapProviderMode::OnChain,
            SwapperProvider::Mayan | SwapperProvider::Mayachain | SwapperProvider::Squid | SwapperProvider::SwapsXyz | SwapperProvider::Bridgers => SwapProviderMode::CrossChain,
            SwapperProvider::Chainflip => SwapProviderMode::OmniChain(vec![Chain::Tron]),
            SwapperProvider::Thorchain => SwapProviderMode::OmniChain(vec![Chain::Thorchain, Chain::Tron]),
            SwapperProvider::NearIntents => SwapProviderMode::OmniChain(vec![Chain::Near, Chain::Tron]),
            SwapperProvider::Relay => SwapProviderMode::OmniChain(vec![Chain::Hyperliquid, Chain::Berachain]),
            SwapperProvider::Across => SwapProviderMode::Bridge,
            SwapperProvider::Hyperliquid => SwapProviderMode::OmniChain(vec![Chain::HyperCore, Chain::Hyperliquid]),
        }
    }

    pub fn slippage_mode(id: SwapperProvider) -> SlippageMode {
        match id {
            SwapperProvider::Okx | SwapperProvider::Squid | SwapperProvider::Mayan | SwapperProvider::Relay => SlippageMode::Auto,
            SwapperProvider::UniswapV3
            | SwapperProvider::UniswapV4
            | SwapperProvider::PancakeswapV3
            | SwapperProvider::Panora
            | SwapperProvider::Jupiter
            | SwapperProvider::Oku
            | SwapperProvider::Wagmi
            | SwapperProvider::CetusClmm
            | SwapperProvider::StonfiV2
            | SwapperProvider::Aerodrome
            | SwapperProvider::Orca
            | SwapperProvider::RaydiumClmm
            | SwapperProvider::Hyperion
            | SwapperProvider::Dedust
            | SwapperProvider::PumpSwap
            | SwapperProvider::MeteoraDammV2
            | SwapperProvider::SunswapV3
            | SwapperProvider::StellarDex
            | SwapperProvider::DeepbookV3
            | SwapperProvider::Aero
            | SwapperProvider::Mayachain
            | SwapperProvider::Chainflip
            | SwapperProvider::Thorchain
            | SwapperProvider::NearIntents
            | SwapperProvider::Across
            | SwapperProvider::Hyperliquid
            | SwapperProvider::SwapsXyz
            | SwapperProvider::Bridgers => SlippageMode::Exact,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct QuoteRequest {
    pub from_asset: SwapperQuoteAsset,
    pub to_asset: SwapperQuoteAsset,
    pub wallet_address: String,
    pub destination_address: String,
    #[serde(serialize_with = "serialize_biguint")]
    pub value: BigUint,
    #[serde(skip_serializing)]
    pub options: Options,
}

impl QuoteRequest {
    pub fn value_u64(&self) -> Result<u64, SwapperError> {
        swap_value_u64(&self.value)
    }

    pub fn is_stablecoin_swap(&self) -> bool {
        STABLECOIN_ASSET_IDS.contains(&self.from_asset.asset_id()) && STABLECOIN_ASSET_IDS.contains(&self.to_asset.asset_id())
    }

    pub fn on_chain_referral_fee(&self) -> ReferralFee {
        on_chain_referral_fees(self).for_chain(self.from_asset.chain()).cloned().unwrap_or_default()
    }

    fn spends_all_native(&self) -> bool {
        self.options.use_max_amount && self.from_asset.is_native()
    }

    pub(crate) fn value_after_fee_reserve(&self) -> Result<BigUint, SwapperError> {
        match reserved_transaction_fees(self.from_asset.chain()) {
            Some(reserve) if self.spends_all_native() => (self.value > reserve).then(|| &self.value - &reserve).ok_or(SwapperError::InputAmountError { min_amount: Some(reserve.to_string()) }),
            _ => Ok(self.value.clone()),
        }
    }

    pub(crate) fn reserve_native_attachment(&self, attachment: &BigUint) -> Result<Self, SwapperError> {
        if !self.spends_all_native() {
            return Ok(self.clone());
        }
        if self.value <= *attachment {
            let reserved_fee = reserved_transaction_fees(self.from_asset.chain()).unwrap_or_default();
            return Err(SwapperError::InputAmountError {
                min_amount: Some((attachment + reserved_fee).to_string()),
            });
        }
        Ok(Self {
            value: &self.value - attachment,
            ..self.clone()
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Options {
    pub slippage: SwapperSlippage,
    pub use_max_amount: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            slippage: DEFAULT_SLIPPAGE_BPS.into(),
            use_max_amount: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Quote {
    #[serde(serialize_with = "serialize_biguint")]
    pub from_value: BigUint,
    #[serde(serialize_with = "serialize_option_biguint")]
    pub min_from_value: Option<BigUint>,
    #[serde(serialize_with = "serialize_biguint")]
    pub to_value: BigUint,
    pub data: ProviderData,
    pub request: QuoteRequest,
    pub eta_in_seconds: Option<u32>,
}

impl Quote {
    pub fn from_value_u64(&self) -> Result<u64, SwapperError> {
        swap_value_u64(&self.from_value)
    }

    pub fn from_route(request: &QuoteRequest, provider: &ProviderType, to_value: BigUint, slippage_bps: u32, route: &impl Serialize, eta_in_seconds: Option<u32>) -> Result<Self, SwapperError> {
        Ok(Self {
            from_value: request.value.clone(),
            min_from_value: None,
            to_value,
            data: ProviderData {
                provider: provider.clone(),
                routes: vec![Route {
                    input: request.from_asset.asset_id(),
                    output: request.to_asset.asset_id(),
                    route_data: serde_json::to_string(route)?,
                }],
                slippage_bps,
            },
            request: request.clone(),
            eta_in_seconds,
        })
    }

    pub fn with_provider(&self, provider: ProviderType) -> Self {
        Self {
            data: ProviderData { provider, ..self.data.clone() },
            ..self.clone()
        }
    }

    pub fn route_data<T: DeserializeOwned>(&self) -> Result<T, SwapperError> {
        let route = self.data.routes.first().ok_or(SwapperError::InvalidRoute)?;
        serde_json::from_str(&route.route_data).map_err(|_| SwapperError::InvalidRoute)
    }
}

pub(crate) fn nonzero_swap_value(value: &BigUint) -> Result<&BigUint, SwapperError> {
    match value == &BigUint::ZERO {
        true => Err(SwapperError::InputAmountError { min_amount: Some("1".into()) }),
        false => Ok(value),
    }
}

fn swap_value_u64(value: &BigUint) -> Result<u64, SwapperError> {
    nonzero_swap_value(value)?.to_u64().ok_or_else(|| SwapperError::ComputeQuoteError("swap amount is too large".into()))
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SwapQuotes {
    pub quotes: Vec<Quote>,
    pub errors: Vec<SwapQuoteError>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SwapQuoteError {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(serialize_with = "serialize_display")]
    pub error: SwapperError,
}

impl SwapQuoteError {
    pub fn new(provider: Option<String>, error: SwapperError) -> Self {
        Self { provider, error }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProviderData {
    pub provider: ProviderType,
    pub slippage_bps: u32,
    pub routes: Vec<Route>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Route {
    pub input: AssetId,
    pub output: AssetId,
    pub route_data: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FetchQuoteData {
    Permit2(Permit2Data),
    EstimateGas,
    None,
}

impl FetchQuoteData {
    pub fn permit2_data(&self) -> Option<Permit2Data> {
        match self {
            Self::Permit2(data) => Some(data.clone()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwapAmountMode {
    Fixed,
    Flexible,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SwapperChainAsset {
    All(Chain),
    Assets(Chain, Vec<AssetId>),
}

impl SwapperChainAsset {
    pub fn assets(chain: Chain, assets: impl IntoIterator<Item = AssetId>) -> Self {
        Self::Assets(chain, assets.into_iter().collect())
    }

    pub fn get_chain(&self) -> Chain {
        match self {
            Self::All(chain) => *chain,
            Self::Assets(chain, _) => *chain,
        }
    }

    pub fn supports(&self, asset_id: &AssetId) -> bool {
        match self {
            Self::All(chain) => *chain == asset_id.chain,
            Self::Assets(chain, assets) => *chain == asset_id.chain && (asset_id.is_native() || assets.contains(asset_id)),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssetList {
    pub chains: Vec<Chain>,
    pub asset_ids: Vec<AssetId>,
}

impl AssetList {
    pub fn contains(&self, asset_id: &AssetId) -> bool {
        self.chains.contains(&asset_id.chain) || self.asset_ids.contains(asset_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_u64() {
        let request = QuoteRequest::mock(Chain::Aptos, None);

        assert_eq!(
            QuoteRequest {
                value: BigUint::from(u64::MAX),
                ..request.clone()
            }
            .value_u64(),
            Ok(u64::MAX)
        );
        assert_eq!(QuoteRequest { value: BigUint::ZERO, ..request.clone() }.value_u64(), Err(SwapperError::InputAmountError { min_amount: Some("1".into()) }));
        assert_eq!(
            QuoteRequest {
                value: BigUint::from(u64::MAX) + 1u8,
                ..request
            }
            .value_u64(),
            Err(SwapperError::ComputeQuoteError("swap amount is too large".into()))
        );
    }

    #[test]
    fn test_value_after_fee_reserve() {
        let max = |asset: AssetId, value: u64| QuoteRequest {
            from_asset: SwapperQuoteAsset::from(asset),
            value: BigUint::from(value),
            options: Options { use_max_amount: true, ..Options::default() },
            ..QuoteRequest::mock(Chain::Solana, None)
        };
        let solana = AssetId::from_chain(Chain::Solana);
        let tron_usdt = AssetId::from_token(Chain::Tron, primitives::asset_constants::TRON_USDT_TOKEN_ID);

        assert_eq!(max(solana.clone(), 105_814_789).value_after_fee_reserve(), Ok(BigUint::from(100_814_789u64)));
        assert_eq!(max(AssetId::from_chain(Chain::Tron), 50_000_000).value_after_fee_reserve(), Ok(BigUint::from(48_000_000u64)));
        assert_eq!(max(tron_usdt, 50_000_000).value_after_fee_reserve(), Ok(BigUint::from(50_000_000u64)));
        assert_eq!(
            QuoteRequest {
                options: Options::default(),
                ..max(solana.clone(), 105_814_789)
            }
            .value_after_fee_reserve(),
            Ok(BigUint::from(105_814_789u64))
        );
        assert_eq!(max(solana, 5_000_000).value_after_fee_reserve(), Err(SwapperError::InputAmountError { min_amount: Some("5000000".to_string()) }));
    }

    #[test]
    fn test_reserve_native_attachment() {
        let attachment = BigUint::from(310_000_000u64);
        let request = QuoteRequest {
            value: BigUint::from(1_195_893_271u64),
            options: Options { use_max_amount: true, ..Options::default() },
            ..QuoteRequest::mock(Chain::Ton, None)
        };
        let small = QuoteRequest {
            value: attachment.clone(),
            ..request.clone()
        };

        assert_eq!(request.reserve_native_attachment(&attachment).unwrap().value, BigUint::from(885_893_271u64));
        assert_eq!(small.reserve_native_attachment(&attachment).unwrap_err(), SwapperError::InputAmountError { min_amount: Some("330000000".to_string()) });
        assert_eq!(QuoteRequest { options: Options::default(), ..small }.reserve_native_attachment(&attachment).unwrap().value, attachment);
        assert_eq!(QuoteRequest::mock(Chain::Ton, Some("token")).reserve_native_attachment(&attachment).unwrap().value, BigUint::from(1000000u64));
    }

    #[test]
    fn test_route_data() {
        let quote = Quote::mock(Chain::Aptos, None);
        let route = Route::mock(AssetId::from_chain(Chain::Aptos), AssetId::from_chain(Chain::Aptos));

        assert_eq!(
            Quote {
                data: ProviderData {
                    routes: vec![route.clone()],
                    ..quote.data.clone()
                },
                ..quote.clone()
            }
            .route_data::<serde_json::Value>(),
            Ok(serde_json::from_str(&route.route_data).unwrap())
        );
        assert_eq!(quote.route_data::<serde_json::Value>(), Err(SwapperError::InvalidRoute));
        assert_eq!(
            Quote {
                data: ProviderData {
                    routes: vec![Route { route_data: "{".into(), ..route }],
                    ..quote.data.clone()
                },
                ..quote
            }
            .route_data::<serde_json::Value>(),
            Err(SwapperError::InvalidRoute)
        );
    }

    #[test]
    fn test_chain_asset_supports() {
        let usdt = primitives::asset_constants::ETHEREUM_USDT_ASSET_ID.clone();
        let listed = SwapperChainAsset::Assets(Chain::Ethereum, vec![usdt.clone()]);
        let cardano = SwapperChainAsset::Assets(Chain::Cardano, vec![]);

        assert!(SwapperChainAsset::All(Chain::Ethereum).supports(&usdt));
        assert!(listed.supports(&usdt));
        assert!(listed.supports(&AssetId::from_chain(Chain::Ethereum)));
        assert!(cardano.supports(&AssetId::from_chain(Chain::Cardano)));
        assert!(!cardano.supports(&AssetId::from_token(Chain::Cardano, "policy.asset")));
        assert!(!listed.supports(&AssetId::from_chain(Chain::Arbitrum)));
    }
}
