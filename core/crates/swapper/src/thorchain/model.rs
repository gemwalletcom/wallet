use num_bigint::BigInt;
use num_bigint::BigUint;
use primitives::{Asset, AssetId, Chain, swap::SwapStatus};
use serde::{Deserialize, Serialize};
use serde_serializers::deserialize_bigint_from_str;

use super::{
    SIGNING_PERIOD_BLOCKS, THORCHAIN_BLOCK_SECONDS, TOKEN_SIGNING_PERIOD_BLOCKS,
    asset::{THORChainAsset, value_to},
    chain::ChainName,
    constants::{THORCHAIN_INBOUND_ADDRESS, ZERO_HASH},
    quote_mapper,
};
use crate::{SwapperError, error::ProviderErrorResponse};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteSwapRequest {
    pub from_asset: String,
    pub to_asset: String,
    pub amount: String,
    pub affiliate: String,
    pub affiliate_bps: i64,
    pub streaming_interval: i64,
    pub streaming_quantity: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteSwapResponse {
    pub expected_amount_out: String,
    #[serde(deserialize_with = "deserialize_bigint_from_str")]
    pub recommended_min_amount_in: BigInt,
    pub inbound_address: Option<String>,
    pub router: Option<String>,
    pub total_swap_seconds: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionStatus {
    pub tx: Option<TransactionStatusTx>,
    pub stages: TransactionStages,
    pub planned_out_txs: Option<Vec<PlannedOutTx>>,
    pub out_txs: Option<Vec<TransactionStatusOutTx>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedOutTx {
    pub chain: String,
    pub coin: TransactionCoin,
    pub refund: bool,
}

impl PlannedOutTx {
    fn external_chain(&self) -> Option<ChainName> {
        ChainName::from_symbol(&self.chain).filter(|name| name.chain() != Chain::Thorchain)
    }

    fn signing_period_blocks(&self, chain: ChainName) -> u32 {
        if chain.is_evm_chain() && self.coin.asset_id().is_some_and(|asset_id| asset_id.token_id.is_some()) {
            TOKEN_SIGNING_PERIOD_BLOCKS
        } else {
            SIGNING_PERIOD_BLOCKS
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionStatusTx {
    pub chain: String,
    pub memo: String,
    pub coins: Vec<TransactionCoin>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionCoin {
    pub asset: String,
    pub amount: String,
    pub decimals: Option<u32>,
}

impl TransactionCoin {
    pub fn native_value(&self) -> Option<BigUint> {
        let decimals = match self.decimals {
            Some(decimals) => decimals,
            None => THORChainAsset::from_id(&self.asset_id()?)?.decimals,
        };
        Some(value_to(&self.amount, decimals).magnitude().clone())
    }

    pub fn asset_id(&self) -> Option<AssetId> {
        let (chain_symbol, asset_symbol) = self.asset.split_once('.')?;
        let chain_name = ChainName::from_symbol(chain_symbol)?;
        let chain = chain_name.chain();
        let native_asset = Asset::from_chain(chain);
        if asset_symbol == native_asset.symbol {
            return Some(native_asset.id);
        }

        let Some((symbol, token_id)) = asset_symbol.split_once('-') else {
            return chain_name.token_assets().into_iter().find(|asset| asset.symbol == asset_symbol).map(|asset| asset.id);
        };
        let token_id = chain_name.checksum_address(token_id);

        match THORChainAsset::from(chain_name, &token_id).and_then(|a| a.token_id) {
            Some(token_id) => Some(AssetId::from_token(chain, &token_id)),
            None if chain == Chain::Tron => chain_name.token_assets().into_iter().find(|asset| asset.symbol == symbol).map(|asset| asset.id),
            None => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionStages {
    inbound_confirmation_counted: Option<InboundConfirmationStage>,
    pub swap_status: Option<TransactionSwapStage>,
    outbound_delay: Option<OutboundDelayStage>,
    pub outbound_signed: Option<OutboundSignedStage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InboundConfirmationStage {
    remaining_confirmation_seconds: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionSwapStage {
    pub pending: bool,
    streaming: Option<StreamingSwapStage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StreamingSwapStage {
    interval: u32,
    quantity: u32,
    count: u32,
}

impl StreamingSwapStage {
    fn remaining_seconds(&self) -> u32 {
        self.quantity.saturating_sub(self.count).saturating_mul(self.interval).saturating_mul(THORCHAIN_BLOCK_SECONDS)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboundSignedStage {
    pub completed: bool,
    blocks_since_scheduled: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OutboundDelayStage {
    remaining_delay_seconds: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionStatusOutTx {
    pub id: String,
    pub chain: String,
    #[serde(default)]
    pub to_address: Option<String>,
    pub coins: Vec<TransactionCoin>,
}

impl TransactionStatus {
    pub fn swap_status(&self) -> SwapStatus {
        let has_output = self.out_txs.as_ref().is_some_and(|transactions| !transactions.is_empty());
        let swap_done = self.stages.swap_status.as_ref().is_some_and(|s| !s.pending);
        let outbound_done = self.stages.outbound_signed.as_ref().is_none_or(|s| s.completed);

        if !(swap_done && has_output && outbound_done) {
            SwapStatus::Pending
        } else if self.is_refunded() {
            SwapStatus::Refunded
        } else {
            SwapStatus::Completed
        }
    }

    pub fn destination_coin(&self) -> Option<&TransactionCoin> {
        if self.is_refunded() {
            return None;
        }

        let real_out = self
            .out_txs
            .as_ref()
            .and_then(|transactions| transactions.iter().find(|x| x.id != ZERO_HASH && !x.id.is_empty()))
            .and_then(|transaction| transaction.coins.first());
        if real_out.is_some() {
            return real_out;
        }
        let planned = self.planned_out_txs.as_ref().and_then(|transactions| transactions.iter().find(|t| !t.refund)).map(|t| &t.coin);
        if planned.is_some() {
            return planned;
        }
        self.out_txs.as_ref().and_then(|transactions| transactions.first()).and_then(|transaction| transaction.coins.first())
    }

    pub(super) fn eta_in_seconds(&self) -> Option<u32> {
        if self.swap_status() != SwapStatus::Pending {
            return None;
        }

        let confirmation = self.stages.inbound_confirmation_counted.as_ref().and_then(|stage| stage.remaining_confirmation_seconds).unwrap_or_default();
        let streaming = self.stages.swap_status.as_ref().and_then(|stage| stage.streaming.as_ref()).map_or(0, StreamingSwapStage::remaining_seconds);
        let delay = self.stages.outbound_delay.as_ref().and_then(|stage| stage.remaining_delay_seconds).unwrap_or_default();
        let payout = self.payout_seconds().unwrap_or_default();
        [confirmation, streaming, delay, payout].into_iter().try_fold(0, u32::checked_add).filter(|seconds| *seconds > 0)
    }

    fn payout_seconds(&self) -> Option<u32> {
        let stage = self.stages.outbound_signed.as_ref().filter(|stage| !stage.completed)?;
        let (payout, chain) = self.planned_out_txs.iter().flatten().filter_map(|tx| Some((tx, tx.external_chain()?))).min_by_key(|(tx, _)| tx.refund)?;
        let allowance = quote_mapper::outbound_seconds(chain.chain());
        let Some(blocks) = stage.blocks_since_scheduled else {
            return Some(allowance);
        };
        let period = payout.signing_period_blocks(chain);
        let assigned_blocks = blocks % period;
        allowance
            .checked_sub(assigned_blocks.saturating_mul(THORCHAIN_BLOCK_SECONDS))
            .filter(|seconds| *seconds > 0)
            .or_else(|| (period - assigned_blocks).checked_mul(THORCHAIN_BLOCK_SECONDS)?.checked_add(allowance))
    }

    fn is_refunded(&self) -> bool {
        self.planned_out_txs.as_ref().is_some_and(|transactions| !transactions.is_empty() && transactions.iter().all(|transaction| transaction.refund))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteData {
    pub router_address: Option<String>,
    pub inbound_address: String,
}

impl RouteData {
    pub fn get_inbound_address(from_asset: &THORChainAsset, quote_inbound_address: Option<String>) -> Result<String, SwapperError> {
        if from_asset.chain.chain() == Chain::Thorchain {
            Ok(THORCHAIN_INBOUND_ADDRESS.to_string())
        } else {
            quote_inbound_address.ok_or(SwapperError::InvalidRoute)
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct InboundAddress {
    pub chain: String,
    pub address: String,
    pub router: Option<String>,
    pub halted: bool,
    pub global_trading_paused: Option<bool>,
    pub chain_trading_paused: Option<bool>,
    #[serde(deserialize_with = "deserialize_bigint_from_str")]
    pub dust_threshold: BigInt,
}

impl InboundAddress {
    pub(crate) fn is_swap_available(&self) -> bool {
        !self.halted && self.global_trading_paused != Some(true) && self.chain_trading_paused != Some(true)
    }
}

pub(crate) trait InboundAddressesExt {
    fn inbound_address_for_asset(&self, asset: &THORChainAsset) -> Result<Option<&InboundAddress>, SwapperError>;
}

impl InboundAddressesExt for [InboundAddress] {
    fn inbound_address_for_asset(&self, asset: &THORChainAsset) -> Result<Option<&InboundAddress>, SwapperError> {
        if asset.chain.chain() == Chain::Thorchain {
            return Ok(None);
        }

        self.iter().find(|address| address.chain == asset.chain.long_name()).map(Some).ok_or(SwapperError::InvalidRoute)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct AsgardVault {
    pub addresses: Vec<VaultAddress>,
    pub routers: Vec<VaultRouter>,
}

impl AsgardVault {
    pub fn all_addresses(vaults: &[AsgardVault]) -> Vec<String> {
        vaults
            .iter()
            .flat_map(|vault| {
                let addrs = vault.addresses.iter().filter_map(|a| {
                    let chain = ChainName::from_symbol(&a.chain)?;
                    Some(chain.checksum_address(&a.address))
                });
                let routers = vault.routers.iter().filter_map(|r| {
                    let chain = ChainName::from_symbol(&r.chain)?;
                    Some(chain.checksum_address(&r.router))
                });
                addrs.chain(routers)
            })
            .collect()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct VaultAddress {
    pub chain: String,
    pub address: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VaultRouter {
    pub chain: String,
    pub router: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ErrorResponse {
    pub message: String,
}

impl ErrorResponse {
    const MIN_AMOUNT_PREFIX: &str = "recommended_min_amount_in: ";
    const DUST_THRESHOLD_MSG: &str = "amount less than dust threshold";
    const MIN_SWAP_AMOUNT_MSG: &str = "amount less than min swap amount";

    pub fn is_input_amount_error(&self) -> bool {
        self.message.contains(Self::DUST_THRESHOLD_MSG) || self.message.contains(Self::MIN_SWAP_AMOUNT_MSG)
    }

    pub fn parse_min_amount(&self) -> Option<String> {
        self.message
            .find(Self::MIN_AMOUNT_PREFIX)
            .map(|start| self.message[start + Self::MIN_AMOUNT_PREFIX.len()..].chars().take_while(char::is_ascii_digit).collect())
            .filter(|s: &String| !s.is_empty())
    }
}

impl ProviderErrorResponse for ErrorResponse {
    fn into_swapper_error(self) -> Option<SwapperError> {
        self.is_input_amount_error().then(|| SwapperError::InputAmountError { min_amount: self.parse_min_amount() })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gem_client::ClientError;
    use primitives::asset_constants::{ETHEREUM_USDT_ASSET_ID, ETHEREUM_USDT_TOKEN_ID, THORCHAIN_TCY_ASSET_ID, TRON_USDT_ASSET_ID, TRON_USDT_TOKEN_ID};

    #[test]
    fn test_input_amount_error() {
        let error = ClientError::Http {
            status: 400,
            body: br#"{"code":3,"message":"amount less than min swap amount, recommended_min_amount_in: 100000","details":[]}"#.to_vec(),
        }
        .decode_body::<ErrorResponse>();
        assert_eq!(SwapperError::from(error), SwapperError::InputAmountError { min_amount: Some("100000".to_string()) });
    }

    #[test]
    fn test_tx_status_completed_ltc_to_tron() {
        let status: TransactionStatus = serde_json::from_str(include_str!("testdata/tx_status_ltc_to_tron_usdt.json")).unwrap();
        assert_eq!(status.swap_status(), SwapStatus::Completed);
        assert_eq!(status.destination_coin().unwrap().amount, "7915842900");
    }

    #[test]
    fn test_tx_status_pending_btc_to_tron() {
        let status: TransactionStatus = serde_json::from_str(include_str!("testdata/tx_status_btc_to_tron_pending.json")).unwrap();
        assert_eq!(status.swap_status(), SwapStatus::Pending);
        assert!(status.destination_coin().is_none());
    }

    #[test]
    fn test_tx_status_completed_tcy_to_eth_usdt() {
        let status: TransactionStatus = serde_json::from_str(include_str!("testdata/tx_status_tcy_to_eth_usdt.json")).unwrap();
        assert_eq!(status.swap_status(), SwapStatus::Completed);
        assert_eq!(status.destination_coin().unwrap().amount, "380962656200");
    }

    #[test]
    fn test_tx_status_refunded_btc_to_eth_usdc() {
        let status: TransactionStatus = serde_json::from_str(include_str!("testdata/tx_status_btc_to_eth_usdc_refund.json")).unwrap();
        assert_eq!(status.swap_status(), SwapStatus::Refunded);
        assert!(status.destination_coin().is_none());

        let scheduled = TransactionStatus {
            out_txs: None,
            stages: TransactionStages {
                outbound_signed: Some(OutboundSignedStage {
                    completed: false,
                    blocks_since_scheduled: Some(5),
                }),
                ..status.stages.clone()
            },
            ..status.clone()
        };
        assert_eq!(scheduled.swap_status(), SwapStatus::Pending);
        assert!(scheduled.destination_coin().is_none());
        assert_eq!(scheduled.eta_in_seconds(), Some(630));
    }

    #[test]
    fn test_inbound_addresses() {
        let inbound_addresses: Vec<InboundAddress> = serde_json::from_str(include_str!("testdata/inbound_addresses_bsc_halted.json")).unwrap();
        let bsc = THORChainAsset::from_asset_id(Chain::SmartChain.as_ref()).unwrap();
        let bitcoin = THORChainAsset::from_asset_id(Chain::Bitcoin.as_ref()).unwrap();
        let rune = THORChainAsset::from_asset_id(Chain::Thorchain.as_ref()).unwrap();
        let ethereum = THORChainAsset::from_asset_id(Chain::Ethereum.as_ref()).unwrap();

        let bsc_address = inbound_addresses.inbound_address_for_asset(&bsc).unwrap().unwrap();
        let bitcoin_address = inbound_addresses.inbound_address_for_asset(&bitcoin).unwrap().unwrap();

        assert_eq!(bsc_address.chain, "BSC");
        assert!(!bsc_address.is_swap_available());
        assert_eq!(bitcoin_address.chain, "BTC");
        assert!(bitcoin_address.is_swap_available());
        assert!(!InboundAddress { halted: true, ..bitcoin_address.clone() }.is_swap_available());
        assert!(
            !InboundAddress {
                global_trading_paused: Some(true),
                ..bitcoin_address.clone()
            }
            .is_swap_available()
        );
        assert!(
            !InboundAddress {
                chain_trading_paused: Some(true),
                ..bitcoin_address.clone()
            }
            .is_swap_available()
        );
        assert!(inbound_addresses.inbound_address_for_asset(&rune).unwrap().is_none());
        assert_eq!(inbound_addresses.inbound_address_for_asset(&ethereum).unwrap_err(), SwapperError::InvalidRoute);
    }

    #[test]
    fn test_get_inbound_address_thorchain() {
        let from_asset = THORChainAsset::from_asset_id("thorchain").unwrap();
        let result = RouteData::get_inbound_address(&from_asset, None);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), THORCHAIN_INBOUND_ADDRESS);
    }

    #[test]
    fn test_get_inbound_address_other_chain() {
        let from_asset = THORChainAsset::from_asset_id("ethereum").unwrap();
        let quote_address = "0x1234567890abcdef".to_string();
        let result = RouteData::get_inbound_address(&from_asset, Some(quote_address.clone()));

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), quote_address);
    }

    #[test]
    fn test_native_value() {
        let native = TransactionCoin {
            asset: "LTC.LTC".to_string(),
            amount: "160661010".to_string(),
            decimals: None,
        };
        assert_eq!(native.native_value(), Some(BigUint::from(160661010u64)));

        let native_18 = TransactionCoin {
            asset: "ETH.ETH".to_string(),
            amount: "2509674".to_string(),
            decimals: None,
        };
        assert_eq!(native_18.native_value(), Some(BigUint::from(25096740000000000u64)));

        let token_with_decimals = TransactionCoin {
            asset: format!("ETH.USDT-{ETHEREUM_USDT_TOKEN_ID}"),
            amount: "380962656200".to_string(),
            decimals: Some(6),
        };
        assert_eq!(token_with_decimals.native_value(), Some(BigUint::from(3809626562u64)));

        let token_no_decimals = TransactionCoin {
            asset: format!("ETH.USDT-{ETHEREUM_USDT_TOKEN_ID}"),
            amount: "380962656200".to_string(),
            decimals: None,
        };
        assert_eq!(token_no_decimals.native_value(), Some(BigUint::from(3809626562u64)));

        let unknown_token = TransactionCoin {
            asset: "ETH.UNKNOWN-0X0000000000000000000000000000000000000001".to_string(),
            amount: "380962656200".to_string(),
            decimals: None,
        };
        assert_eq!(unknown_token.native_value(), None);
    }

    #[test]
    fn test_tx_status_completed_eth_usdt_to_rune() {
        let status: TransactionStatus = serde_json::from_str(include_str!("testdata/tx_status_eth_usdt_to_rune.json")).unwrap();
        assert_eq!(status.swap_status(), SwapStatus::Completed);
        assert_eq!(status.destination_coin().unwrap().amount, "2096315169517");
    }

    #[test]
    fn test_tx_status_completed_bnb_to_tron() {
        let status: TransactionStatus = serde_json::from_str(include_str!("testdata/tx_status_bnb_to_tron.json")).unwrap();
        assert_eq!(status.swap_status(), SwapStatus::Completed);
        assert_eq!(status.destination_coin().unwrap().amount, "4307055600");
    }

    #[test]
    fn test_transaction_status_eta() {
        let mut status: TransactionStatus = serde_json::from_str(include_str!("testdata/tx_status_bnb_to_tron_pending.json")).unwrap();
        assert_eq!(status.swap_status(), SwapStatus::Pending);
        assert_eq!(status.eta_in_seconds(), Some(183));

        status.stages.swap_status = Some(TransactionSwapStage {
            pending: true,
            streaming: Some(StreamingSwapStage { interval: 2, quantity: 10, count: 4 }),
        });
        assert_eq!(status.eta_in_seconds(), Some(255));

        status.stages.swap_status = Some(TransactionSwapStage { pending: false, streaming: None });
        status.stages.inbound_confirmation_counted = None;
        status.stages.outbound_delay = None;
        assert_eq!(status.eta_in_seconds(), Some(63));

        status.planned_out_txs = None;
        assert_eq!(status.eta_in_seconds(), None);

        status.stages.inbound_confirmation_counted = Some(InboundConfirmationStage { remaining_confirmation_seconds: Some(0) });
        status.stages.outbound_delay = Some(OutboundDelayStage { remaining_delay_seconds: Some(0) });
        assert_eq!(status.eta_in_seconds(), None);

        status.stages.inbound_confirmation_counted = Some(InboundConfirmationStage {
            remaining_confirmation_seconds: Some(u32::MAX),
        });
        status.stages.outbound_delay = Some(OutboundDelayStage { remaining_delay_seconds: Some(1) });
        assert_eq!(status.eta_in_seconds(), None);

        let mut status: TransactionStatus = serde_json::from_str(include_str!("testdata/tx_status_btc_to_eth_usdc_outbound_late.json")).unwrap();
        assert_eq!(status.swap_status(), SwapStatus::Pending);
        assert_eq!(status.eta_in_seconds(), Some(1380));

        for (blocks_since_scheduled, eta) in [(None, 72), (Some(0), 72), (Some(5), 42), (Some(12), 5400), (Some(400), 3072), (Some(905), 42)] {
            status.stages.outbound_signed = Some(OutboundSignedStage { completed: false, blocks_since_scheduled });
            assert_eq!(status.eta_in_seconds(), Some(eta), "{blocks_since_scheduled:?}");
        }

        status.stages.outbound_signed = Some(OutboundSignedStage {
            completed: false,
            blocks_since_scheduled: Some(400),
        });
        status.planned_out_txs.as_mut().unwrap()[0].coin.asset = "ETH.ETH".to_string();
        assert_eq!(status.eta_in_seconds(), Some(1272));
    }

    #[test]
    fn test_tx_status_not_observed() {
        let json = r#"{"stages":{"inbound_observed":{"started":false,"final_count":0,"completed":false}}}"#;
        let status: TransactionStatus = serde_json::from_str(json).unwrap();

        assert_eq!(status.swap_status(), SwapStatus::Pending);
        assert!(status.tx.is_none());
        assert!(status.out_txs.is_none());
    }

    #[test]
    fn test_asset_id() {
        fn coin(asset: &str) -> TransactionCoin {
            TransactionCoin {
                asset: asset.to_string(),
                amount: "0".to_string(),
                decimals: None,
            }
        }

        assert_eq!(coin("LTC.LTC").asset_id(), Some(Chain::Litecoin.as_asset_id()));
        assert_eq!(coin("ETH.ETH").asset_id(), Some(Chain::Ethereum.as_asset_id()));
        assert_eq!(coin("BTC.BTC").asset_id(), Some(Chain::Bitcoin.as_asset_id()));
        assert_eq!(coin("THOR.RUNE").asset_id(), Some(Chain::Thorchain.as_asset_id()));
        assert_eq!(coin("ZEC.ZEC").asset_id(), Some(Chain::Zcash.as_asset_id()));
        assert_eq!(coin("ARB.ETH").asset_id(), None);
        assert_eq!(coin("ETH.USDT-0XDAC17F958D2EE523A2206206994597C13D831EC7").asset_id(), Some(ETHEREUM_USDT_ASSET_ID.clone()));
        assert_eq!(coin(&format!("TRON.USDT-{TRON_USDT_TOKEN_ID}")).asset_id(), Some(TRON_USDT_ASSET_ID.clone()));
        assert_eq!(coin("THOR.TCY").asset_id(), Some(THORCHAIN_TCY_ASSET_ID.clone()));
        assert_eq!(coin("ETH.UNKNOWN-0x1234567890abcdef1234567890abcdef12345678").asset_id(), None);
        assert_eq!(coin("INVALID").asset_id(), None);
    }

    #[test]
    fn test_asgard_vaults_deserialization() {
        let vaults: Vec<AsgardVault> = serde_json::from_str(include_str!("testdata/asgard_vaults.json")).unwrap();
        let addresses = AsgardVault::all_addresses(&vaults);
        assert_eq!(vaults.len(), 2);
        assert!(!addresses.is_empty());
        assert_eq!(vaults[0].addresses.len(), 12);
        assert_eq!(vaults[0].routers.len(), 4);
        assert_eq!(vaults[0].addresses[0].chain, "BTC");
        assert_eq!(vaults[0].routers[0].chain, "ETH");
        assert_eq!(vaults[1].addresses.len(), 3);
        assert_eq!(vaults[1].routers.len(), 1);
    }

    #[test]
    fn test_error_response() {
        let error = ErrorResponse {
            message: "amount less than min swap amount (recommended_min_amount_in: 50570): invalid request".into(),
        };
        assert!(error.is_input_amount_error());
        assert_eq!(error.parse_min_amount(), Some("50570".into()));

        let error = ErrorResponse {
            message: "amount less than dust threshold: invalid request".into(),
        };
        assert!(error.is_input_amount_error());
        assert_eq!(error.parse_min_amount(), None);
    }

    #[test]
    fn test_quote_swap_response() {
        let response: QuoteSwapResponse = serde_json::from_str(r#"{"expected_amount_out":"42","recommended_min_amount_in":"50570","inbound_address":null,"router":null,"fees":{},"total_swap_seconds":null}"#).unwrap();

        assert_eq!(response.recommended_min_amount_in, BigInt::from(50570));
    }
}
