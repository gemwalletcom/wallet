mod mapper;
mod model;

use std::{
    cmp::Reverse,
    collections::{BTreeMap, BTreeSet, HashMap},
    error::Error,
    slice,
};

use futures::future::try_join_all;
use gem_client::{Client, ClientExt};
use num_bigint::BigUint;
use primitives::{Transaction, TransactionIdRequest};

use self::mapper::{map_address_transfer, map_raw_transaction};
use self::model::{FastNearTransaction, FastNearTransfer, TransactionsRequest, TransactionsResponse, TransferDirection, TransfersRequest, TransfersResponse};

const MAX_TRANSFERS_LIMIT: usize = 100;
const TRANSACTIONS_BATCH_SIZE: usize = 20;

#[derive(Debug)]
pub struct NearIndexer<C: Client> {
    transfers_client: C,
    transactions_client: C,
}

impl<C: Client> NearIndexer<C> {
    pub fn new(transfers_client: C, transactions_client: C) -> Self {
        Self {
            transfers_client,
            transactions_client,
        }
    }

    async fn get_transfers(
        &self,
        address: &str,
        direction: TransferDirection,
        limit: usize,
        from_timestamp_ms: Option<u64>,
    ) -> Result<Vec<FastNearTransfer>, Box<dyn Error + Send + Sync>> {
        let request = TransfersRequest {
            account_id: address,
            direction,
            desc: true,
            limit: limit.min(MAX_TRANSFERS_LIMIT),
            from_timestamp_ms,
        };
        let response: TransfersResponse = self.transfers_client.post("/v0/transfers", &request).await?;
        Ok(response.transfers)
    }

    async fn get_transaction_fees(&self, transfers: &[FastNearTransfer], address: &str) -> Result<HashMap<String, BigUint>, Box<dyn Error + Send + Sync>> {
        let transaction_ids = transfers
            .iter()
            .filter(|transfer| transfer.signer_id == address && transfer.predecessor_id == address)
            .filter_map(|transfer| transfer.transaction_id.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        Ok(self
            .get_transactions_by_hashes(&transaction_ids)
            .await?
            .into_iter()
            .map(|transaction| {
                let fee = transaction.fee();
                (transaction.transaction.hash, fee)
            })
            .collect())
    }

    async fn get_transactions_by_hashes(&self, transaction_ids: &[String]) -> Result<Vec<FastNearTransaction>, Box<dyn Error + Send + Sync>> {
        let requests = transaction_ids.chunks(TRANSACTIONS_BATCH_SIZE).map(|transaction_ids| async move {
            let request = TransactionsRequest { tx_hashes: transaction_ids };
            let response: TransactionsResponse = self.transactions_client.post("/v0/transactions", &request).await?;
            Ok::<_, Box<dyn Error + Send + Sync>>(response.transactions)
        });
        Ok(try_join_all(requests).await?.into_iter().flatten().collect())
    }

    pub(crate) async fn get_transactions_by_address(&self, address: &str, limit: usize, from_timestamp: Option<u64>) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        let from_timestamp_ms = from_timestamp
            .map(|timestamp| timestamp.checked_mul(1_000).ok_or("NEAR timestamp exceeds milliseconds range"))
            .transpose()?;
        let sender_transfers = self.get_transfers(address, TransferDirection::Sender, limit, from_timestamp_ms).await?;
        let receiver_transfers = self.get_transfers(address, TransferDirection::Receiver, limit, from_timestamp_ms).await?;
        let transfers = sender_transfers
            .into_iter()
            .chain(receiver_transfers)
            .map(|transfer| ((Reverse(transfer.block_timestamp), transfer.receipt_id.clone()), transfer))
            .collect::<BTreeMap<_, _>>()
            .into_values()
            .take(limit)
            .collect::<Vec<_>>();

        let fees = self.get_transaction_fees(&transfers, address).await?;
        transfers.into_iter().map(|transfer| map_address_transfer(transfer, &fees, address)).collect()
    }

