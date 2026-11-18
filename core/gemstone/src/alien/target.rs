use std::collections::HashMap;

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
    pub fn new(status: Option<u16>, headers: HashMap<String, String>, data: Vec<u8>) -> Self {
        Self {
            response: RpcResponse {
                status,
                headers: headers.into_iter().map(|(name, value)| (name.to_ascii_lowercase(), value)).collect(),
                data,
            },
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alien::AlienProviderWrapper;
    use crate::testkit::TestAlienProvider;
    use futures::executor::block_on;
    use gem_jsonrpc::grpc::{AlienGrpcTransport, GrpcTransport};
    use std::sync::Arc;

    #[test]
    fn test_grpc_status_in_app_headers_reaches_the_grpc_transport() {
        let response = AlienResponse::new(
            Some(200),
            HashMap::from([
                ("Grpc-Status".to_string(), "3".to_string()),
                ("Grpc-Message".to_string(), "Balance%20of%20gas%20object%20is%20lower%20than%20the%20needed%20amount".to_string()),
            ]),
            Vec::new(),
        );
        let transport = AlienGrpcTransport::new(Arc::new(AlienProviderWrapper::new(Arc::new(TestAlienProvider::new(response)))));

        let error = block_on(transport.unary("https://example.com", "/sui.rpc.v2.TransactionExecutionService/SimulateTransaction", Vec::new())).unwrap_err();

        assert_eq!(error.to_string(), "gRPC error 3: Balance of gas object is lower than the needed amount");
    }
}
