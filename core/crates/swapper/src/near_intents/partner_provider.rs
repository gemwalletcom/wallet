use std::{fmt::Debug, str::FromStr};

use async_trait::async_trait;
use chrono::{Duration, Utc};
use gem_client::Client;
use num_bigint::BigUint;
use primitives::{
    SwapProvider,
    swap::{SwapPartnerTransaction, SwapReferralFee},
};

use super::{
    assets::get_asset_id_from_near_asset,
    client::NearIntentsExplorer,
    model::{ExplorerPartnerCursor, ExplorerPartnerTransaction, ExplorerPartnerTransactionsQuery},
    provider::map_transaction_status,
};
use crate::{
    SwapperError,
    fees::{DEFAULT_REFERRER, default_referral_fees},
    partner::{SwapPartnerCursor, SwapPartnerProvider, SwapPartnerTransactionsPage},
};

const TRANSACTIONS_LIMIT: usize = 1000;
const LATEST_LOOKBACK: Duration = Duration::days(1);
const BPS_DIVISOR: u32 = 10_000;

#[derive(Debug)]
pub struct NearIntentsPartnerProvider<C: Client> {
    explorer: NearIntentsExplorer<C>,
    fee_address: String,
}

impl<C: Client + Send + Sync + Debug> NearIntentsPartnerProvider<C> {
    pub fn new(client: C) -> Self {
        Self {
            explorer: NearIntentsExplorer::new(client),
            fee_address: default_referral_fees().evm.address,
        }
    }
}

pub fn map_partner_transaction(transaction: &ExplorerPartnerTransaction, fee_address: &str) -> Option<SwapPartnerTransaction> {
    let from_asset_id = get_asset_id_from_near_asset(&transaction.origin_asset)?;
    let referral_fee = transaction.app_fees.iter().find(|fee| fee.recipient.eq_ignore_ascii_case(fee_address)).and_then(|fee| {
        let value = BigUint::from_str(&transaction.amount_in).ok()? * fee.fee / BPS_DIVISOR;
        Some(SwapReferralFee {
            asset_id: from_asset_id.clone(),
            value: value.to_string(),
            amount_usd: transaction.amount_in_usd.parse::<f64>().ok().map(|amount| amount * f64::from(fee.fee) / f64::from(BPS_DIVISOR)),
        })
    });
    Some(SwapPartnerTransaction {
        provider: SwapProvider::NearIntents,
        provider_transaction_id: match &transaction.deposit_memo {
            Some(memo) => format!("{}:{memo}", transaction.deposit_address),
            None => transaction.deposit_address.clone(),
        },
        status: map_transaction_status(&transaction.status),
        from_address: transaction.senders.first().unwrap_or(&transaction.refund_to).clone(),
        to_address: transaction.recipient.clone(),
        from_asset_id,
        from_value: transaction.amount_in.clone(),
        from_amount_usd: transaction.amount_in_usd.parse().ok(),
        to_asset_id: get_asset_id_from_near_asset(&transaction.destination_asset)?,
        to_value: transaction.amount_out.clone(),
        to_amount_usd: transaction.amount_out_usd.parse().ok(),
        referral_fee,
        from_transaction_hash: transaction.origin_chain_tx_hashes.first().cloned(),
        to_transaction_hash: transaction.destination_chain_tx_hashes.first().cloned(),
    })
}

fn map_next_cursor(cursor: ExplorerPartnerCursor, transactions: &[ExplorerPartnerTransaction]) -> Result<SwapPartnerCursor, SwapperError> {
    match transactions.last() {
        Some(last) if transactions.len() == TRANSACTIONS_LIMIT => Ok(SwapPartnerCursor::Next(serde_json::to_string(&ExplorerPartnerCursor {
            last_deposit_address: Some(last.deposit_address.clone()),
            last_deposit_memo: last.deposit_memo.clone(),
            ..cursor
        })?)),
        _ => Ok(SwapPartnerCursor::Latest(serde_json::to_string(&ExplorerPartnerCursor {
            start_timestamp: cursor.walk_started_at.map(|started_at| started_at - LATEST_LOOKBACK.num_seconds()),
            ..ExplorerPartnerCursor::default()
        })?)),
    }
}

