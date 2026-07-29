use gem_encoding::protobuf::{MessageDecode, MessageResult, decode_grpc_frame, proto_decode};

use super::{ContractCall, ContractCallDecoder, ContractRequest};

const SIMULATE_TRANSACTION_PATH: &str = "/sui.rpc.v2.TransactionExecutionService/SimulateTransaction";

pub(super) struct SuiContractCallDecoder;

impl ContractCallDecoder for SuiContractCallDecoder {
    fn decode_contract_calls(&self, request: ContractRequest<'_>) -> Option<Vec<ContractCall>> {
        let ContractRequest::Http { path, method, body } = request else {
            return None;
        };
        if method != "POST" || path != SIMULATE_TRANSACTION_PATH {
            return None;
        }
        decode_simulation(body).ok()
    }
}

fn decode_simulation(body: &[u8]) -> MessageResult<Vec<ContractCall>> {
    let request = SimulationRequest::decode(decode_grpc_frame(body)?)?;
    let transaction = request.transaction.ok_or("missing Sui simulation transaction")?;
    transaction
        .kind
        .ok_or("missing Sui transaction kind")?
        .programmable_transaction
        .ok_or("missing Sui programmable transaction")?
        .commands
        .into_iter()
        .map(|command| {
            let call = command.move_call.ok_or("non-Move Sui command is not cacheable")?;
            Ok(ContractCall {
                address: call.package.ok_or("missing Sui package")?,
                identifier: format!("{}::{}", call.module.ok_or("missing Sui module")?, call.function.ok_or("missing Sui function")?),
            })
        })
        .collect()
}

#[derive(Debug, Default)]
struct SimulationRequest {
    transaction: Option<Transaction>,
}

proto_decode!(SimulationRequest {
    1 => transaction: optional_message,
});

#[derive(Debug, Default)]
struct Transaction {
    kind: Option<TransactionKind>,
}

proto_decode!(Transaction {
    4 => kind: optional_message,
});

#[derive(Debug, Default)]
struct TransactionKind {
    programmable_transaction: Option<ProgrammableTransaction>,
}

proto_decode!(TransactionKind {
    2 => programmable_transaction: optional_message,
});

#[derive(Debug, Default)]
struct ProgrammableTransaction {
    commands: Vec<Command>,
}

proto_decode!(ProgrammableTransaction {
    2 => commands: repeated_message,
});

#[derive(Debug, Default)]
struct Command {
    move_call: Option<MoveCall>,
}

proto_decode!(Command {
    1 => move_call: optional_message,
});

#[derive(Debug, Default)]
struct MoveCall {
    package: Option<String>,
    module: Option<String>,
    function: Option<String>,
}

proto_decode!(MoveCall {
    1 => package: optional_string,
    2 => module: optional_string,
    3 => function: optional_string,
});

#[cfg(test)]
mod tests {
    use gem_encoding::protobuf::{encode_grpc_frame, encode_message_field, encode_string_field};

    use super::*;

    const PACKAGE: &str = "0x25ebb9a7c50eb17b3fa9c5a30fb8b5ad8f97caaf4928943acbcff7153dfee5e3";

    fn request(functions: &[&str]) -> Vec<u8> {
        let commands = functions
            .iter()
            .flat_map(|function| {
                let move_call = [encode_string_field(1, PACKAGE), encode_string_field(2, "factory"), encode_string_field(3, function)].concat();
                encode_message_field(2, &encode_message_field(1, &move_call))
            })
            .collect::<Vec<_>>();
        let kind = encode_message_field(2, &commands);
        let transaction = encode_message_field(4, &kind);
        encode_grpc_frame(&encode_message_field(1, &transaction))
    }

    #[test]
    fn test_decode_contract_calls() {
        let body = request(&["new_pool_key", "pool_simple_info", "pool_id"]);
        let calls = SuiContractCallDecoder
            .decode_contract_calls(ContractRequest::Http {
                path: SIMULATE_TRANSACTION_PATH,
                method: "POST",
                body: &body,
            })
            .unwrap();

        assert_eq!(calls.len(), 3);
        assert_eq!(calls[0].address, PACKAGE);
        assert_eq!(calls[0].identifier, "factory::new_pool_key");
    }

    #[test]
    fn test_rejects_other_requests() {
        let body = request(&["calculate_swap_result"]);

        assert!(
            SuiContractCallDecoder
                .decode_contract_calls(ContractRequest::Http {
                    path: SIMULATE_TRANSACTION_PATH,
                    method: "GET",
                    body: &body,
                })
                .is_none()
        );
        assert!(
            SuiContractCallDecoder
                .decode_contract_calls(ContractRequest::Http {
                    path: "/other",
                    method: "POST",
                    body: &body,
                })
                .is_none()
        );
    }
}
