use std::error::Error;

use gem_encoding::protobuf::{decode_grpc_message, encode_grpc_message};
use gem_jsonrpc::grpc::{GrpcStatusError, GrpcTransport, ReqwestGrpcTransport};

use super::model::{LookupNameRequest, LookupNameResponse};

const LOOKUP_NAME_PATH: &str = "/sui.rpc.v2.NameService/LookupName";

pub struct SuinsClient {
    url: String,
    transport: ReqwestGrpcTransport,
}

impl SuinsClient {
    pub fn new(url: String) -> Self {
        Self { url, transport: ReqwestGrpcTransport::new() }
    }

    pub async fn lookup_name(&self, name: &str) -> Result<Option<LookupNameResponse>, Box<dyn Error + Send + Sync>> {
        let request = LookupNameRequest { name: Some(name.to_string()) };
        match self.transport.unary(&self.url, LOOKUP_NAME_PATH, encode_grpc_message(&request)).await {
            Ok(response) => Ok(Some(decode_grpc_message(&response)?)),
            Err(error) if error.downcast_ref::<GrpcStatusError>().is_some_and(GrpcStatusError::is_not_found) => Ok(None),
            Err(error) => Err(error),
        }
    }
}
