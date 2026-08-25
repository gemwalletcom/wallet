use std::collections::HashMap;

pub use gem_client::X_CACHE_TTL;
use gem_jsonrpc::RpcResponse;
pub type AlienTarget = swapper::Target;
pub type AlienHttpMethod = swapper::HttpMethod;

#[uniffi::remote(Record)]
pub struct AlienTarget {
    pub url: String,
    pub method: AlienHttpMethod,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<Vec<u8>>,
}

#[derive(Debug, uniffi::Object)]
pub struct AlienResponse {
    response: RpcResponse,
}

#[uniffi::export]
impl AlienResponse {
    #[uniffi::constructor]
    pub fn new(status: Option<u16>, data: Vec<u8>) -> Self {
        Self {
            response: RpcResponse { status, data },
        }
    }
}

impl AlienResponse {
    pub(crate) fn to_rpc_response(&self) -> RpcResponse {
        self.response.clone()
    }
}

#[uniffi::remote(Enum)]
pub enum AlienHttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Head,
    Options,
    Patch,
}

#[uniffi::export]
fn alien_method_to_string(method: AlienHttpMethod) -> String {
    method.into()
}
