use gem_jsonrpc::types::ToJsonRpcRequest;
use serde_json::Value;

const REQUEST_SWAP_PARAMETER_ENCODING: &str = "broker_request_swap_parameter_encoding";

#[derive(Clone, Debug)]
pub(super) struct RequestSwapParameterEncoding(pub Value);

impl ToJsonRpcRequest for RequestSwapParameterEncoding {
    fn method(&self) -> &'static str {
        REQUEST_SWAP_PARAMETER_ENCODING
    }

    fn params(&self) -> Value {
        self.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn request_swap_parameter_encoding_preserves_positional_params() {
        let request = RequestSwapParameterEncoding(json!(["Ethereum", "ETH", "0x1234"])).to_jsonrpc_request(7);

        assert_eq!(request.id, 7);
        assert_eq!(request.method, REQUEST_SWAP_PARAMETER_ENCODING);
        assert_eq!(request.params, json!(["Ethereum", "ETH", "0x1234"]));
    }
}
