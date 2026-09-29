use std::fmt::Debug;

use async_trait::async_trait;
use chrono::{Duration, Utc};
use gem_client::Client;
use primitives::{
    SwapProvider,
    swap::{SwapPartnerTransaction, SwapReferralFee},
};
use serde::{Deserialize, Serialize};

use super::{
    broker::{BrokerClient, BrokerSwap, SWAPS_LIMIT},
    chain::ChainflipChain,
    client::{ChainflipClient, SwapTxResponse, chainflip_asset_to_asset_id},
};
use crate::{
    SwapperError,
    partner::{SwapPartnerCursor, SwapPartnerProvider, SwapPartnerTransactionsPage},
};

const PENDING_LOOKBACK: Duration = Duration::days(1);
const BROKER_FEE: &str = "BROKER";

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
struct ChainflipPartnerCursor {
    offset: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pending: Option<u64>,
}

#[derive(Debug)]
pub struct ChainflipPartnerProvider<C: Client + Clone + Debug> {
    broker: BrokerClient<C>,
    client: ChainflipClient<C>,
    api_key: String,
}

impl<C: Client + Clone + Debug> ChainflipPartnerProvider<C> {
    pub fn new(broker_client: C, client: C, api_key: String) -> Self {
        Self {
            broker: BrokerClient::new(broker_client),
            client: ChainflipClient::new(client),
            api_key,
        }
    }
}

fn map_asset_id(chain: &str, asset: &str) -> Option<primitives::AssetId> {
    chainflip_asset_to_asset_id(chain.parse::<ChainflipChain>().ok()?.to_chain(), asset)
}

pub fn map_partner_transaction(swap_id: &str, status: &SwapTxResponse) -> Option<SwapPartnerTransaction> {
    let deposit = status.deposit.as_ref()?;
    let referral_fee = status.fees.iter().find(|fee| fee.fee_type == BROKER_FEE).and_then(|fee| {
        Some(SwapReferralFee {
            asset_id: map_asset_id(&fee.chain, &fee.asset)?,
            value: fee.amount.clone(),
            amount_usd: None,
        })
    });
    Some(SwapPartnerTransaction {
        provider: SwapProvider::Chainflip,
        provider_transaction_id: swap_id.to_string(),
        status: status.swap_status(),
        from_address: status.fill_or_kill_params.as_ref()?.refund_address.clone(),
        to_address: status.dest_address.clone()?,
        from_asset_id: map_asset_id(&status.src_chain, &status.src_asset)?,
        from_value: deposit.amount.to_string(),
        from_amount_usd: None,
        to_asset_id: map_asset_id(&status.dest_chain, &status.dest_asset)?,
        to_value: status.swap_egress.as_ref().map(|egress| egress.amount.clone()).unwrap_or_else(|| "0".to_string()),
        to_amount_usd: None,
        referral_fee,
        from_transaction_hash: deposit.tx_ref.clone(),
        to_transaction_hash: status.swap_egress.as_ref().and_then(|egress| egress.tx_ref.clone()),
    })
}

fn is_recent_pending(swap: &BrokerSwap, now_ms: i64) -> bool {
    swap.status.last_state_chain_update_at.is_some_and(|updated_at| now_ms - updated_at < PENDING_LOOKBACK.num_milliseconds())
}

fn map_next_cursor(cursor: ChainflipPartnerCursor, swaps: &[BrokerSwap], pending: Option<u64>) -> Result<SwapPartnerCursor, SwapperError> {
    let offset = swaps.last().map_or(cursor.offset, |swap| swap.id + 1);
    if swaps.len() == SWAPS_LIMIT {
        return Ok(SwapPartnerCursor::Next(serde_json::to_string(&ChainflipPartnerCursor { offset, pending })?));
    }
    Ok(SwapPartnerCursor::Latest(serde_json::to_string(&ChainflipPartnerCursor {
        offset: pending.unwrap_or(offset),
        pending: None,
    })?))
}

#[async_trait]
impl<C: Client + Clone + Debug> SwapPartnerProvider for ChainflipPartnerProvider<C> {
    fn name(&self) -> &'static str {
        "chainflip"
    }

    async fn get_transactions(&self, cursor: Option<String>) -> Result<SwapPartnerTransactionsPage, SwapperError> {
        let cursor: ChainflipPartnerCursor = cursor.map(|cursor| serde_json::from_str(&cursor)).transpose()?.unwrap_or_default();
        let swaps = self.broker.get_swaps(&self.api_key, cursor.offset).await?;
        let now_ms = Utc::now().timestamp_millis();
        let mut pending = cursor.pending;
        let mut transactions = Vec::new();
        for swap in &swaps {
            match (swap.status.state.as_str(), &swap.status.swap_id) {
                ("completed" | "failed", Some(swap_id)) => {
                    let status = self.client.get_tx_status(swap_id).await?;
                    transactions.extend(map_partner_transaction(swap_id, &status));
                }
                _ if is_recent_pending(swap, now_ms) => pending = Some(pending.map_or(swap.id, |id| id.min(swap.id))),
                _ => {}
            }
        }
        Ok(SwapPartnerTransactionsPage {
            transactions,
            cursor: map_next_cursor(cursor, &swaps, pending)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use primitives::{AssetId, Chain, swap::SwapStatus};

    use super::*;

    #[test]
    fn test_map_partner_transaction() {
        let status: SwapTxResponse = serde_json::from_str(include_str!("test/partner_swap_usdc_to_btc.json")).unwrap();
        let transaction = map_partner_transaction("948784", &status).unwrap();
        let usdc = AssetId::from_token(Chain::Ethereum, "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48");

        assert_eq!(transaction.status, SwapStatus::Completed);
        assert_eq!(transaction.from_address, "0x0f01e8306cd0b63f4bdb9b3e2ce2dd62c459bc01");
        assert_eq!(transaction.to_address, "bc1qpgh96l6v2ny87ka6hfkhx8kq02v9z6yffmegz2");
        assert_eq!((transaction.from_asset_id, transaction.from_value), (usdc.clone(), "100000000".to_string()));
        assert_eq!((transaction.to_asset_id, transaction.to_value), (AssetId::from_chain(Chain::Bitcoin), "89187".to_string()));
        let fee = transaction.referral_fee.unwrap();
        assert_eq!((fee.asset_id, fee.value), (usdc, "497500".to_string()));
        assert_eq!(transaction.from_transaction_hash.as_deref(), Some("0x8e5b889acd7c4f27c967e06fc7c62e893cc1009f0b0f484d68c6e20123724d2d"));
        assert_eq!(transaction.to_transaction_hash.as_deref(), Some("c1b2bc233cd9e81aaf6a5b49bc375c8a7f0d2b918d42c63c6a5b051c97adfcf7"));
    }

    #[test]
    fn test_map_next_cursor() {
        let swaps: Vec<BrokerSwap> = serde_json::from_str(include_str!("test/partner_broker_swaps.json")).unwrap();

        assert!(!is_recent_pending(&swaps[1], 1_790_600_000_000));
        assert!(is_recent_pending(&swaps[1], 1_746_173_435_024 + 1_000));
        assert_eq!(map_next_cursor(ChainflipPartnerCursor::default(), &swaps, None).unwrap(), SwapPartnerCursor::Latest(r#"{"offset":47415}"#.to_string()));
        assert_eq!(map_next_cursor(ChainflipPartnerCursor::default(), &swaps, Some(25386)).unwrap(), SwapPartnerCursor::Latest(r#"{"offset":25386}"#.to_string()));
    }
}
