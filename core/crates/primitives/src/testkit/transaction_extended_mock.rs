use crate::{Asset, Transaction, TransactionExtended};

impl TransactionExtended {
    pub fn mock() -> Self {
        Self::mock_transaction(Transaction::mock())
    }

    pub fn mock_transaction(transaction: Transaction) -> Self {
        let asset = Asset::from_chain(transaction.asset_id.chain);
        TransactionExtended {
            record_id: 1,
            transaction,
            fee_asset: asset.clone(),
            asset,
            price: None,
            fee_price: None,
            assets: vec![],
            prices: vec![],
            from_address: None,
            to_address: None,
            confirmation_eta_seconds: None,
        }
    }
}