#[async_trait]
impl<C: Client + Send + Sync + Debug> SwapPartnerProvider for NearIntentsPartnerProvider<C> {
    fn name(&self) -> &str {
        SwapProvider::NearIntents.id()
    }

    async fn get_transactions(&self, cursor: Option<String>) -> Result<SwapPartnerTransactionsPage, SwapperError> {
        let cursor: ExplorerPartnerCursor = cursor.map(|cursor| serde_json::from_str(&cursor)).transpose()?.unwrap_or_default();
        let cursor = ExplorerPartnerCursor {
            walk_started_at: cursor.walk_started_at.or(Some(Utc::now().timestamp())),
            ..cursor
        };
        let query = ExplorerPartnerTransactionsQuery {
            referral: DEFAULT_REFERRER.to_string(),
            number_of_transactions: TRANSACTIONS_LIMIT,
            start_timestamp_unix: cursor.start_timestamp,
            last_deposit_address: cursor.last_deposit_address.clone(),
            last_deposit_memo: cursor.last_deposit_memo.clone(),
        };
        let transactions = self.explorer.get_partner_transactions(query).await?;
        Ok(SwapPartnerTransactionsPage {
            transactions: transactions.iter().filter_map(|transaction| map_partner_transaction(transaction, &self.fee_address)).collect(),
            cursor: map_next_cursor(cursor, &transactions)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use primitives::{AssetId, Chain, swap::SwapStatus};

    use super::*;

    const FEE_ADDRESS: &str = "0x0D9DAB1A248f63B0a48965bA8435e4de7497a3dC";

    fn transactions() -> Vec<ExplorerPartnerTransaction> {
        serde_json::from_str(include_str!("testdata/partner_transactions.json")).unwrap()
    }

    #[test]
    fn test_map_partner_transaction() {
        let transaction = map_partner_transaction(&transactions()[0], FEE_ADDRESS).unwrap();

        assert_eq!(transaction.status, SwapStatus::Completed);
        assert_eq!(transaction.from_address, "8a9T3RFfsouT5ttxgCPJjnf4RSyhDwEofUVSsLrohXf4");
        assert_eq!(transaction.from_asset_id, AssetId::from_chain(Chain::Solana));
        assert_eq!(transaction.from_value, "68908191");
        assert_eq!(transaction.to_asset_id, AssetId::from_chain(Chain::Litecoin));
        assert_eq!(transaction.to_value, "11875369");
        assert_eq!(transaction.from_amount_usd, Some(8.10429234351));

        let referral_fee = transaction.referral_fee.unwrap();
        assert_eq!(referral_fee.asset_id, AssetId::from_chain(Chain::Solana));
        assert_eq!(referral_fee.value, "172270");
        assert!((referral_fee.amount_usd.unwrap() - 0.020260730858775).abs() < 1e-12);

        let legacy = ExplorerPartnerTransaction {
            senders: vec![],
            origin_asset: "nep141:btc.omft.near".to_string(),
            ..transactions()[0].clone()
        };
        let transaction = map_partner_transaction(&legacy, FEE_ADDRESS).unwrap();
        assert_eq!(transaction.from_address, legacy.refund_to);
        assert_eq!(transaction.from_asset_id, AssetId::from_chain(Chain::Bitcoin));
    }

    #[test]
    fn test_map_next_cursor() {
        let transactions = transactions();
        let walk = ExplorerPartnerCursor {
            walk_started_at: Some(1_790_600_000),
            ..ExplorerPartnerCursor::default()
        };

        assert_eq!(map_next_cursor(walk, &transactions).unwrap(), SwapPartnerCursor::Latest(r#"{"startTimestamp":1790513600}"#.to_string()));
    }
}
