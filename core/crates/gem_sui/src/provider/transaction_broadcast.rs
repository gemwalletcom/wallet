use std::str;

use async_trait::async_trait;
use chain_traits::{ChainTransactionBroadcast, ChainTransactionDecode};

use primitives::BroadcastOptions;

use crate::{
    provider::{
        BroadcastProvider,
        transaction_broadcast_mapper::{
            map_transaction_broadcast_request, map_transaction_broadcast_response, map_transaction_broadcast_response_from_grpc, map_transaction_broadcast_response_from_str,
        },
    },
    rpc::client::SuiClient,
};

#[async_trait]
impl ChainTransactionBroadcast for SuiClient {
    async fn transaction_broadcast(&self, data: String, _options: BroadcastOptions) -> Result<String, Box<dyn std::error::Error + Sync + Send>> {
        let (transaction_data, signature) = map_transaction_broadcast_request(&data)?;
        let response = self.broadcast(transaction_data, signature).await?;
        map_transaction_broadcast_response(response)
    }
}

impl ChainTransactionDecode for BroadcastProvider {
    fn decode_transaction_broadcast(&self, response: &str) -> Option<String> {
        map_transaction_broadcast_response_from_str(response).ok()
    }

    fn decode_transaction_broadcast_bytes(&self, response: &[u8]) -> Option<String> {
        map_transaction_broadcast_response_from_grpc(response).ok().or_else(|| {
            str::from_utf8(response)
                .ok()
                .and_then(|response| map_transaction_broadcast_response_from_str(response).ok())
        })
    }
}
