use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use gem_auth::build_device_auth_header;
use gem_client::{Client, ClientError};
use primitives::{NFTAssetData, NFTAssetId, NFTData, ReportNft, ScanTransaction, ScanTransactionPayload};
use serde::de::DeserializeOwned;

use crate::device_target::GemDeviceApiTarget;
use crate::method::GemApiMethod;

/// Signs `/v2/devices/*` requests with the device Ed25519 key.
///
/// The key is passed in once at construction and is expected to stay in memory for
/// the life of the client. It is a device-scoped credential, unrelated to wallet keys
/// or recovery material, and is held this way deliberately so signing does not read
/// secure storage on every request.
#[derive(Debug, Clone)]
pub struct GemDeviceApiClient<C: Client> {
    client: C,
    device_private_key: Vec<u8>,
}

impl<C: Client> GemDeviceApiClient<C> {
    pub fn new(client: C, device_private_key: Vec<u8>) -> Self {
        Self { client, device_private_key }
    }

    pub async fn scan_transaction(&self, payload: ScanTransactionPayload) -> Result<ScanTransaction, ClientError> {
        self.send(GemDeviceApiTarget::ScanTransaction(payload)).await
    }

    pub async fn get_nft_assets(&self, wallet_id: String) -> Result<Vec<NFTData>, ClientError> {
        let assets: Option<Vec<NFTData>> = self.send(GemDeviceApiTarget::GetNftAssets(wallet_id)).await?;
        Ok(assets.unwrap_or_default())
    }

    pub async fn get_nft_asset(&self, asset_id: NFTAssetId) -> Result<NFTAssetData, ClientError> {
        self.send(GemDeviceApiTarget::GetNftAsset(asset_id)).await
    }

    pub async fn refresh_nft_asset(&self, wallet_id: String, asset_id: NFTAssetId) -> Result<bool, ClientError> {
        self.send(GemDeviceApiTarget::RefreshNftAsset(wallet_id, asset_id)).await
    }

    pub async fn report_nft(&self, report: ReportNft) -> Result<bool, ClientError> {
        self.send(GemDeviceApiTarget::ReportNft(report)).await
    }

    async fn send<R: DeserializeOwned>(&self, target: GemDeviceApiTarget) -> Result<R, ClientError> {
        let path = target.path();
        let body = target.body()?;
        let body_bytes = body.as_ref().map(|value| value.get().as_bytes()).unwrap_or_default();
        let headers = self.authorization(&target, body_bytes)?;

        match target.method() {
            GemApiMethod::Get => self.client.get_with(&path, &[], headers).await,
            GemApiMethod::Post => self.client.post_with(&path, &body, headers).await,
        }
    }

    fn authorization(&self, target: &GemDeviceApiTarget, body: &[u8]) -> Result<HashMap<String, String>, ClientError> {
        let path = target.path();
        let signed_path = path.split('?').next().unwrap_or(&path);
        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| ClientError::Serialization(error.to_string()))?
            .as_millis() as u64;

        let header = build_device_auth_header(&self.device_private_key, target.method().as_ref(), signed_path, target.wallet_id(), body, timestamp_ms)
            .map_err(|error| ClientError::Serialization(error.to_string()))?;

        Ok(HashMap::from([("Authorization".to_string(), header)]))
    }
}
