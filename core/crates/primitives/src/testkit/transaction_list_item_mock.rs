use crate::{Asset, Transaction, TransactionListItem};

impl TransactionListItem {
    pub fn mock_transaction(transaction: Transaction) -> Self {
        TransactionListItem {
            asset: Asset::from_chain(transaction.asset_id.chain),
            transaction,
            assets: vec![],
            from_address: None,
            to_address: None,
        }
    }
}
