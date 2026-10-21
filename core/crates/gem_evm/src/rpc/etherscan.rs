use std::error::Error;

use etherscan::EtherscanApi;
use etherscan::client::EtherscanClient;
use gem_client::Client;
use primitives::EVMChain;

use super::TransactionReference;

pub struct EtherscanClients<C: Client> {
    pub(crate) etherscan: EtherscanClient<C>,
    pub(crate) routescan: EtherscanClient<C>,
    pub(crate) zksync: EtherscanClient<C>,
}

impl<C: Client> EtherscanClients<C> {
    pub fn new(etherscan: C, etherscan_key: Option<String>, routescan: C, zksync: C) -> Self {
        Self {
            etherscan: EtherscanClient::new_with_client(etherscan, etherscan_key, EtherscanApi::Etherscan),
            routescan: EtherscanClient::new_with_client(routescan, None, EtherscanApi::Routescan),
            zksync: EtherscanClient::new_with_client(zksync, None, EtherscanApi::ZkSync),
        }
    }
}

pub(crate) async fn get_transactions_by_address<C: Client>(client: &EtherscanClient<C>, chain: EVMChain, address: &str, limit: usize) -> Result<Vec<TransactionReference>, Box<dyn Error + Send + Sync>> {
    let (transactions, token_transfers) = futures::try_join!(client.get_transactions(chain, address, limit), client.get_token_transfers(chain, address, limit))?;
    Ok(TransactionReference::unique_newest_first(
        transactions.into_iter().chain(token_transfers).map(|transaction| (transaction.block_number, transaction.hash)),
    ))
}

#[cfg(test)]
mod tests {
    use etherscan::testkit::{NO_TRANSACTIONS, TOKEN_TRANSFERS, TRANSACTIONS};
    use gem_client::testkit::MockClient;

    use super::*;

    #[tokio::test]
    async fn test_get_transactions_by_address() {
        let client = EtherscanClient::new_with_client(
            MockClient::new().with_get(|path| {
                let response = match path {
                    "/v2/api?chainid=81457&module=account&action=txlist&address=0x123&page=1&offset=3&sort=desc" => TRANSACTIONS,
                    "/v2/api?chainid=81457&module=account&action=tokentx&address=0x123&page=1&offset=3&sort=desc" => TOKEN_TRANSFERS,
                    "/v2/api?chainid=81457&module=account&action=txlist&address=0x456&page=1&offset=3&sort=desc" => NO_TRANSACTIONS,
                    "/v2/api?chainid=81457&module=account&action=tokentx&address=0x456&page=1&offset=3&sort=desc" => NO_TRANSACTIONS,
                    _ => panic!("unexpected path: {path}"),
                };
                Ok(response.as_bytes().to_vec())
            }),
            None,
            EtherscanApi::Etherscan,
        );

        assert_eq!(
            get_transactions_by_address(&client, EVMChain::Blast, "0x123", 3).await.unwrap(),
            vec![
                TransactionReference::new("0xtoken".to_string(), Some(11)),
                TransactionReference::new("0xnormal".to_string(), Some(10)),
                TransactionReference::new("0xshared".to_string(), Some(8))
            ]
        );
        assert_eq!(get_transactions_by_address(&client, EVMChain::Blast, "0x456", 3).await.unwrap(), vec![]);
    }
}
