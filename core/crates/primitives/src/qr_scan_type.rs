use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[typeshare(swift = "Equatable, CaseIterable, Sendable")]
#[serde(rename_all = "camelCase")]
pub enum QRScanType {
    Universal,
    WalletConnect,
    Address,
    Memo,
    NodeUrl,
    TokenContract,
    SecretPhrase,
    PrivateKey,
}
