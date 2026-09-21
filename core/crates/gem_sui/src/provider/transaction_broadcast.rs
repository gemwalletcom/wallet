use std::{error::Error, str};

use async_trait::async_trait;
use chain_traits::{ChainTransactionBroadcast, ChainTransactionDecode};

use primitives::BroadcastOptions;

use crate::{
    provider::{
        BroadcastProvider,
        transaction_broadcast_mapper::{map_transaction_broadcast_request, map_transaction_broadcast_response, map_transaction_broadcast_response_from_grpc, map_transaction_broadcast_response_from_str},
    },
    rpc::SuiProvider,
};

#[async_trait]
impl ChainTransactionBroadcast for SuiProvider {
    async fn transaction_broadcast(&self, data: String, _options: BroadcastOptions) -> Result<String, Box<dyn Error + Sync + Send>> {
        let (transaction_data, signature) = map_transaction_broadcast_request(&data)?;
        let response = self.broadcast(transaction_data, signature).await?;
        map_transaction_broadcast_response(response)
    }
}

impl ChainTransactionDecode for BroadcastProvider {
    fn decode_transaction_broadcast(&self, _request: &[u8], response: &str) -> Result<String, Box<dyn Error + Sync + Send>> {
        map_transaction_broadcast_response_from_str(response)
    }

    fn decode_transaction_broadcast_bytes(&self, request: &[u8], response: &[u8]) -> Result<String, Box<dyn Error + Sync + Send>> {
        if let Ok(response) = str::from_utf8(response)
            && response.trim_start().starts_with('{')
        {
            self.decode_transaction_broadcast(request, response)
        } else {
            map_transaction_broadcast_response_from_grpc(response)
        }
    }
}
