use std::str::FromStr;

use async_trait::async_trait;
use chrono::{Duration, Utc};
use gem_client::{Client, ClientExt};
use num_bigint::BigUint;
use num_traits::ToPrimitive;
use primitives::{
    SwapProvider,
    swap::{SwapPartnerTransaction, SwapReferralFee, SwapStatus},
};

use super::{
    THORChainNetwork,
    model::{MidgardAction, MidgardActionsQuery, MidgardActionsResponse, MidgardPartnerCursor, TransactionCoin},
    target::MidgardTarget,
};
use crate::{
    SwapperError,
    fees::default_referral_fees,
    partner::{SwapPartnerCursor, SwapPartnerProvider, SwapPartnerTransactionsPage},
};

const ACTIONS_LIMIT: usize = 50;
const ACTION_TYPES: &str = "swap,refund";
const SWAP_ACTION: &str = "swap";
const LATEST_LOOKBACK: Duration = Duration::days(1);
const THORCHAIN_UNIT: f64 = 100_000_000.0;
const BPS_DIVISOR: f64 = 10_000.0;

pub struct ThorchainPartnerProvider<C: Client> {
    client: C,
    network: THORChainNetwork,
    affiliate: String,
}

impl<C: Client> ThorchainPartnerProvider<C> {
    pub fn new(client: C, network: THORChainNetwork) -> Self {
        Self {
            client,
            network,
            affiliate: default_referral_fees().thorchain.address,
        }
    }
}

fn map_status(action: &MidgardAction) -> SwapStatus {
    match (action.action_type.as_str(), action.status.as_str()) {
        (_, "pending") => SwapStatus::Pending,
        ("refund", _) => SwapStatus::Refunded,
        (_, "success") => SwapStatus::Completed,
        _ => SwapStatus::Failed,
    }
}

fn map_amount_usd(coin: &TransactionCoin, price_usd: Option<&String>) -> Option<f64> {
    Some(BigUint::from_str(&coin.amount).ok()?.to_f64()? / THORCHAIN_UNIT * price_usd?.parse::<f64>().ok()?)
}

pub fn map_partner_transaction(action: &MidgardAction, network: THORChainNetwork) -> Option<SwapPartnerTransaction> {
    let input = action.inputs.first()?;
    let output = action.outputs.iter().find(|output| !output.affiliate)?;
    let affiliate = action.outputs.iter().find(|output| output.affiliate);
    let swap = action.metadata.swap.as_ref();
    let from_coin = input.coins.first()?;
    let to_coin = output.coins.first()?;
    let from_amount_usd = map_amount_usd(from_coin, swap.and_then(|swap| swap.in_price_usd.as_ref()));
    let fee_bps = swap.and_then(|swap| swap.affiliate_fee.as_ref()?.parse::<u32>().ok()).filter(|bps| *bps > 0);
    let fee_amount_usd = from_amount_usd.zip(fee_bps).map(|(amount, bps)| amount * f64::from(bps) / BPS_DIVISOR);
    let referral_fee = affiliate
        .and_then(|affiliate| affiliate.coins.first())
        .and_then(|coin| {
            Some(SwapReferralFee {
                asset_id: coin.asset_id(network)?,
                value: coin.native_value(network)?.to_string(),
                amount_usd: fee_amount_usd,
            })
        })
        .or_else(|| {
            Some(SwapReferralFee {
                asset_id: from_coin.asset_id(network)?,
                value: (from_coin.native_value(network)? * fee_bps? / BPS_DIVISOR as u32).to_string(),
                amount_usd: fee_amount_usd,
            })
        });
    Some(SwapPartnerTransaction {
        provider: network.provider(),
        provider_transaction_id: input.tx_id.clone(),
        status: map_status(action),
        from_address: input.address.clone(),
        to_address: output.address.clone(),
        from_asset_id: from_coin.asset_id(network)?,
        from_value: from_coin.native_value(network)?.to_string(),
        from_amount_usd,
        to_asset_id: to_coin.asset_id(network)?,
        to_value: to_coin.native_value(network)?.to_string(),
        to_amount_usd: map_amount_usd(to_coin, swap.and_then(|swap| swap.out_price_usd.as_ref())),
        referral_fee,
        from_transaction_hash: Some(input.tx_id.clone()),
        to_transaction_hash: Some(output.tx_id.clone()).filter(|hash| !hash.is_empty()),
    })
}

fn map_partner_transactions(actions: &[MidgardAction], network: THORChainNetwork) -> Vec<SwapPartnerTransaction> {
    let mut transactions: Vec<SwapPartnerTransaction> = Vec::new();
    for action in actions {
        let Some(transaction) = map_partner_transaction(action, network) else {
            continue;
        };
        match transactions.iter().position(|existing| existing.provider_transaction_id == transaction.provider_transaction_id) {
            Some(index) if action.action_type == SWAP_ACTION => transactions[index] = transaction,
            Some(_) => {}
            None => transactions.push(transaction),
        }
    }
    transactions
}

