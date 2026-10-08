use std::sync::Arc;

use ::nft::NFTProviderClient;
use axum::extract::State;
use primitives::{NFTAsset, NFTCollection, NFTData};

use crate::auth::api_client::ChainRead;
use crate::error::ApiError;
use crate::request::{AddressParam, ChainParam, NftAssetIdParam, NftCollectionIdParam, Path};
use crate::response::ApiResponse;

pub async fn get_nft_asset(_permission: ChainRead, Path(asset_id): Path<NftAssetIdParam>, State(client): State<Arc<NFTProviderClient>>) -> Result<ApiResponse<NFTAsset>, ApiError> {
    Ok(client.get_nft_asset(asset_id.0).await?.into())
}

pub async fn get_nft_collection(_permission: ChainRead, Path(collection_id): Path<NftCollectionIdParam>, State(client): State<Arc<NFTProviderClient>>) -> Result<ApiResponse<NFTCollection>, ApiError> {
    Ok(client.get_nft_collection(collection_id.0).await?.into())
}

pub async fn get_nfts(_permission: ChainRead, Path((chain, address)): Path<(ChainParam, AddressParam)>, State(client): State<Arc<NFTProviderClient>>) -> Result<ApiResponse<Vec<NFTData>>, ApiError> {
    Ok(client.get_nft_data(chain.0, &address.0).await?.into())
}
