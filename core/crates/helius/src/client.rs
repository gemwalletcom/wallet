use gem_client::{Client as Transport, ClientError, ClientExt};

use crate::model::{Transaction, TransactionsQuery};
use crate::target::HeliusTarget;

pub struct Client<C: Transport> {
    client: C,
}

impl<C: Transport> Client<C> {
    pub fn new(client: C) -> Self {
        Self { client }
    }

    pub async fn get_address_transactions(&self, address: &str, after_signature: Option<String>, limit: usize) -> Result<Vec<Transaction>, ClientError> {
        let target = HeliusTarget::AddressTransactions {
            address: address.to_string(),
            query: TransactionsQuery {
                limit,
                token_accounts: "balanceChanged",
                sort_order: "asc",
                after_signature,
            },
        };
        self.client.get(target).await
    }
}
