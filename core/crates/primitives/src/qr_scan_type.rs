use model_derive::Model;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Model)]
#[model(swift = "Equatable, CaseIterable, Sendable")]
#[serde(rename_all = "camelCase")]
pub enum QRScanType {
    Universal,
    WalletConnect,
    Address,
    Memo,
    Url,
    TokenContract,
    SecretPhrase,
    PrivateKey,
}
