use crate::models::{BlockTransaction, Meta, Transaction, TransactionMessage};

impl BlockTransaction {
    pub fn mock(account_keys: &[&str], pre_balances: Vec<u64>, post_balances: Vec<u64>) -> Self {
        Self {
            meta: Meta {
                err: None,
                fee: 5000,
                pre_balances,
                post_balances,
                pre_token_balances: vec![],
                post_token_balances: vec![],
                loaded_addresses: None,
            },
            transaction: Transaction {
                message: TransactionMessage {
                    account_keys: account_keys.iter().map(|key| key.to_string()).collect(),
                    instructions: vec![],
                },
                signatures: vec![],
            },
        }
    }
}
