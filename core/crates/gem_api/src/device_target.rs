use gem_client::ClientError;
use primitives::{NFTAssetId, ReportNft, ScanTransactionPayload};
use serde_json::value::{RawValue, to_raw_value};

use crate::method::GemApiMethod;

#[derive(Clone, Debug)]
pub enum GemDeviceApiTarget {
    ScanTransaction(ScanTransactionPayload),
    GetNftAssets(String),
    GetNftAsset(NFTAssetId),
    RefreshNftAsset(String, NFTAssetId),
    ReportNft(ReportNft),
}

impl GemDeviceApiTarget {
    pub fn method(&self) -> GemApiMethod {
        match self {
            Self::GetNftAssets(_) | Self::GetNftAsset(_) => GemApiMethod::Get,
            Self::ScanTransaction(_) | Self::RefreshNftAsset(_, _) | Self::ReportNft(_) => GemApiMethod::Post,
        }
    }

    pub fn path(&self) -> String {
        match self {
            Self::ScanTransaction(_) => "/v2/devices/scan/transaction".to_string(),
            Self::GetNftAssets(_) => "/v2/devices/nft_assets".to_string(),
            Self::GetNftAsset(asset_id) => format!("/v2/devices/nft_assets/{asset_id}"),
            Self::RefreshNftAsset(_, asset_id) => format!("/v2/devices/nft_assets/{asset_id}/refresh"),
            Self::ReportNft(_) => "/v2/devices/nft/report".to_string(),
        }
    }

    pub fn wallet_id(&self) -> &str {
        match self {
            Self::GetNftAssets(wallet_id) | Self::RefreshNftAsset(wallet_id, _) => wallet_id.as_str(),
            Self::ScanTransaction(_) | Self::GetNftAsset(_) | Self::ReportNft(_) => "",
        }
    }

    pub fn body(&self) -> Result<Option<Box<RawValue>>, ClientError> {
        match self {
            Self::ScanTransaction(payload) => Ok(Some(to_raw_value(payload)?)),
            Self::ReportNft(report) => Ok(Some(to_raw_value(report)?)),
            Self::GetNftAssets(_) | Self::GetNftAsset(_) | Self::RefreshNftAsset(_, _) => Ok(None),
        }
    }
}
