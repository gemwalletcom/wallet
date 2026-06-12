use crate::GemstoneError;
use gem_evm::call_decoder;

pub type GemDecodedCall = call_decoder::DecodedCall;
pub type GemDecodedCallParam = call_decoder::DecodedCallParam;

#[uniffi::remote(Record)]
pub struct GemDecodedCall {
    pub function: String,
    pub params: Vec<GemDecodedCallParam>,
}

#[uniffi::remote(Record)]
pub struct GemDecodedCallParam {
    pub name: String,
    pub r#type: String,
    pub value: String,
}

#[derive(Debug, Default, uniffi::Object)]
pub struct EthereumDecoder;

#[uniffi::export]
impl EthereumDecoder {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self {}
    }

    pub fn decode_call(&self, calldata: String, abi: Option<String>) -> Result<GemDecodedCall, GemstoneError> {
        call_decoder::decode_call(&calldata, abi.as_deref()).map_err(GemstoneError::from)
    }
}
