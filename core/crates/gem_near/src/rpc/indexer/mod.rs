mod mapper;

use std::{
    cmp::Reverse,
    collections::{BTreeMap, BTreeSet, HashMap},
    error::Error,
    str::FromStr,
};

use futures::future::try_join_all;
use gem_client::{Client, ClientExt, build_request_url};
use num_bigint::BigUint;
use primitives::Transaction;
use serde::{Deserialize, Serialize};
use serde_serializers::deserialize_u64_from_str;

use self::mapper::map_transaction;

pub const FASTNEAR_URL: &str = "https://{service}.main.fastnear.com";

const NATIVE_ASSET_ID: &str = "native:near";
const MAX_TRANSFERS_LIMIT: usize = 100;
const TRANSACTIONS_BATCH_SIZE: usize = 20;

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
enum TransferDirection {
    Sender,
    Receiver,
}

#[derive(Debug, Serialize)]
struct TransfersRequest<'a> {
    account_id: &'a str,
    asset_id: &'static str,
    direction: TransferDirection,
    desc: bool,
    limit: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    from_timestamp_ms: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct TransfersResponse {
    transfers: Vec<FastNearTransfer>,
}

#[derive(Debug, Deserialize)]
pub(super) struct FastNearTransfer {
    pub amount: String,
    #[serde(deserialize_with = "deserialize_u64_from_str")]
    pub block_timestamp: u64,
    pub predecessor_id: String,
    pub receipt_account_id: String,
    pub receipt_id: String,
    pub signer_id: String,
    pub transaction_id: Option<String>,
}

#[derive(Debug, Serialize)]
struct TransactionsRequest<'a> {
    tx_hashes: &'a [String],
}

#[derive(Debug, Deserialize)]
struct TransactionsResponse {
    transactions: Vec<FastNearTransaction>,
}

#[derive(Debug, Deserialize)]
struct FastNearTransaction {
    execution_outcome: FastNearExecutionOutcome,
    receipts: Vec<FastNearReceipt>,
    transaction: FastNearSignedTransaction,
}

#[derive(Debug, Deserialize)]
struct FastNearSignedTransaction {
    hash: String,
}

#[derive(Debug, Deserialize)]
struct FastNearReceipt {
    execution_outcome: FastNearExecutionOutcome,
}

#[derive(Debug, Deserialize)]
struct FastNearExecutionOutcome {
    outcome: FastNearOutcome,
}

#[derive(Debug, Deserialize)]
struct FastNearOutcome {
    tokens_burnt: String,
}

impl FastNearExecutionOutcome {
    fn tokens_burnt(&self) -> Result<BigUint, Box<dyn Error + Send + Sync>> {
        Ok(BigUint::from_str(&self.outcome.tokens_burnt)?)
    }
}

impl FastNearTransaction {
    fn fee(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        let transaction_fee = self.execution_outcome.tokens_burnt()?;
        self.receipts
            .iter()
            .try_fold(transaction_fee, |fee, receipt| Ok(fee + receipt.execution_outcome.tokens_burnt()?))
            .map(|fee: BigUint| fee.to_string())
    }
}

#[derive(Clone, Debug)]
pub struct NearIndexer<C: Client> {
    client: C,
    url: String,
}

impl<C: Client> NearIndexer<C> {
    pub fn new(client: C, url: String) -> Self {
        Self { client, url }
    }

    fn service_url(&self, service: &str, path: &str) -> String {
        build_request_url(&self.url.replace("{service}", service), path)
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
            asset_id: NATIVE_ASSET_ID,
            direction,
            desc: true,
            limit: limit.min(MAX_TRANSFERS_LIMIT),
            from_timestamp_ms,
        };
        let response: TransfersResponse = self.client.post(&self.service_url("transfers", "/v0/transfers"), &request).await?;
        Ok(response.transfers)
    }

    async fn get_transaction_fees(&self, transfers: &[FastNearTransfer], address: &str) -> Result<HashMap<String, String>, Box<dyn Error + Send + Sync>> {
        let transaction_ids = transfers
            .iter()
            .filter(|transfer| transfer.signer_id == address && transfer.predecessor_id == address)
            .filter_map(|transfer| transfer.transaction_id.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let requests = transaction_ids.chunks(TRANSACTIONS_BATCH_SIZE).map(|transaction_ids| async move {
            let request = TransactionsRequest { tx_hashes: transaction_ids };
            let response: TransactionsResponse = self.client.post(&self.service_url("tx", "/v0/transactions"), &request).await?;
            response
                .transactions
                .into_iter()
                .map(|transaction| {
                    let fee = transaction.fee()?;
                    Ok((transaction.transaction.hash, fee))
                })
                .collect::<Result<Vec<_>, Box<dyn Error + Send + Sync>>>()
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
        transfers.into_iter().map(|transfer| map_transaction(transfer, &fees, address)).collect()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use gem_client::testkit::MockClient;
    use serde_json::Value;

    use super::*;

    fn client(transfer_requests: Arc<Mutex<Vec<Value>>>, transaction_requests: Arc<Mutex<Vec<Value>>>, transactions_response: &'static str) -> MockClient {
        MockClient::new().with_post(move |path, body| {
            let request = serde_json::from_slice::<Value>(body).unwrap();
            match path {
                "https://transfers.main.fastnear.com/v0/transfers" => {
                    let response = match request["direction"].as_str().unwrap() {
                        "sender" => include_str!("../../../testdata/fastnear_sender_transfers.json"),
                        "receiver" => include_str!("../../../testdata/fastnear_receiver_transfers.json"),
                        direction => panic!("unexpected transfer direction: {direction}"),
                    };
                    transfer_requests.lock().unwrap().push(request);
                    Ok(response.as_bytes().to_vec())
                }
                "https://tx.main.fastnear.com/v0/transactions" => {
                    transaction_requests.lock().unwrap().push(request);
                    Ok(transactions_response.as_bytes().to_vec())
                }
                path => panic!("unexpected path: {path}"),
            }
        })
    }

    #[tokio::test]
    async fn test_get_transactions_by_address() {
        let transfer_requests = Arc::new(Mutex::new(Vec::new()));
        let transaction_requests = Arc::new(Mutex::new(Vec::new()));
        let indexer = NearIndexer::new(
            client(
                transfer_requests.clone(),
                transaction_requests.clone(),
                include_str!("../../../testdata/fastnear_transactions.json"),
            ),
            FASTNEAR_URL.to_string(),
        );

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
        assert_eq!(*transfer_requests.lock().unwrap(), vec![expected_sender_request, expected_receiver_request]);
        assert_eq!(*transaction_requests.lock().unwrap(), vec![expected_transactions_request]);
    }

    #[tokio::test]
    async fn test_get_transactions_by_address_missing_details() {
        let error = NearIndexer::new(
            client(
                Arc::new(Mutex::new(Vec::new())),
                Arc::new(Mutex::new(Vec::new())),
                include_str!("../../../testdata/fastnear_empty_transactions.json"),
            ),
            FASTNEAR_URL.to_string(),
        )
        .get_transactions_by_address("address.near", 3, Some(1_700_000_000))
        .await
        .unwrap_err();

        assert_eq!(error.to_string(), "missing FastNear sender transaction details");
    }
}
