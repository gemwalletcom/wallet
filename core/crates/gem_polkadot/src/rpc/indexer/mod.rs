mod mapper;

use std::error::Error;

use gem_client::{Client, ClientExt};
use num_bigint::BigUint;
use number_formatter::BigNumberFormatter;
use primitives::Transaction;
use serde::{Deserialize, Serialize, de};

use self::mapper::map_transaction;
use crate::rpc::target::SubscanTarget;

const NATIVE_ASSET_SYMBOL: &str = "DOT";
const MAX_TRANSFERS_LIMIT: usize = 25;
const POLKADOT_DECIMALS: u32 = 10;

fn deserialize_dot_amount<'de, D>(deserializer: D) -> Result<BigUint, D::Error>
where
    D: de::Deserializer<'de>,
{
    let amount = String::deserialize(deserializer)?;
    BigNumberFormatter::value_from_amount_biguint(&amount, POLKADOT_DECIMALS).map_err(de::Error::custom)
}

#[derive(Debug, Serialize)]
struct TransfersRequest<'a> {
    address: &'a str,
    asset_symbol: &'static str,
    direction: &'static str,
    order: &'static str,
    page: usize,
    row: usize,
}

#[derive(Debug, Deserialize)]
struct SubscanResponse {
    data: TransfersData,
}

#[derive(Debug, Deserialize)]
struct TransfersData {
    transfers: Option<Vec<SubscanTransfer>>,
}

#[derive(Debug, Deserialize)]
pub(super) struct SubscanTransfer {
    #[serde(deserialize_with = "deserialize_dot_amount")]
    pub amount: BigUint,
    pub block_timestamp: i64,
    #[serde(deserialize_with = "deserialize_dot_amount")]
    pub fee: BigUint,
    pub from: String,
    pub hash: String,
    pub success: bool,
    pub to: String,
}

#[derive(Clone, Debug)]
pub struct PolkadotIndexer<C: Client> {
    client: C,
}

impl<C: Client> PolkadotIndexer<C> {
    pub fn new(client: C) -> Self {
        Self { client }
    }

    async fn get_transfers(&self, address: &str, limit: usize, from_timestamp: Option<u64>) -> Result<Vec<SubscanTransfer>, Box<dyn Error + Send + Sync>> {
        let from_timestamp = from_timestamp.map(i64::try_from).transpose()?;
        let request = TransfersRequest {
            address,
            asset_symbol: NATIVE_ASSET_SYMBOL,
            direction: "all",
            order: "desc",
            page: 0,
            row: limit.min(MAX_TRANSFERS_LIMIT),
        };
        let response: SubscanResponse = self.client.post(SubscanTarget::Transfers, &request).await?;
        Ok(response
            .data
            .transfers
            .unwrap_or_default()
            .into_iter()
            .filter(|transfer| from_timestamp.is_none_or(|timestamp| transfer.block_timestamp >= timestamp))
            .collect())
    }

    pub(crate) async fn get_transactions_by_address(&self, address: &str, limit: usize, from_timestamp: Option<u64>) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        self.get_transfers(address, limit, from_timestamp).await?.into_iter().map(map_transaction).collect()
    }
}

#[cfg(test)]
mod tests {
    use gem_client::testkit::MockClient;
    use serde_json::Value;

    use super::*;

    #[tokio::test]
    async fn test_get_transactions_by_address() {
        let client = MockClient::new().with_post(move |path, body| {
            assert_eq!(path, "/api/v2/scan/transfers");
            let request: Value = serde_json::from_slice(body).unwrap();
            let expected_request: Value = serde_json::from_str(include_str!("../../../testdata/subscan_transfers_request.json")).unwrap();
            assert_eq!(request, expected_request);
            Ok(include_str!("../../../testdata/subscan_asset_hub_transfers.json").as_bytes().to_vec())
        });

        let transactions = PolkadotIndexer::new(client).get_transactions_by_address("address", 100, None).await.unwrap();

        assert_eq!(
            transactions.iter().map(|transaction| transaction.hash()).collect::<Vec<_>>(),
            vec!["asset-hub-newest", "asset-hub-older"]
        );
        assert_eq!(
            transactions.iter().map(|transaction| transaction.value.to_string()).collect::<Vec<_>>(),
            vec!["2500000000", "20000000000"]
        );
        assert_eq!(
            transactions.iter().map(|transaction| transaction.fee.to_string()).collect::<Vec<_>>(),
            vec!["100000000", "50000000"]
        );

        let empty_client = MockClient::new().with_post(|path, _| {
            assert_eq!(path, "/api/v2/scan/transfers");
            Ok(include_str!("../../../testdata/subscan_empty_transfers.json").as_bytes().to_vec())
        });

        let empty_transactions = PolkadotIndexer::new(empty_client).get_transactions_by_address("address", 100, None).await.unwrap();

        assert_eq!(empty_transactions.len(), 0);
    }
}
