use serde_json::Value;

use super::{ContractCall, ContractCallDecoder, ContractRequest};

pub(crate) const ETH_CALL: &str = "eth_call";

pub(super) struct EvmContractCallDecoder;

impl ContractCallDecoder for EvmContractCallDecoder {
    fn decode_contract_calls(&self, request: ContractRequest<'_>) -> Option<Vec<ContractCall>> {
        let ContractRequest::JsonRpc(call) = request else {
            return None;
        };
        if call.method != ETH_CALL {
            return None;
        }

        let params = call.params.as_array()?;
        if params.len() != 2 || params.get(1).and_then(Value::as_str) != Some("latest") {
            return None;
        }

        let call = params.first()?.as_object()?;
        if call.len() != 2 {
            return None;
        }

        Some(vec![ContractCall {
            address: call.get("to")?.as_str()?.to_string(),
            identifier: call.get("data")?.as_str()?.get(..10)?.to_string(),
        }])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jsonrpc_types::JsonRpcCall;

    const CONTRACT: &str = "0x1111111111111111111111111111111111111111";

    fn call(data: &str, block: &str) -> JsonRpcCall {
        JsonRpcCall::mock_with_params(1, ETH_CALL, serde_json::json!([{ "to": CONTRACT, "data": data }, block]))
    }

    #[test]
    fn test_decode_contract_calls() {
        let call = call("0x1698ee820000", "latest");
        let calls = EvmContractCallDecoder.decode_contract_calls(ContractRequest::JsonRpc(&call)).unwrap();

        assert_eq!(
            calls,
            vec![ContractCall {
                address: CONTRACT.to_string(),
                identifier: "0x1698ee82".to_string(),
            }]
        );
    }

    #[test]
    fn test_rejects_unsupported_calls() {
        let pending = call("0x1698ee820000", "pending");
        let short_data = call("0x1698", "latest");
        let extra_field = JsonRpcCall::mock_with_params(1, ETH_CALL, serde_json::json!([{ "to": CONTRACT, "data": "0x1698ee82", "value": "0x1" }, "latest"]));

        assert!(EvmContractCallDecoder.decode_contract_calls(ContractRequest::JsonRpc(&pending)).is_none());
        assert!(EvmContractCallDecoder.decode_contract_calls(ContractRequest::JsonRpc(&short_data)).is_none());
        assert!(EvmContractCallDecoder.decode_contract_calls(ContractRequest::JsonRpc(&extra_field)).is_none());
    }
}
