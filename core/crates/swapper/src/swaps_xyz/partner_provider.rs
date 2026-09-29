use std::str::FromStr;

use async_trait::async_trait;
use chrono::{Duration, Utc};
use gem_client::{Client, ClientExt};
use num_bigint::BigUint;
use primitives::{
    AssetId, SwapProvider,
    swap::{SwapPartnerTransaction, SwapReferralFee, SwapStatus},
};

use super::{
    chain::SwapsXyzChain,
    model::{PartnerCursor, PartnerTransaction, PaymentToken, TransactionsQuery, TransactionsResponse},
    target::SwapsXyzTarget,
};
use crate::{
    SwapperError,
    fees::default_referral_fees,
    partner::{SwapPartnerCursor, SwapPartnerProvider, SwapPartnerTransactionsPage},
};

const TRANSACTIONS_LIMIT: usize = 50;
const LATEST_LOOKBACK: Duration = Duration::days(1);
const BPS_DIVISOR: u32 = 10_000;

pub struct SwapsXyzPartnerProvider<C: Client> {
    client: C,
    fee_address: String,
}

impl<C: Client> SwapsXyzPartnerProvider<C> {
    pub fn new(client: C) -> Self {
        Self {
            client,
            fee_address: default_referral_fees().evm.address,
        }
    }
}

fn map_status(status: &str) -> SwapStatus {
    match status {
        "success" | "completed" => SwapStatus::Completed,
        "refunded" => SwapStatus::Refunded,
        "failed" | "expired" => SwapStatus::Failed,
        _ => SwapStatus::Pending,
    }
}

fn map_asset_id(token: &PaymentToken) -> Option<AssetId> {
    let chain = SwapsXyzChain::from_id(token.chain_id)?.chain;
    match (token.is_native, &token.token_address) {
        (true, _) => Some(AssetId::from_chain(chain)),
        (false, Some(address)) => Some(AssetId::from_token(chain, address)),
        (false, None) => None,
    }
}

pub fn map_partner_transaction(transaction: &PartnerTransaction, fee_address: &str) -> Option<SwapPartnerTransaction> {
    let src = transaction.src_tx.as_ref()?;
    let dst = transaction.dst_tx.as_ref()?;
    let from_asset_id = map_asset_id(&src.payment_token)?;
    let fee_bps = transaction
        .action_request
        .as_ref()
        .and_then(|request| request.app_fees.iter().find(|fee| fee.receiver_address.eq_ignore_ascii_case(fee_address)))
        .map(|fee| fee.bps);
    let referral_fee = fee_bps.and_then(|bps| {
        Some(SwapReferralFee {
            asset_id: from_asset_id.clone(),
            value: (BigUint::from_str(&src.payment_token.amount).ok()? * bps / BPS_DIVISOR).to_string(),
            amount_usd: src.payment_token.usd_amount.map(|amount| amount * f64::from(bps) / f64::from(BPS_DIVISOR)),
        })
    });
    Some(SwapPartnerTransaction {
        provider: SwapProvider::SwapsXyz,
        provider_transaction_id: transaction.tx_id.clone(),
        status: map_status(&transaction.status),
        from_address: transaction.sender.clone(),
        to_address: dst.to_address.clone()?,
        from_asset_id,
        from_value: src.payment_token.amount.clone(),
        from_amount_usd: src.payment_token.usd_amount,
        to_asset_id: map_asset_id(&dst.payment_token)?,
        to_value: dst.payment_token.amount.clone(),
        to_amount_usd: dst.payment_token.usd_amount,
        referral_fee,
        from_transaction_hash: src.tx_hash.clone(),
        to_transaction_hash: dst.tx_hash.clone(),
    })
}

fn map_next_cursor(cursor: PartnerCursor, response: &TransactionsResponse) -> Result<SwapPartnerCursor, SwapperError> {
    match &response.cursor.next {
        Some(next) if !response.txs.is_empty() => Ok(SwapPartnerCursor::Next(serde_json::to_string(&PartnerCursor { next: Some(next.clone()), ..cursor })?)),
        _ => Ok(SwapPartnerCursor::Latest(serde_json::to_string(&PartnerCursor {
            start_date: cursor.walk_started_at.map(|started_at| started_at - LATEST_LOOKBACK.num_seconds()),
            ..PartnerCursor::default()
        })?)),
    }
}

#[async_trait]
impl<C: Client> SwapPartnerProvider for SwapsXyzPartnerProvider<C> {
    fn name(&self) -> &str {
        SwapProvider::SwapsXyz.id()
    }

    async fn get_transactions(&self, cursor: Option<String>) -> Result<SwapPartnerTransactionsPage, SwapperError> {
        let cursor: PartnerCursor = cursor.map(|cursor| serde_json::from_str(&cursor)).transpose()?.unwrap_or_default();
        let cursor = PartnerCursor {
            walk_started_at: cursor.walk_started_at.or(Some(Utc::now().timestamp())),
            ..cursor
        };
        let query = TransactionsQuery {
            limit: TRANSACTIONS_LIMIT,
            start_date: cursor.start_date,
            cursor: cursor.next.clone(),
        };
        let response: TransactionsResponse = self.client.get(SwapsXyzTarget::Transactions { query }).await.map_err(SwapperError::from)?;
        Ok(SwapPartnerTransactionsPage {
            transactions: response.txs.iter().filter_map(|transaction| map_partner_transaction(transaction, &self.fee_address)).collect(),
            cursor: map_next_cursor(cursor, &response)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use primitives::Chain;

    use super::*;

    const FEE_ADDRESS: &str = "0x0D9DAB1A248f63B0a48965bA8435e4de7497a3dC";

    fn response() -> TransactionsResponse {
        serde_json::from_str(include_str!("testdata/partner_transactions.json")).unwrap()
    }

    #[test]
    fn test_map_partner_transaction() {
        let transaction = map_partner_transaction(&response().txs[0], FEE_ADDRESS).unwrap();

        assert_eq!(transaction.status, SwapStatus::Completed);
        assert_eq!(transaction.from_address, "37FSXICWN3SYMKWTCYP4XPKKLK3657JBEHVSZJAYHJ4VWP6PQDRBE5ZY3U");
        assert_eq!(transaction.to_address, "rD8TMhN9Ke4diTB7XKuAWVD57harKdNG7G");
        assert_eq!((transaction.from_asset_id, transaction.from_value), (AssetId::from_chain(Chain::Algorand), "316899014".to_string()));
        assert_eq!((transaction.to_asset_id, transaction.to_value), (AssetId::from_chain(Chain::Xrp), "27854042".to_string()));
        let fee = transaction.referral_fee.unwrap();
        assert_eq!((fee.asset_id, fee.value), (AssetId::from_chain(Chain::Algorand), "1584495".to_string()));
        assert!((fee.amount_usd.unwrap() - 0.2149526).abs() < 1e-9);
    }

    #[test]
    fn test_map_next_cursor() {
        let walk = PartnerCursor {
            walk_started_at: Some(1_790_600_000),
            ..PartnerCursor::default()
        };
        assert!(matches!(map_next_cursor(walk.clone(), &response()).unwrap(), SwapPartnerCursor::Next(_)));

        let last = TransactionsResponse {
            cursor: super::super::model::TransactionsCursor { next: None },
            ..response()
        };
        assert_eq!(map_next_cursor(walk, &last).unwrap(), SwapPartnerCursor::Latest(r#"{"startDate":1790513600}"#.to_string()));
    }
}
