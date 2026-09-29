use num_bigint::BigUint;
use serde_serializers::deserialize_biguint_from_str;
use std::collections::BTreeMap;
use std::str::FromStr;
use std::sync::LazyLock;

use num_traits::ToPrimitive;
use primitives::swap::SwapStatus;
use primitives::{AssetId, Chain, TransactionSwapMetadata, TransactionSwapReferralFee, known_assets::*, swap::CHAINFLIP_REFERRAL_ADDRESS};
use serde::{Deserialize, Serialize};

use crate::chainflip::{broker::ChainflipAsset, chain::ChainflipChain};
use crate::{SwapResult, SwapperChainAsset, SwapperError, SwapperProvider};

const BROKER_FEE_TYPE: &str = "BROKER";

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapTxResponse {
    pub state: String,
    pub src_asset: String,
    pub src_chain: String,
    pub dest_asset: String,
    pub dest_chain: String,
    pub deposit: Option<SwapDeposit>,
    pub swap: Option<SwapDetail>,
    pub refund_egress: Option<serde_json::Value>,
    estimated_durations_seconds: Option<EstimatedDurations>,
    #[serde(default)]
    brokers: Vec<SwapBroker>,
    #[serde(default)]
    fees: Vec<SwapFee>,
}

impl SwapTxResponse {
    pub fn swap_status(&self) -> SwapStatus {
        match self.state.as_str() {
            "COMPLETED" if self.refund_egress.is_some() => SwapStatus::Refunded,
            "COMPLETED" => SwapStatus::Completed,
            "FAILED" => SwapStatus::Failed,
            _ => SwapStatus::Pending,
        }
    }

    fn referral_fee(&self) -> Option<TransactionSwapReferralFee> {
        let broker = self.brokers.iter().find(|broker| broker.account == CHAINFLIP_REFERRAL_ADDRESS && broker.commission_bps > 0)?;
        let total_bps: u32 = self.brokers.iter().map(|broker| broker.commission_bps).sum();
        let fee = self.fees.iter().find(|fee| fee.fee_type == BROKER_FEE_TYPE)?;
        let chain = fee.chain.parse::<ChainflipChain>().ok()?.to_chain();
        Some(TransactionSwapReferralFee {
            asset_id: chainflip_asset_to_asset_id(chain, &fee.asset)?,
            value: &fee.amount * broker.commission_bps / total_bps,
        })
    }

