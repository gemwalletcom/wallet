use cacher::{CacheKey, CacherClient};
use primitives::ChainAddress;
use rocket::serde::json::Json;
use rocket::{State, post};
use streamer::{ChainAddressPayload, StreamProducer, StreamProducerQueue};

use crate::api_clients::PermissionAdminWrite;
use crate::responders::{ApiError, ApiResponse};

#[post("/addresses/refresh", format = "json", data = "<addresses>")]
pub async fn refresh_addresses(
    _permission: PermissionAdminWrite,
    addresses: Json<Vec<ChainAddress>>,
    cacher: &State<CacherClient>,
    stream_producer: &State<StreamProducer>,
) -> Result<ApiResponse<Vec<ChainAddress>>, ApiError> {
    let addresses = addresses.into_inner();
    let cache_keys = addresses.iter().flat_map(refresh_cache_keys).collect::<Vec<_>>();
    cacher.delete_keys(&cache_keys).await?;

    let payload = addresses.iter().cloned().map(ChainAddressPayload::from).collect();
    stream_producer.publish_new_addresses(payload).await?;
    Ok(addresses.into())
}

fn refresh_cache_keys(address: &ChainAddress) -> [String; 4] {
    let chain = address.chain.as_ref();
    [
        CacheKey::FetchCoinAddresses(chain, &address.address).key(),
        CacheKey::FetchTokenAddresses(chain, &address.address).key(),
        CacheKey::FetchNftAssetsAddresses(chain, &address.address).key(),
        CacheKey::FetchAddressTransactions(chain, &address.address).key(),
    ]
}

#[cfg(test)]
mod tests {
    use primitives::{Chain, ChainAddress};

    use super::refresh_cache_keys;

    #[test]
    fn test_refresh_cache_keys() {
        let address = ChainAddress::new(Chain::Ethereum, "0x123".to_string());

        assert_eq!(
            refresh_cache_keys(&address),
            [
                "fetch:coin_addresses:ethereum:0x123",
                "fetch:token_addresses:ethereum:0x123",
                "fetch:nft_assets_addresses:ethereum:0x123",
                "fetch:address_transactions:ethereum:0x123",
            ]
        );
    }
}
