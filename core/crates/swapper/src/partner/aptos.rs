use async_trait::async_trait;
use gem_aptos::{
    APTOS_NATIVE_COIN,
    models::FungibleAssetActivity,
    provider::transactions_mapper::map_transaction,
    rpc::{AptosClient, AptosIndexer},
};
use gem_client::Client;
use primitives::{
    AssetId, Chain, SwapProvider, Transaction, TransactionState,
    swap::{SwapPartnerTransaction, SwapReferralFee, SwapStatus},
};

use crate::{
    SwapperError,
    fees::default_referral_fees,
    partner::{SwapPartnerCursor, SwapPartnerProvider, SwapPartnerTransactionsPage},
};

const DEPOSITS_LIMIT: usize = 20;

pub struct AptosPartnerProvider<C: Client> {
    indexer: AptosIndexer<C>,
    client: AptosClient<C>,
    fee_address: String,
}

impl<C: Client> AptosPartnerProvider<C> {
    pub fn new(indexer: C, archive: C) -> Self {
        Self {
            indexer: AptosIndexer::new(indexer),
            client: AptosClient::new(archive),
            fee_address: default_referral_fees().aptos.address,
        }
    }
}

fn map_asset_id(asset_type: &str) -> AssetId {
    if asset_type == APTOS_NATIVE_COIN || asset_type.trim_start_matches("0x").trim_start_matches('0') == "a" {
        return AssetId::from_chain(Chain::Aptos);
    }
    AssetId::from_token(Chain::Aptos, asset_type)
}

pub fn map_partner_transaction(transaction: &Transaction, deposit: &FungibleAssetActivity) -> Option<SwapPartnerTransaction> {
    let swap = transaction.swap_metadata()?;
    if swap.provider.as_deref() != Some(SwapProvider::Panora.id()) {
        return None;
    }
    Some(SwapPartnerTransaction {
        provider: SwapProvider::Panora,
        provider_transaction_id: transaction.id.hash.clone(),
        status: if transaction.state == TransactionState::Confirmed { SwapStatus::Completed } else { SwapStatus::Failed },
        from_address: transaction.from.clone(),
        to_address: transaction.from.clone(),
        from_asset_id: swap.from_asset,
        from_value: swap.from_value.to_string(),
        from_amount_usd: None,
        to_asset_id: swap.to_asset,
        to_value: swap.to_value.to_string(),
        to_amount_usd: None,
        referral_fee: Some(SwapReferralFee {
            asset_id: map_asset_id(&deposit.asset_type),
            value: deposit.amount.to_string(),
            amount_usd: None,
        }),
        from_transaction_hash: Some(transaction.id.hash.clone()),
        to_transaction_hash: Some(transaction.id.hash.clone()),
    })
}

#[async_trait]
impl<C: Client> SwapPartnerProvider for AptosPartnerProvider<C> {
    fn name(&self) -> &'static str {
        "aptos"
    }

    async fn get_transactions(&self, cursor: Option<String>) -> Result<SwapPartnerTransactionsPage, SwapperError> {
        let after_version = cursor.as_deref().filter(|cursor| !cursor.is_empty()).map(str::parse::<u64>).transpose()?.unwrap_or_default();
        let mut deposits = self.indexer.get_deposits(&self.fee_address, after_version, DEPOSITS_LIMIT).await.map_err(SwapperError::compute_quote_error)?;
        let is_full_page = deposits.len() == DEPOSITS_LIMIT;
        let last_version = deposits.last().map_or(after_version, |deposit| deposit.transaction_version);
        deposits.dedup_by_key(|deposit| deposit.transaction_version);

        let mut transactions = Vec::with_capacity(deposits.len());
        for deposit in deposits {
            let transaction = self.client.get_transaction_by_version(deposit.transaction_version).await.map_err(SwapperError::compute_quote_error)?;
            if let Some(transaction) = map_transaction(transaction).and_then(|transaction| map_partner_transaction(&transaction, &deposit)) {
                transactions.push(transaction);
            }
        }
        let cursor = last_version.to_string();
        Ok(SwapPartnerTransactionsPage {
            transactions,
            cursor: if is_full_page { SwapPartnerCursor::Next(cursor) } else { SwapPartnerCursor::Latest(cursor) },
        })
    }
}

#[cfg(test)]
mod tests {
    use gem_client::testkit::MockClient;

    use super::*;

    #[tokio::test]
    async fn test_get_transactions() {
        let indexer = MockClient::new().with_post(|_, _| Ok(include_bytes!("testdata/aptos_deposits.json").to_vec()));
        let archive = MockClient::new().with_get(|path| {
            assert_eq!(path, "/v1/transactions/by_version/4121649458");
            Ok(include_bytes!("testdata/aptos_transaction.json").to_vec())
        });

        let page = AptosPartnerProvider::new(indexer, archive).get_transactions(Some("4121649457".to_string())).await.unwrap();

        assert_eq!(page.cursor, SwapPartnerCursor::Latest("4121649458".to_string()));
        assert_eq!(
            page.transactions,
            vec![SwapPartnerTransaction {
                provider: SwapProvider::Panora,
                provider_transaction_id: "0x77e358c9a4c3090ec23803c6046cf34bf1a13bdc5245afdcceec4054db0f0980".to_string(),
                status: SwapStatus::Completed,
                from_address: "0x4eb20e735591a85bb58921ef2e6b55c385bba10e817ffe1e02e50deb6c594aef".to_string(),
                to_address: "0x4eb20e735591a85bb58921ef2e6b55c385bba10e817ffe1e02e50deb6c594aef".to_string(),
                from_asset_id: AssetId::from_chain(Chain::Aptos),
                from_value: "100000000".to_string(),
                from_amount_usd: None,
                to_asset_id: AssetId::from_token(Chain::Aptos, "0x357b0b74bc833e95a115ad22604854d6b0fca151cecd94111770e5d6ffc9dc2b"),
                to_value: "1891378".to_string(),
                to_amount_usd: None,
                referral_fee: Some(SwapReferralFee {
                    asset_id: AssetId::from_chain(Chain::Aptos),
                    value: "249652".to_string(),
                    amount_usd: None,
                }),
                from_transaction_hash: Some("0x77e358c9a4c3090ec23803c6046cf34bf1a13bdc5245afdcceec4054db0f0980".to_string()),
                to_transaction_hash: Some("0x77e358c9a4c3090ec23803c6046cf34bf1a13bdc5245afdcceec4054db0f0980".to_string()),
            }]
        );
    }
}
