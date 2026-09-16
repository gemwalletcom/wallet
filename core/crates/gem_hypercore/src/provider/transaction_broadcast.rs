use async_trait::async_trait;
use chain_traits::{ChainTransactionBroadcast, ChainTransactionDecode};
use std::error::Error;

use gem_client::Client;
use primitives::BroadcastOptions;

use crate::{
    provider::{BroadcastProvider, transactions_mapper::map_transaction_broadcast},
    rpc::client::HyperCoreClient,
};

#[async_trait]
impl<C: Client> ChainTransactionBroadcast for HyperCoreClient<C> {
    async fn transaction_broadcast(&self, data: String, _options: BroadcastOptions) -> Result<String, Box<dyn Error + Sync + Send>> {
        let request: serde_json::Value = serde_json::from_str(&data)?;
        let response = self.exchange(request).await?;
        map_transaction_broadcast(data.as_bytes(), response)
    }
}

impl ChainTransactionDecode for BroadcastProvider {
    fn decode_transaction_broadcast(&self, request: &[u8], response: &str) -> Result<String, Box<dyn Error + Sync + Send>> {
        map_transaction_broadcast(request, serde_json::from_str(response)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_transaction_broadcast() {
        let provider = BroadcastProvider;
        let response = r#"{"status":"ok","response":{"type":"default"}}"#;
        for request in [
            r#"{"action":{"type":"approveAgent"},"nonce":123}"#,
            r#"{"action":{"type":"approveBuilderFee"},"nonce":123}"#,
            r#"{"action":{"type":"setReferrer"},"nonce":123}"#,
            r#"{"action":{"type":"updateLeverage"},"nonce":123}"#,
            r#"{"action":{"type":"spotSend"},"nonce":123}"#,
            r#"{"action":{"type":"withdraw3"},"nonce":123}"#,
            r#"{"action":{"type":"usdClassTransfer"},"nonce":123}"#,
            r#"{"action":{"type":"usdSend"},"nonce":123}"#,
        ] {
            assert_eq!(provider.decode_transaction_broadcast(request.as_bytes(), response).unwrap(), "action:123");
        }
        for (request, expected) in [
            (
                include_bytes!("../../testdata/hl_action_spot_to_stake.json").as_slice(),
                "action:cDeposit:10000000:1755231476741",
            ),
            (
                include_bytes!("../../testdata/hl_action_stake_to_validator.json").as_slice(),
                "action:tokenDelegate:10000000:stake:1755231522831",
            ),
            (br#"{"action":{"type":"cWithdraw","wei":100},"nonce":123}"#.as_slice(), "action:cWithdraw:100:123"),
            (
                br#"{"action":{"type":"tokenDelegate","wei":100,"isUndelegate":true},"nonce":123}"#.as_slice(),
                "action:tokenDelegate:100:unstake:123",
            ),
        ] {
            assert_eq!(provider.decode_transaction_broadcast_bytes(request, response.as_bytes()).unwrap(), expected);
        }
        let request = include_bytes!("../../testdata/hl_action_update_position_tp_sl.json");
        for response in [
            r#"{"status":"ok","response":{"type":"order","data":{"statuses":["waitingForTrigger","waitingForTrigger"]}}}"#,
            r#"{"status":"ok","response":{"type":"order","data":{"statuses":["waitingForFill"]}}}"#,
        ] {
            assert_eq!(provider.decode_transaction_broadcast(request, response).unwrap(), "action:order:1755132472149");
        }
        let request = br#"{"action":{"type":"cancel","cancels":[{"a":1,"o":123}]},"nonce":123}"#;
        let response = r#"{"status":"ok","response":{"type":"cancel","data":{"statuses":["success"]}}}"#;
        assert_eq!(provider.decode_transaction_broadcast(request, response).unwrap(), "action:123");
    }
}
