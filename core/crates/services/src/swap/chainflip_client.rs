use std::{error::Error, sync::Arc};

use cacher::SwapVaultAddressCacher;
use gem_jsonrpc::types::ToJsonRpcRequest;
use primitives::SwapProvider;
use swapper::chainflip::broker::{DepositChannelRequest, DepositChannelResponse};
use swapper::chainflip::chain::ChainflipChain;

use super::proxy_client::SwapProxyClient;

pub struct ChainflipProxyClient {
    client: SwapProxyClient,
}

impl ChainflipProxyClient {
    pub fn new(url: String, deposit_addresses: Arc<dyn SwapVaultAddressCacher>) -> Self {
        Self {
            client: SwapProxyClient::new(url, deposit_addresses, SwapProvider::Chainflip, &[]),
        }
    }

    pub async fn deposit_channel(&self, request: &DepositChannelRequest) -> Result<gem_client::Response, Box<dyn Error + Send + Sync>> {
        validate_source(request)?;
        let response = self.client.post("", &request.to_jsonrpc_request(1)).await?;
        if let Some(deposit_address) = DepositChannelResponse::from_jsonrpc(&response.data).and_then(|channel| channel.deposit_address().ok()) {
            self.client.cache_deposit_id(deposit_address).await;
        }
        Ok(response)
    }
}

fn validate_source(request: &DepositChannelRequest) -> Result<(), Box<dyn Error + Send + Sync>> {
    if request.source_asset.chain != ChainflipChain::Assethub.as_ref() {
        return Err(format!("unsupported Chainflip deposit channel source: {}", request.source_asset.chain).into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_source() {
        let request = DepositChannelRequest::mock();
        assert!(validate_source(&request).is_ok());

        let ethereum = DepositChannelRequest {
            source_asset: request.destination_asset.clone(),
            ..request
        };
        assert!(validate_source(&ethereum).is_err());
    }
}
