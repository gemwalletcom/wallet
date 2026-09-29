use async_trait::async_trait;
use gem_client::Client;
use gem_sui::{
    models::{Digest, STATUS_SUCCESS},
    provider::transactions_mapper::map_asset_id,
    rpc::{SuiIndexer, SuiTransactionsPage},
};
use num_bigint::{BigInt, Sign};
use primitives::{
    AssetId, SwapProvider,
    swap::{SwapPartnerTransaction, SwapReferralFee, SwapStatus},
};

use crate::{
    SwapperError,
    fees::default_referral_fees,
    mayan::SUI_MCTP_PACKAGE_ID,
    partner::{SwapPartnerCursor, SwapPartnerProvider, SwapPartnerTransactionsPage},
};

const TRANSACTIONS_LIMIT: usize = 50;

pub struct SuiPartnerProvider<C: Client> {
    indexer: SuiIndexer<C>,
    fee_address: String,
}

impl<C: Client> SuiPartnerProvider<C> {
    pub fn new(client: C) -> Self {
        Self {
            indexer: SuiIndexer::new(client),
            fee_address: default_referral_fees().sui.address,
        }
    }
}

fn map_balance_changes(transaction: &Digest, address: &str) -> Vec<(AssetId, BigInt)> {
    transaction
        .balance_changes
        .iter()
        .flatten()
        .filter(|change| change.owner.get_address_owner().as_deref() == Some(address))
        .map(|change| (map_asset_id(&change.coin_type), change.amount.clone()))
        .collect()
}

fn map_swap_legs(transaction: &Digest, sender: &str, fee_asset_id: &AssetId) -> Option<((AssetId, BigInt), (AssetId, BigInt))> {
    let gas_used = &transaction.effects.gas_used;
    let gas = BigInt::from(gas_used.computation_cost.clone()) + BigInt::from(gas_used.storage_cost.clone()) - BigInt::from(gas_used.storage_rebate.clone());
    let changes = map_balance_changes(transaction, sender)
        .into_iter()
        .map(|(asset_id, amount)| if asset_id.is_native() { (asset_id, amount + &gas) } else { (asset_id, amount) })
        .collect::<Vec<_>>();
    let leg = |sign: Sign| {
        changes
            .iter()
            .filter(|(_, amount)| amount.sign() == sign)
            .max_by_key(|(asset_id, amount)| (asset_id == fee_asset_id, amount.magnitude().clone()))
    };
    let (from_asset_id, from_amount) = leg(Sign::Minus)?;
    Some(((from_asset_id.clone(), -from_amount), leg(Sign::Plus)?.clone()))
}

pub fn map_partner_transaction(transaction: &Digest, fee_address: &str) -> Option<SwapPartnerTransaction> {
    let sender = transaction.effects.gas_object.owner.get_address_owner()?;
    let is_swap = transaction.events.iter().any(|event| event.event_type.contains("Swap"));
    let is_mayan = transaction.events.iter().any(|event| event.event_type.starts_with(SUI_MCTP_PACKAGE_ID));
    if sender == fee_address || !is_swap || is_mayan {
        return None;
    }
    let (fee_asset_id, fee_amount) = map_balance_changes(transaction, fee_address).into_iter().find(|(_, amount)| amount.sign() == Sign::Plus)?;
    let ((from_asset_id, from_amount), (to_asset_id, to_amount)) = map_swap_legs(transaction, &sender, &fee_asset_id)?;
    Some(SwapPartnerTransaction {
        provider: SwapProvider::CetusClmm,
        provider_transaction_id: transaction.digest.clone(),
        status: if transaction.effects.status.status == STATUS_SUCCESS { SwapStatus::Completed } else { SwapStatus::Failed },
        from_address: sender.clone(),
        to_address: sender,
        from_asset_id,
        from_value: from_amount.to_string(),
        from_amount_usd: None,
        to_asset_id,
        to_value: to_amount.to_string(),
        to_amount_usd: None,
        referral_fee: Some(SwapReferralFee {
            asset_id: fee_asset_id,
            value: fee_amount.to_string(),
            amount_usd: None,
        }),
        from_transaction_hash: Some(transaction.digest.clone()),
        to_transaction_hash: Some(transaction.digest.clone()),
    })
}

fn map_next_cursor(cursor: Option<String>, page: &SuiTransactionsPage) -> SwapPartnerCursor {
    let value = page.cursor.clone().or(cursor).unwrap_or_default();
    if page.has_next_page { SwapPartnerCursor::Next(value) } else { SwapPartnerCursor::Latest(value) }
}

#[async_trait]
impl<C: Client> SwapPartnerProvider for SuiPartnerProvider<C> {
    fn name(&self) -> &'static str {
        "sui"
    }

    async fn get_transactions(&self, cursor: Option<String>) -> Result<SwapPartnerTransactionsPage, SwapperError> {
        let cursor = cursor.filter(|cursor| !cursor.is_empty());
        let page = self.indexer.get_transactions_after(&self.fee_address, cursor.clone(), TRANSACTIONS_LIMIT).await.map_err(SwapperError::compute_quote_error)?;
        Ok(SwapPartnerTransactionsPage {
            transactions: page.transactions.iter().filter_map(|transaction| map_partner_transaction(transaction, &self.fee_address)).collect(),
            cursor: map_next_cursor(cursor, &page),
        })
    }
}

#[cfg(test)]
mod tests {
    use gem_client::testkit::MockClient;
    use primitives::Chain;

    use super::*;

    #[tokio::test]
    async fn test_get_transactions() {
        let client = MockClient::new().with_post(|_, _| Ok(include_bytes!("testdata/sui_transactions.json").to_vec()));
        let page = SuiPartnerProvider::new(client).get_transactions(None).await.unwrap();
        let sui = AssetId::from_chain(Chain::Sui);
        let token = |coin_type: &str| AssetId::from_token(Chain::Sui, coin_type);
        let summary = page
            .transactions
            .iter()
            .map(|transaction| {
                let fee = transaction.referral_fee.clone().unwrap();
                (
                    transaction.from_asset_id.clone(),
                    transaction.from_value.as_str(),
                    transaction.to_asset_id.clone(),
                    transaction.to_value.as_str(),
                    fee.asset_id,
                    fee.value,
                )
            })
            .collect::<Vec<_>>();

        assert_eq!(page.cursor, SwapPartnerCursor::Next("cursor".to_string()));
        assert_eq!(
            summary,
            vec![
                (
                    sui.clone(),
                    "4000000000",
                    token("0x356a26eb9e012a68958082340d4c4116e7f55615cf27affcff209cf0ae544f59::wal::WAL"),
                    "134875882463",
                    sui.clone(),
                    "20000000".to_string()
                ),
                (
                    token("0xdba34672e30cb065b1f93e3ab55318768fd6fef66c15942c9f7cb846e2f900e7::usdc::USDC"),
                    "740000",
                    sui.clone(),
                    "630350565",
                    sui,
                    "3167249".to_string()
                ),
                (
                    token("0x3a304c7feba2d819ea57c3542d68439ca2c386ba02159c740f7b406e592c62ea::haedal::HAEDAL"),
                    "50000000000",
                    token("0x06864a6f921804860930db6ddbe2e16acdf8504495ea7481637a1c8b9a8fe54b::cetus::CETUS"),
                    "48071929955",
                    token("0x06864a6f921804860930db6ddbe2e16acdf8504495ea7481637a1c8b9a8fe54b::cetus::CETUS"),
                    "241567487".to_string()
                ),
            ]
        );
        assert!(page.transactions.iter().all(|transaction| transaction.provider == SwapProvider::CetusClmm));
    }
}
