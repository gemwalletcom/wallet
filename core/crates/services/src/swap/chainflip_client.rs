use std::error::Error;

use cacher::CacherClient;
use gem_jsonrpc::types::ToJsonRpcRequest;
use primitives::{Chain, RequestError, SwapProvider};
use swapper::chainflip::broker::DepositChannelRequest;

use super::proxy_client::SwapProxyClient;

pub struct ChainflipProxyClient {
    client: SwapProxyClient,
}

impl ChainflipProxyClient {
    pub fn new(url: String, deposit_addresses: CacherClient) -> Self {
        Self {
            client: SwapProxyClient::new(url, deposit_addresses, SwapProvider::Chainflip),
        }
    }

    pub async fn deposit_channel(&self, request: &DepositChannelRequest) -> Result<gem_client::Response, Box<dyn Error + Send + Sync>> {
        validate_assets(request)?;
        Ok(self.client.post("", &request.to_jsonrpc_request(1)).await?)
    }
}

fn validate_assets(request: &DepositChannelRequest) -> Result<(), RequestError> {
    let source = request.source_asset.asset_id().ok_or(RequestError::Invalid)?;
    let destination = request.destination_asset.asset_id().ok_or(RequestError::Invalid)?;
    if source.chain != Chain::Polkadot || source == destination {
        return Err(RequestError::Invalid);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use swapper::chainflip::broker::ChainflipAsset;

    use super::*;

    #[test]
    fn test_validate_assets() {
        let request = DepositChannelRequest::mock();
        assert_eq!(validate_assets(&request), Ok(()));

        for source_asset in [
            request.destination_asset.clone(),
            ChainflipAsset::mock("Assethub", "ETH"),
            ChainflipAsset::mock("assethub", "DOT"),
            ChainflipAsset::mock("Assethub ", "DOT"),
        ] {
            let request = DepositChannelRequest { source_asset, ..request.clone() };
            assert_eq!(validate_assets(&request), Err(RequestError::Invalid));
        }
        for destination_asset in [request.source_asset.clone(), ChainflipAsset::mock("Ethereum", "DOT"), ChainflipAsset::mock("Assethub", "ETH")] {
            let request = DepositChannelRequest { destination_asset, ..request.clone() };
            assert_eq!(validate_assets(&request), Err(RequestError::Invalid));
        }
    }
}