    fn eta_in_seconds(&self) -> Option<u32> {
        let durations = self.estimated_durations_seconds.as_ref()?;
        let durations = [durations.deposit, durations.swap, durations.egress];
        let remaining = match self.state.as_str() {
            "WAITING" | "RECEIVING" => &durations[..],
            "SWAPPING" => &durations[1..],
            "SENDING" | "SENT" => &durations[2..],
            _ => return None,
        };

        remaining
            .iter()
            .copied()
            .try_fold(0.0, |total, duration| {
                let duration = duration?;
                if !duration.is_finite() || duration < 0.0 {
                    return None;
                }
                Some(total + duration)
            })?
            .ceil()
            .to_u32()
            .filter(|seconds| *seconds > 0)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SwapBroker {
    account: String,
    commission_bps: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SwapFee {
    chain: String,
    asset: String,
    #[serde(rename = "type")]
    fee_type: String,
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    amount: BigUint,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EstimatedDurations {
    deposit: Option<f64>,
    swap: Option<f64>,
    egress: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapDeposit {
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    pub amount: BigUint,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapDetail {
    pub swapped_output_amount: String,
}

static ASSETS: LazyLock<Vec<(&'static str, AssetId)>> = LazyLock::new(|| {
    vec![
        ("BTC", AssetId::from_chain(Chain::Bitcoin)),
        ("ETH", AssetId::from_chain(Chain::Ethereum)),
        ("USDC", ETHEREUM_USDC.id.clone()),
        ("USDT", ETHEREUM_USDT.id.clone()),
        ("WBTC", ETHEREUM_WBTC.id.clone()),
        ("CBBTC", ETHEREUM_CBBTC.id.clone()),
        ("FLIP", ETHEREUM_FLIP.id.clone()),
        ("BNB", AssetId::from_chain(Chain::SmartChain)),
        ("USDT", SMARTCHAIN_USDT.id.clone()),
        ("SOL", AssetId::from_chain(Chain::Solana)),
        ("USDC", SOLANA_USDC.id.clone()),
        ("USDT", SOLANA_USDT.id.clone()),
        ("TRX", AssetId::from_chain(Chain::Tron)),
        ("USDT", TRON_USDT.id.clone()),
        ("ETH", AssetId::from_chain(Chain::Arbitrum)),
        ("USDC", ARBITRUM_USDC.id.clone()),
        ("USDT", ARBITRUM_USDT.id.clone()),
    ]
});

impl ChainflipAsset {
    pub(crate) fn from_asset_id(asset_id: &AssetId) -> Result<Self, SwapperError> {
        let (symbol, _) = ASSETS.iter().find(|(_, id)| id == asset_id).ok_or(SwapperError::NotSupportedAsset)?;
        let chain = ChainflipChain::from_chain(asset_id.chain).ok_or(SwapperError::NotSupportedChain)?;
        Ok(Self {
            chain: chain.as_ref().to_string(),
            asset: symbol.to_string(),
        })
    }
}

pub static SUPPORTED_ASSETS: LazyLock<Vec<SwapperChainAsset>> = LazyLock::new(|| {
    let mut chains: BTreeMap<Chain, Vec<AssetId>> = BTreeMap::new();
    for (_, asset_id) in ASSETS.iter() {
        let tokens = chains.entry(asset_id.chain).or_default();
        if asset_id.token_id.is_some() {
            tokens.push(asset_id.clone());
        }
    }
    chains.into_iter().map(|(chain, tokens)| SwapperChainAsset::Assets(chain, tokens)).collect()
});

fn chainflip_asset_to_asset_id(chain: Chain, asset: &str) -> Option<AssetId> {
    ASSETS.iter().find(|(symbol, id)| id.chain == chain && *symbol == asset).map(|(_, id)| id.clone())
}

pub fn map_swap_result(response: &SwapTxResponse) -> SwapResult {
    let status = response.swap_status();
    let eta_in_seconds = response.eta_in_seconds();

    let metadata = if status != SwapStatus::Pending {
        let from_chain = response.src_chain.parse::<ChainflipChain>().ok().map(ChainflipChain::to_chain);
        let to_chain = response.dest_chain.parse::<ChainflipChain>().ok().map(ChainflipChain::to_chain);

        from_chain.zip(to_chain).and_then(|(fc, tc)| {
            let from_asset = chainflip_asset_to_asset_id(fc, &response.src_asset)?;
            let to_asset = chainflip_asset_to_asset_id(tc, &response.dest_asset)?;
            let from_value = response.deposit.as_ref()?.amount.clone();
            let to_value = response.swap.as_ref().map(|swap| BigUint::from_str(&swap.swapped_output_amount)).transpose().ok()?.unwrap_or_default();
            Some(TransactionSwapMetadata::new(from_asset, from_value, to_asset, to_value, SwapperProvider::Chainflip).with_referral_fee(response.referral_fee()))
        })
    } else {
        None
    };

    SwapResult { status, metadata, eta_in_seconds }
}

#[cfg(test)]
pub mod test {
    use super::*;
    use primitives::{
        AssetId,
        asset_constants::{ETHEREUM_USDC_ASSET_ID, ETHEREUM_USDT_ASSET_ID},
    };

    fn swap_response(json: &str) -> SwapTxResponse {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn test_map_swap_result_eth_to_btc() {
        assert_eq!(
            map_swap_result(&swap_response(include_str!("./test/swap_eth_to_btc.json"))),
            SwapResult {
                status: SwapStatus::Completed,
                metadata: Some(TransactionSwapMetadata::new(
                    AssetId::from_chain(Chain::Ethereum),
                    BigUint::from(140000000000000000u64),
                    AssetId::from_chain(Chain::Bitcoin),
                    BigUint::from(405772u64),
                    SwapperProvider::Chainflip
                )),
                eta_in_seconds: None,
            }
        );
    }

    #[test]
    fn test_map_swap_result_broker_referral_fee() {
        let metadata = map_swap_result(&swap_response(include_str!("./test/swap_sol_to_btc_broker_fee.json"))).metadata.unwrap();

        assert_eq!(metadata.from_asset, AssetId::from_chain(Chain::Solana));
        assert_eq!(metadata.from_value, BigUint::from(150000000u64));
        assert_eq!(metadata.to_asset, AssetId::from_chain(Chain::Bitcoin));
        assert_eq!(metadata.to_value, BigUint::from(17567u64));
        assert_eq!(
            metadata.referral_fee,
            Some(TransactionSwapReferralFee {
                asset_id: ETHEREUM_USDC_ASSET_ID.clone(),
                value: BigUint::from(50873u64),
            })
        );
    }

    #[test]
    fn test_map_swap_result_usdc_to_sol() {
        assert_eq!(
            map_swap_result(&swap_response(include_str!("./test/swap_usdc_to_sol.json"))),
            SwapResult {
                status: SwapStatus::Completed,
                metadata: Some(TransactionSwapMetadata::new(
                    ETHEREUM_USDC_ASSET_ID.clone(),
                    BigUint::from(100000000u64),
                    AssetId::from_chain(Chain::Solana),
                    BigUint::from(1143469990u64),
                    SwapperProvider::Chainflip
                )),
                eta_in_seconds: None,
            }
        );
    }

    #[test]
    fn test_map_swap_result_sol_to_btc() {
        assert_eq!(
            map_swap_result(&swap_response(include_str!("./test/swap_sol_to_btc.json"))),
            SwapResult {
                status: SwapStatus::Completed,
                metadata: Some(TransactionSwapMetadata::new(
                    AssetId::from_chain(Chain::Solana),
                    BigUint::from(150000000u64),
                    AssetId::from_chain(Chain::Bitcoin),
                    BigUint::from(17567u64),
                    SwapperProvider::Chainflip
                )),
                eta_in_seconds: None,
            }
        );
    }

    #[test]
    fn test_map_swap_result_pending_eta_by_phase() {
        let sending = map_swap_result(&swap_response(include_str!("./test/swap_usdc_to_btc_pending.json")));
        let receiving = map_swap_result(&swap_response(include_str!("./test/swap_usdc_to_btc_receiving_eta.json")));
        let swapping = map_swap_result(&swap_response(include_str!("./test/swap_usdc_to_btc_swapping_eta.json")));

        assert_eq!(sending.status, SwapStatus::Pending);
        assert!(sending.metadata.is_none());
        assert_eq!(sending.eta_in_seconds, Some(102));
        assert_eq!(receiving.eta_in_seconds, Some(1920));
        assert_eq!(swapping.eta_in_seconds, Some(114));
    }

    #[test]
    fn test_eta_duration_validation() {
        let mut fractional = swap_response(include_str!("./test/swap_usdc_to_btc_receiving_eta.json"));
        fractional.estimated_durations_seconds = Some(EstimatedDurations {
            deposit: Some(0.4),
            swap: Some(0.4),
            egress: Some(0.4),
        });
        assert_eq!(fractional.eta_in_seconds(), Some(2));

        for invalid in [-1.0, f64::NAN, f64::INFINITY, u32::MAX as f64 + 1.0] {
            let mut response = swap_response(include_str!("./test/swap_usdc_to_btc_receiving_eta.json"));
            response.estimated_durations_seconds.as_mut().unwrap().deposit = Some(invalid);
            assert_eq!(response.eta_in_seconds(), None);
        }

        let mut zero = swap_response(include_str!("./test/swap_usdc_to_btc_receiving_eta.json"));
        zero.estimated_durations_seconds = Some(EstimatedDurations {
            deposit: Some(0.0),
            swap: Some(0.0),
            egress: Some(0.0),
        });
        assert_eq!(zero.eta_in_seconds(), None);

        let mut terminal = swap_response(include_str!("./test/swap_usdc_to_btc_pending.json"));
        terminal.state = "COMPLETED".to_string();
        assert_eq!(terminal.eta_in_seconds(), None);

        let mut overflow = swap_response(include_str!("./test/swap_usdc_to_btc_receiving_eta.json"));
        let durations = overflow.estimated_durations_seconds.as_mut().unwrap();
        durations.deposit = Some(u32::MAX as f64);
        durations.swap = Some(1.0);
        durations.egress = Some(1.0);
        assert_eq!(overflow.eta_in_seconds(), None);

        let mut missing = swap_response(include_str!("./test/swap_usdc_to_btc_receiving_eta.json"));
        missing.estimated_durations_seconds.as_mut().unwrap().deposit = None;
        assert_eq!(missing.eta_in_seconds(), None);
        missing.estimated_durations_seconds = None;
        assert_eq!(missing.eta_in_seconds(), None);
    }

    #[test]
    fn test_map_swap_result_refunded() {
        assert_eq!(
            map_swap_result(&swap_response(include_str!("./test/swap_btc_to_usdt_refunded.json"))),
            SwapResult {
                status: SwapStatus::Refunded,
                metadata: Some(TransactionSwapMetadata::new(
                    AssetId::from_chain(Chain::Bitcoin),
                    BigUint::from(1508475u64),
                    ETHEREUM_USDT_ASSET_ID.clone(),
                    BigUint::from(0u64),
                    SwapperProvider::Chainflip
                )),
                eta_in_seconds: None,
            }
        );
    }
}
