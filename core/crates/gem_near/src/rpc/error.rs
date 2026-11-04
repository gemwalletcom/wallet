use gem_jsonrpc::types::TypedJsonRpcError;
use serde::Deserialize;

pub type NearRpcError = TypedJsonRpcError<NearErrorCause>;

#[derive(Debug, Default, Deserialize)]
pub struct NearErrorCause {
    cause: Option<NearErrorName>,
}

#[derive(Debug, Deserialize)]
struct NearErrorName {
    name: String,
}

impl NearErrorCause {
    pub fn is_unknown_account(&self) -> bool {
        self.cause.as_ref().is_some_and(|cause| cause.name == "UNKNOWN_ACCOUNT")
    }

    pub fn is_unknown_block(&self) -> bool {
        self.cause.as_ref().is_some_and(|cause| cause.name == "UNKNOWN_BLOCK")
    }
}
