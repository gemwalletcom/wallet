use std::collections::BTreeMap;
use std::sync::LazyLock;

use num_bigint::BigUint;
use num_traits::ToPrimitive;
use primitives::swap::SwapStatus;
use primitives::{AssetId, Chain, TransactionSwapMetadata, known_assets::*};
use serde::{Deserialize, Serialize};
use serde_serializers::{deserialize_biguint_from_str, serialize_biguint};

use crate::{SwapResult, SwapperChainAsset, SwapperProvider};

const STATE_CHAIN_BLOCK_SECONDS: f64 = 6.0;
const ORACLE_SLIPPAGE_DISABLE_BPS: u8 = 255;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteRequest {
    pub amount: String,
    pub src_chain: String,
    pub src_asset: String,
    pub dest_chain: String,
    pub dest_asset: String,
    pub is_vault_swap: bool,
    pub dca_enabled: bool,
    pub broker_commission_bps: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DcaParams {
    pub number_of_chunks: u32,
    pub chunk_interval_blocks: u32,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteResponse {
    #[serde(deserialize_with = "deserialize_biguint_from_str", serialize_with = "serialize_biguint")]
    pub egress_amount: BigUint,
    pub recommended_slippage_tolerance_percent: f64,
    #[serde(default)]
    pub recommended_live_price_slippage_tolerance_percent: Option<f64>,
    #[serde(default)]
    pub recommended_retry_duration_minutes: Option<f64>,
    pub estimated_duration_seconds: f64,
    #[serde(rename = "type")]
    pub quote_type: String,
    pub deposit_amount: String,
    pub is_vault_swap: bool,
    pub boost_quote: Option<BoostQuote>,
    pub estimated_price: String,
    pub dca_params: Option<DcaParams>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoostQuote {
    #[serde(deserialize_with = "deserialize_biguint_from_str", serialize_with = "serialize_biguint")]
    pub egress_amount: BigUint,
    pub recommended_slippage_tolerance_percent: f64,
    #[serde(default)]
    pub recommended_live_price_slippage_tolerance_percent: Option<f64>,
    #[serde(default)]
    pub recommended_retry_duration_minutes: Option<f64>,
    pub estimated_duration_seconds: f64,
    pub estimated_boost_fee_bps: u32,
    pub max_boost_fee_bps: u32,
    pub estimated_price: String,
    pub dca_params: Option<DcaParams>,
}

impl QuoteResponse {
    pub fn slippage_bps(&self) -> u32 {
        (self.recommended_slippage_tolerance_percent * 100.0) as u32
    }

    pub fn live_price_slippage_bps(&self) -> Option<u8> {
        recommended_live_price_slippage_bps(self.recommended_live_price_slippage_tolerance_percent)
    }

    pub fn retry_duration_blocks(&self) -> Option<u32> {
        recommended_retry_duration_blocks(self.recommended_retry_duration_minutes)
    }
}

impl BoostQuote {
    pub fn slippage_bps(&self) -> u32 {
        (self.recommended_slippage_tolerance_percent * 100.0) as u32
    }

    pub fn live_price_slippage_bps(&self) -> Option<u8> {
        recommended_live_price_slippage_bps(self.recommended_live_price_slippage_tolerance_percent)
    }

    pub fn retry_duration_blocks(&self) -> Option<u32> {
        recommended_retry_duration_blocks(self.recommended_retry_duration_minutes)
    }
}

fn recommended_live_price_slippage_bps(percent: Option<f64>) -> Option<u8> {
    let percent = percent?;
    if !percent.is_finite() || percent < 0.0 {
        return None;
    }

    let bps = (percent * 100.0).round() as u32;
    if bps < ORACLE_SLIPPAGE_DISABLE_BPS as u32 { Some(bps as u8) } else { None }
}

fn recommended_retry_duration_blocks(minutes: Option<f64>) -> Option<u32> {
    let minutes = minutes?;
    if !minutes.is_finite() || minutes <= 0.0 {
        return None;
    }

    Some((minutes * 60.0 / STATE_CHAIN_BLOCK_SECONDS).ceil() as u32)
}

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
}

impl SwapTxResponse {
    pub fn swap_status(&self) -> SwapStatus {
        match self.state.as_str() {
            "COMPLETED" if self.refund_egress.is_some() => SwapStatus::Failed,
            "COMPLETED" => SwapStatus::Completed,
            "FAILED" => SwapStatus::Failed,
            _ => SwapStatus::Pending,
        }
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
struct EstimatedDurations {
    deposit: Option<f64>,
    swap: Option<f64>,
    egress: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapDeposit {
    pub amount: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapDetail {
    pub swapped_output_amount: String,
}

fn chainflip_chain_to_chain(chain: &str) -> Option<Chain> {
    match chain {
        "Ethereum" => Some(Chain::Ethereum),
        "Bitcoin" => Some(Chain::Bitcoin),
        "Solana" => Some(Chain::Solana),
        "Arbitrum" => Some(Chain::Arbitrum),
        "Tron" => Some(Chain::Tron),
        _ => None,
    }
}

static ASSETS: LazyLock<Vec<(Chain, &'static str, AssetId)>> = LazyLock::new(|| {
    vec![
        (Chain::Bitcoin, "BTC", AssetId::from_chain(Chain::Bitcoin)),
        (Chain::Ethereum, "ETH", AssetId::from_chain(Chain::Ethereum)),
        (Chain::Ethereum, "USDC", ETHEREUM_USDC.id.clone()),
        (Chain::Ethereum, "USDT", ETHEREUM_USDT.id.clone()),
        (Chain::Ethereum, "WBTC", ETHEREUM_WBTC.id.clone()),
        (Chain::Ethereum, "FLIP", ETHEREUM_FLIP.id.clone()),
        (Chain::Solana, "SOL", AssetId::from_chain(Chain::Solana)),
        (Chain::Solana, "USDC", SOLANA_USDC.id.clone()),
        (Chain::Solana, "USDT", SOLANA_USDT.id.clone()),
        (Chain::Tron, "TRX", AssetId::from_chain(Chain::Tron)),
        (Chain::Tron, "USDT", TRON_USDT.id.clone()),
        (Chain::Arbitrum, "ETH", AssetId::from_chain(Chain::Arbitrum)),
        (Chain::Arbitrum, "USDC", ARBITRUM_USDC.id.clone()),
        (Chain::Arbitrum, "USDT", ARBITRUM_USDT.id.clone()),
    ]
});

pub static SUPPORTED_ASSETS: LazyLock<Vec<SwapperChainAsset>> = LazyLock::new(|| {
    let mut chains: BTreeMap<Chain, Vec<AssetId>> = BTreeMap::new();
    for (chain, _, asset_id) in ASSETS.iter() {
        let tokens = chains.entry(*chain).or_default();
        if asset_id.token_id.is_some() {
            tokens.push(asset_id.clone());
        }
    }
    chains.into_iter().map(|(chain, tokens)| SwapperChainAsset::Assets(chain, tokens)).collect()
});

fn chainflip_asset_to_asset_id(chain: Chain, asset: &str) -> Option<AssetId> {
    ASSETS.iter().find(|(c, s, _)| *c == chain && *s == asset).map(|(_, _, id)| id.clone())
}

pub fn map_swap_result(response: &SwapTxResponse) -> SwapResult {
    let status = response.swap_status();
    let eta_in_seconds = response.eta_in_seconds();

    let metadata = if status != SwapStatus::Pending {
        let from_chain = chainflip_chain_to_chain(&response.src_chain);
        let to_chain = chainflip_chain_to_chain(&response.dest_chain);

        from_chain.zip(to_chain).and_then(|(fc, tc)| {
            let from_asset = chainflip_asset_to_asset_id(fc, &response.src_asset)?;
            let to_asset = chainflip_asset_to_asset_id(tc, &response.dest_asset)?;
            let from_value = response.deposit.as_ref()?.amount.clone();
            let to_value = response.swap.as_ref().map(|s| s.swapped_output_amount.clone()).unwrap_or_default();
            Some(TransactionSwapMetadata {
                from_asset,
                from_value,
                to_asset,
                to_value,
                provider: Some(SwapperProvider::Chainflip.as_ref().to_string()),
            })
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
    pub fn get_quote_response() {
        let quote_response = serde_json::from_str::<Vec<QuoteResponse>>(include_str!("./test/btc_eth_quote.json")).unwrap();

        assert!(quote_response[0].boost_quote.is_some());
    }

    #[test]
    fn test_quote_recommendations_convert_to_broker_units() {
        let cases: [(f64, f64, Option<u8>, Option<u32>); 2] = [(1.0, 5.0, Some(100), Some(50)), (2.55, 0.0, None, None)];

        for (live_price_slippage_percent, retry_duration_minutes, expected_live_price_slippage_bps, expected_retry_duration_blocks) in cases {
            let quote_response = serde_json::from_value::<QuoteResponse>(serde_json::json!({
                "egressAmount": "1000",
                "recommendedSlippageTolerancePercent": 0.5,
                "recommendedLivePriceSlippageTolerancePercent": live_price_slippage_percent,
                "recommendedRetryDurationMinutes": retry_duration_minutes,
                "estimatedDurationSeconds": 60,
                "type": "REGULAR",
                "depositAmount": "100",
                "isVaultSwap": true,
                "estimatedPrice": "10"
            }))
            .unwrap();

            assert_eq!(quote_response.slippage_bps(), 50);
            assert_eq!(quote_response.live_price_slippage_bps(), expected_live_price_slippage_bps);
            assert_eq!(quote_response.retry_duration_blocks(), expected_retry_duration_blocks);
        }
    }

    #[test]
    fn test_map_swap_result_eth_to_btc() {
        assert_eq!(
            map_swap_result(&swap_response(include_str!("./test/swap_eth_to_btc.json"))),
            SwapResult {
                status: SwapStatus::Completed,
                metadata: Some(TransactionSwapMetadata {
                    from_asset: AssetId::from_chain(Chain::Ethereum),
                    from_value: "140000000000000000".to_string(),
                    to_asset: AssetId::from_chain(Chain::Bitcoin),
                    to_value: "405772".to_string(),
                    provider: Some("chainflip".to_string()),
                }),
                eta_in_seconds: None,
            }
        );
    }

    #[test]
    fn test_map_swap_result_usdc_to_sol() {
        assert_eq!(
            map_swap_result(&swap_response(include_str!("./test/swap_usdc_to_sol.json"))),
            SwapResult {
                status: SwapStatus::Completed,
                metadata: Some(TransactionSwapMetadata {
                    from_asset: ETHEREUM_USDC_ASSET_ID.clone(),
                    from_value: "100000000".to_string(),
                    to_asset: AssetId::from_chain(Chain::Solana),
                    to_value: "1143469990".to_string(),
                    provider: Some("chainflip".to_string()),
                }),
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
                metadata: Some(TransactionSwapMetadata {
                    from_asset: AssetId::from_chain(Chain::Solana),
                    from_value: "150000000".to_string(),
                    to_asset: AssetId::from_chain(Chain::Bitcoin),
                    to_value: "17567".to_string(),
                    provider: Some("chainflip".to_string()),
                }),
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
                status: SwapStatus::Failed,
                metadata: Some(TransactionSwapMetadata {
                    from_asset: AssetId::from_chain(Chain::Bitcoin),
                    from_value: "1508475".to_string(),
                    to_asset: ETHEREUM_USDT_ASSET_ID.clone(),
                    to_value: "0".to_string(),
                    provider: Some("chainflip".to_string()),
                }),
                eta_in_seconds: None,
            }
        );
    }
}
