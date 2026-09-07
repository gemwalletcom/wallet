#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferDataExtra {
    pub chain: Chain,
    pub gas_price: Option<GasPriceType>,
    pub data: Option<Vec<u8>>,
    pub gas_limit: Option<BigInt>,
}