fn map_next_cursor(cursor: MidgardPartnerCursor, response: &MidgardActionsResponse) -> Result<SwapPartnerCursor, SwapperError> {
    match &response.meta.next_page_token {
        Some(token) if response.actions.len() == ACTIONS_LIMIT => Ok(SwapPartnerCursor::Next(serde_json::to_string(&MidgardPartnerCursor {
            next_page_token: Some(token.clone()),
            ..cursor
        })?)),
        _ => Ok(SwapPartnerCursor::Latest(serde_json::to_string(&MidgardPartnerCursor {
            from_timestamp: cursor.walk_started_at.map(|started_at| started_at - LATEST_LOOKBACK.num_seconds()),
            ..MidgardPartnerCursor::default()
        })?)),
    }
}

#[async_trait]
impl<C: Client> SwapPartnerProvider for ThorchainPartnerProvider<C> {
    fn name(&self) -> &str {
        match self.network {
            THORChainNetwork::Thorchain => SwapProvider::Thorchain.id(),
            THORChainNetwork::Mayachain => SwapProvider::Mayachain.id(),
        }
    }

    async fn get_transactions(&self, cursor: Option<String>) -> Result<SwapPartnerTransactionsPage, SwapperError> {
        let cursor: MidgardPartnerCursor = cursor.map(|cursor| serde_json::from_str(&cursor)).transpose()?.unwrap_or_default();
        let cursor = MidgardPartnerCursor {
            walk_started_at: cursor.walk_started_at.or(Some(Utc::now().timestamp())),
            ..cursor
        };
        let query = MidgardActionsQuery {
            affiliate: self.affiliate.clone(),
            action_type: ACTION_TYPES,
            limit: ACTIONS_LIMIT,
            from_timestamp: cursor.from_timestamp,
            next_page_token: cursor.next_page_token.clone(),
        };
        let response: MidgardActionsResponse = self.client.get(MidgardTarget::Actions { network: self.network, query }).await.map_err(SwapperError::from)?;
        Ok(SwapPartnerTransactionsPage {
            transactions: map_partner_transactions(&response.actions, self.network),
            cursor: map_next_cursor(cursor, &response)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use primitives::{AssetId, Chain, SwapProvider, asset_constants::ETHEREUM_USDT_ASSET_ID};

    use super::*;

    fn response() -> MidgardActionsResponse {
        serde_json::from_str(include_str!("testdata/midgard_actions.json")).unwrap()
    }

    #[test]
    fn test_map_partner_transaction() {
        let actions = response().actions;

        let swap = map_partner_transaction(&actions[0], THORChainNetwork::Thorchain).unwrap();
        assert_eq!(swap.provider, SwapProvider::Thorchain);
        assert_eq!(swap.status, SwapStatus::Completed);
        assert_eq!(swap.from_address, "TWVy8uUT1SAieQReu2NPgSF7v6MePvbDoB");
        assert_eq!(swap.to_address, "ltc1q8hdh6lr4gqzzqrf0sfw8xzjclhx20c3e6fp72s");
        assert_eq!((swap.from_asset_id, swap.from_value), (AssetId::from_chain(Chain::Tron), "37000000".to_string()));
        assert_eq!((swap.to_asset_id, swap.to_value), (AssetId::from_chain(Chain::Litecoin), "17731306".to_string()));
        let fee = swap.referral_fee.unwrap();
        assert_eq!((fee.asset_id, fee.value), (AssetId::from_chain(Chain::Thorchain), "8012059".to_string()));
        assert!((swap.from_amount_usd.unwrap() - 12.40679052051771).abs() < 1e-9);

        let token = map_partner_transaction(&actions[1], THORChainNetwork::Thorchain).unwrap();
        assert_eq!((token.to_asset_id, token.to_value), (ETHEREUM_USDT_ASSET_ID.clone(), "2710430".to_string()));
        let fee = token.referral_fee.unwrap();
        assert_eq!((fee.asset_id, fee.value), (AssetId::from_chain(Chain::Tron), "50000".to_string()));

        let refund = map_partner_transaction(&actions[2], THORChainNetwork::Thorchain).unwrap();
        assert_eq!(refund.status, SwapStatus::Refunded);
        assert_eq!(refund.referral_fee, None);

        assert_eq!(map_partner_transaction(&actions[3], THORChainNetwork::Thorchain), None);
    }

    #[test]
    fn test_map_partner_transactions_prefers_swap_over_partial_refund() {
        let actions: Vec<MidgardAction> = serde_json::from_str(include_str!("testdata/midgard_refund_swap_pair.json")).unwrap();
        let transactions = map_partner_transactions(&actions, THORChainNetwork::Thorchain);

        assert_eq!(transactions.len(), 1);
        assert_eq!(transactions[0].status, SwapStatus::Completed);
    }

    #[test]
    fn test_map_next_cursor() {
        let walk = MidgardPartnerCursor {
            walk_started_at: Some(1_790_600_000),
            ..MidgardPartnerCursor::default()
        };
        let response = MidgardActionsResponse { actions: vec![], ..response() };

        assert_eq!(map_next_cursor(walk, &response).unwrap(), SwapPartnerCursor::Latest(r#"{"fromTimestamp":1790513600}"#.to_string()));
    }
}