    pub(crate) async fn get_transaction_by_hash(&self, request: TransactionIdRequest) -> Result<Option<Transaction>, Box<dyn Error + Send + Sync>> {
        let hash = request.hash;
        let transactions = self.get_transactions_by_hashes(slice::from_ref(&hash)).await?;
        let Some(transaction) = transactions.into_iter().find(|transaction| transaction.transaction.hash == hash) else {
            return Ok(None);
        };
        if let Some(expected_block) = request.block_number
            && transaction.execution_outcome.block_height != expected_block
        {
            return Err(format!(
                "NEAR transaction block mismatch: expected {expected_block}, got {}",
                transaction.execution_outcome.block_height
            )
            .into());
        }
        Ok(Some(map_raw_transaction(transaction)?))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use chain_traits::ChainTransaction;
    use gem_client::testkit::MockClient;
    use primitives::{Chain, Transaction, TransactionIdRequest, TransactionType, asset_constants::NEAR_USDT_ASSET_ID};
    use serde_json::Value;

    use super::*;

    #[derive(Clone, Default)]
    struct MockRequests {
        transfers: Arc<Mutex<Vec<Value>>>,
        transactions: Arc<Mutex<Vec<Value>>>,
    }

    #[derive(Default)]
    struct MockResponses {
        sender_transfers: Option<&'static str>,
        receiver_transfers: Option<&'static str>,
        transactions: Option<&'static str>,
    }

    fn mock_client(requests: MockRequests, responses: MockResponses) -> MockClient {
        MockClient::new().with_post(move |path, body| {
            let request = serde_json::from_slice::<Value>(body).unwrap();
            match path {
                "/v0/transfers" => {
                    let response = match request["direction"].as_str().unwrap() {
                        "sender" => responses.sender_transfers.unwrap(),
                        "receiver" => responses.receiver_transfers.unwrap(),
                        direction => panic!("unexpected transfer direction: {direction}"),
                    };
                    requests.transfers.lock().unwrap().push(request);
                    Ok(response.as_bytes().to_vec())
                }
                "/v0/transactions" => {
                    requests.transactions.lock().unwrap().push(request);
                    Ok(responses.transactions.unwrap().as_bytes().to_vec())
                }
                path => panic!("unexpected path: {path}"),
            }
        })
    }

    fn assert_usdt_transaction(transaction: &Transaction) {
        assert_eq!(transaction.hash, "DXUp65qSLjpbMrMVubtH1YY13fDHLA5av7q7skJ8kx5E");
        assert_eq!(transaction.block_number.as_deref(), Some("211048907"));
        assert_eq!(transaction.asset_id, NEAR_USDT_ASSET_ID.clone());
        assert_eq!(transaction.transaction_type, TransactionType::Transfer);
        assert_eq!(transaction.from, "e589457354361489a89039b8be6737cbc2db4d13919b6ccf23889a60f3b0d8f3");
        assert_eq!(transaction.to, "bb90f7cd3f611466d4e8aaee55541d5da6881e01a4155bca49041c1d692b4ff8");
        assert_eq!(transaction.value, "99500026");
        assert_eq!(transaction.fee, "411253844391900000000");
    }

    #[tokio::test]
    async fn test_get_transactions_by_address() {
        let requests = MockRequests::default();
        let client = mock_client(
            requests.clone(),
            MockResponses {
                sender_transfers: Some(include_str!("../../../testdata/fastnear_sender_transfers.json")),
                receiver_transfers: Some(include_str!("../../../testdata/fastnear_receiver_transfers.json")),
                transactions: Some(include_str!("../../../testdata/fastnear_transactions.json")),
            },
        );
        let indexer = NearIndexer::new(client.clone(), client);

        let transactions = indexer.get_transactions_by_address("address.near", 3, Some(1_700_000_000)).await.unwrap();
        let expected_sender_request: Value = serde_json::from_str(include_str!("../../../testdata/fastnear_sender_transfers_request.json")).unwrap();
        let expected_receiver_request: Value = serde_json::from_str(include_str!("../../../testdata/fastnear_receiver_transfers_request.json")).unwrap();
        let expected_transactions_request: Value = serde_json::from_str(include_str!("../../../testdata/fastnear_transactions_request.json")).unwrap();

        assert_eq!(
            transactions.iter().map(|transaction| transaction.hash.as_str()).collect::<Vec<_>>(),
            vec!["incoming-transaction", "outgoing-transaction", "attached-deposit-transaction"]
        );
        assert_eq!(
            transactions.iter().map(|transaction| transaction.value.as_str()).collect::<Vec<_>>(),
            vec!["2000000000000000000000000", "1000000000000000000000000", "500000000000000000000000"]
        );
        assert_eq!(transactions.iter().map(|transaction| transaction.fee.as_str()).collect::<Vec<_>>(), vec!["0", "70", "110"]);
        assert_eq!(*requests.transfers.lock().unwrap(), vec![expected_sender_request, expected_receiver_request]);
        assert_eq!(*requests.transactions.lock().unwrap(), vec![expected_transactions_request]);

        let client = mock_client(
            MockRequests::default(),
            MockResponses {
                sender_transfers: Some(include_str!("../../../testdata/fastnear_empty_transfers.json")),
                receiver_transfers: Some(include_str!("../../../testdata/fastnear_usdt_receiver_transfers.json")),
                ..Default::default()
            },
        );
        let indexer = NearIndexer::new(client.clone(), client);
        let token_transactions = indexer
            .get_transactions_by_address("bb90f7cd3f611466d4e8aaee55541d5da6881e01a4155bca49041c1d692b4ff8", 1, None)
            .await
            .unwrap();
        let token_transaction = token_transactions.first().unwrap();
        assert_eq!(token_transaction.hash, "DXUp65qSLjpbMrMVubtH1YY13fDHLA5av7q7skJ8kx5E");
        assert_eq!(token_transaction.value, "99500026");
        assert_eq!(token_transaction.fee, "0");
        assert_eq!(token_transaction.asset_id, NEAR_USDT_ASSET_ID.clone());
        assert_eq!(token_transaction.from, "e589457354361489a89039b8be6737cbc2db4d13919b6ccf23889a60f3b0d8f3");
        assert_eq!(token_transaction.to, "bb90f7cd3f611466d4e8aaee55541d5da6881e01a4155bca49041c1d692b4ff8");
    }

    #[tokio::test]
    async fn test_get_transactions_by_address_missing_details() {
        let client = mock_client(
            MockRequests::default(),
            MockResponses {
                sender_transfers: Some(include_str!("../../../testdata/fastnear_sender_transfers.json")),
                receiver_transfers: Some(include_str!("../../../testdata/fastnear_receiver_transfers.json")),
                transactions: Some(include_str!("../../../testdata/fastnear_empty_transactions.json")),
            },
        );
        let error = NearIndexer::new(client.clone(), client)
            .get_transactions_by_address("address.near", 3, Some(1_700_000_000))
            .await
            .unwrap_err();

        assert_eq!(error.to_string(), "missing FastNear sender transaction details");
    }

    #[tokio::test]
    async fn test_get_transaction_by_hash() {
        let requests = MockRequests::default();
        let client = mock_client(
            requests.clone(),
            MockResponses {
                transactions: Some(include_str!("../../../testdata/fastnear_usdt_transaction.json")),
                ..Default::default()
            },
        );
        let indexer = NearIndexer::new(client.clone(), client);
        let hash = "DXUp65qSLjpbMrMVubtH1YY13fDHLA5av7q7skJ8kx5E";
        let transaction = ChainTransaction::get_transaction_by_hash(&indexer, TransactionIdRequest::new(Chain::Near, hash.to_string(), Some(211048907)))
            .await
            .unwrap()
            .unwrap();
        let expected_request: Value = serde_json::from_str(include_str!("../../../testdata/fastnear_usdt_transaction_request.json")).unwrap();

        assert_usdt_transaction(&transaction);
        assert_eq!(*requests.transactions.lock().unwrap(), vec![expected_request]);

        let error = ChainTransaction::get_transaction_by_hash(&indexer, TransactionIdRequest::new(Chain::Near, hash.to_string(), Some(211048906)))
            .await
            .unwrap_err();
        assert_eq!(error.to_string(), "NEAR transaction block mismatch: expected 211048906, got 211048907");
    }
}
