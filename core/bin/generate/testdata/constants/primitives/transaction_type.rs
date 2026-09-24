#[typeshare(swift = "Equatable, Sendable")]
#[serde(rename_all = "camelCase")]
pub enum TransactionType {
    #[serde(rename = "transferNFT")]
    TransferNFT,
    Swap,
}
