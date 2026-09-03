use crate::{AddressName, Asset, AssetPrice, Price, Transaction};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable, Equatable, Hashable")]
pub struct TransactionExtended {
    pub transaction: Transaction,
    pub asset: Asset,
    #[serde(rename = "feeAsset")]
    pub fee_asset: Asset,
    pub price: Option<Price>,
    #[serde(rename = "feePrice")]
    pub fee_price: Option<Price>,
    pub assets: Vec<Asset>,
    pub prices: Vec<AssetPrice>,
    #[serde(rename = "fromAddress")]
    pub from_address: Option<AddressName>,
    #[serde(rename = "toAddress")]
    pub to_address: Option<AddressName>,
    #[serde(rename = "confirmationEtaSeconds")]
    pub confirmation_eta_seconds: Option<u32>,
}
