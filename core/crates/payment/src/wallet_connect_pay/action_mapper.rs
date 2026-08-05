use gem_evm::call_decoder::decode_call;
use gem_wallet_connect::{WalletConnectAction, WalletConnectRequestHandler, decode_sign_message};
use num_bigint::BigUint;
use primitives::payment_decoder::wallet_connect_pay::WALLET_CONNECT_PAY_HOST;
use primitives::swap::ApprovalData;
use primitives::{SignableTransaction, SignableTransactionType, TransferDataOutputType};

use crate::PaymentAction;
use crate::error::PaymentError;
use crate::wallet_connect_pay::model::WalletRpcAction;
use crate::wallet_connect_pay::{params, validator};

const APPROVE_CALL: &str = "approve";
const APPROVE_SPENDER: &str = "spender";
const APPROVE_VALUE: &str = "value";

pub fn map_wallet_rpc(account: &str, wallet_rpc: &WalletRpcAction) -> Result<PaymentAction, PaymentError> {
    let params = params::map_signer_params(&wallet_rpc.method, &wallet_rpc.params)?;
    let action = WalletConnectRequestHandler::parse_action(
        wallet_rpc.method.clone(),
        params.to_string(),
        Some(wallet_rpc.chain_id.clone()),
        WALLET_CONNECT_PAY_HOST.to_string(),
    )
    .map_err(PaymentError::InvalidRequest)?;
    if let WalletConnectAction::Unsupported { method } = action {
        return Err(PaymentError::InvalidRequest(method));
    }
    validator::validate_signer(account, &params, &action)?;
    map_action(with_encoded_solana_output(action))
}

fn map_action(action: WalletConnectAction) -> Result<PaymentAction, PaymentError> {
    match action {
        WalletConnectAction::SignMessage { chain, sign_type, data } => Ok(PaymentAction::SignMessage {
            message: decode_sign_message(chain, sign_type, data),
        }),
        WalletConnectAction::SignTransaction { chain, transaction_type, data } => Ok(PaymentAction::SignTransaction {
            chain,
            transaction: WalletConnectRequestHandler::decode_send_transaction(transaction_type, data).map_err(PaymentError::InvalidRequest)?,
        }),
        WalletConnectAction::SendTransaction { chain, transaction_type, data } => {
            let transaction = WalletConnectRequestHandler::decode_send_transaction(transaction_type, data).map_err(PaymentError::InvalidRequest)?;
            Ok(match map_approval(&transaction) {
                Some(approval) => PaymentAction::ApproveToken { chain, approval },
                None => PaymentAction::SendTransaction { chain, transaction },
            })
        }
        WalletConnectAction::SignAllTransactions { .. } => Err(PaymentError::InvalidRequest("signAllTransactions".to_string())),
        WalletConnectAction::ChainOperation { .. } => Err(PaymentError::InvalidRequest("chainOperation".to_string())),
        WalletConnectAction::GetAccounts { .. } => Err(PaymentError::InvalidRequest("getAccounts".to_string())),
        WalletConnectAction::Unsupported { method } => Err(PaymentError::InvalidRequest(method)),
    }
}

fn map_approval(transaction: &SignableTransaction) -> Option<ApprovalData> {
    let SignableTransaction::Ethereum { data, .. } = transaction else {
        return None;
    };
    let call = decode_call(data.data.as_ref()?, None).ok()?;
    if call.function != APPROVE_CALL {
        return None;
    }
    let param = |name: &str| call.params.iter().find(|param| param.name == name).map(|param| param.value.clone());
    let value = param(APPROVE_VALUE)?;
    Some(ApprovalData {
        is_unlimited: is_unlimited_approval(&value),
        token: data.to.clone(),
        spender: param(APPROVE_SPENDER)?,
        value,
    })
}

fn with_encoded_solana_output(action: WalletConnectAction) -> WalletConnectAction {
    match action {
        WalletConnectAction::SignTransaction {
            chain,
            transaction_type: SignableTransactionType::Solana { .. },
            data,
        } => WalletConnectAction::SignTransaction {
            chain,
            transaction_type: SignableTransactionType::Solana {
                output_type: TransferDataOutputType::EncodedTransaction,
            },
            data,
        },
        action => action,
    }
}

const UNLIMITED_APPROVE_BIT_WIDTHS: [u32; 2] = [160, 256];

fn is_unlimited_approval(value: &str) -> bool {
    let Ok(value) = value.parse::<BigUint>() else {
        return false;
    };
    UNLIMITED_APPROVE_BIT_WIDTHS.iter().any(|bits| value == (BigUint::from(1u8) << bits) - BigUint::from(1u8))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wallet_connect_pay::model::{FetchActionsResponse, WalletConnectPayAction};
    use crate::wallet_connect_pay::testkit::{TEST_ACCOUNT_ETHEREUM, TEST_ACCOUNT_POLYGON, TEST_ACCOUNT_SOLANA};
    use primitives::Chain;
    use primitives::{SignDigestType, SignMessage};
    use serde_json::Value;

    fn get_actions(json: &str) -> Vec<WalletRpcAction> {
        let response: FetchActionsResponse = serde_json::from_str(json).unwrap();
        response
            .actions
            .into_iter()
            .map(|action| match action {
                WalletConnectPayAction::WalletRpc(wallet_rpc) => wallet_rpc,
                WalletConnectPayAction::Build(_) => panic!("Expected walletRpc action"),
            })
            .collect()
    }

    fn sign_message_text(account: &str, action: &WalletRpcAction) -> String {
        let PaymentAction::SignMessage { message } = map_wallet_rpc(account, action).unwrap() else {
            panic!("Expected a SignMessage action");
        };
        String::from_utf8(message.data).unwrap()
    }

    #[test]
    fn test_map_wallet_rpc_action() {
        let actions = get_actions(include_str!("../../testdata/fetch_response_transfer_authorization.json"));
        assert!(matches!(
            map_wallet_rpc(TEST_ACCOUNT_ETHEREUM, &actions[0]).unwrap(),
            PaymentAction::SignMessage {
                message: SignMessage {
                    chain: Chain::Ethereum,
                    sign_type: SignDigestType::Eip712,
                    ..
                }
            }
        ));

        let actions = get_actions(include_str!("../../testdata/fetch_response_permit2.json"));
        match map_wallet_rpc(TEST_ACCOUNT_POLYGON, &actions[0]).unwrap() {
            PaymentAction::ApproveToken { chain, approval } => {
                assert_eq!(chain, Chain::Polygon);
                assert_eq!(approval.token, "0xc2132d05d31c914a87c6611c10748aeb04b58e8f");
                assert_eq!(approval.spender, "0x000000000022D473030F116dDEE9F6B43aC78BA3");
                assert!(approval.is_unlimited);
            }
            action => panic!("Expected an approval, got {action:?}"),
        }
        assert!(matches!(
            map_wallet_rpc(TEST_ACCOUNT_POLYGON, &actions[1]).unwrap(),
            PaymentAction::SignMessage {
                message: SignMessage {
                    chain: Chain::Polygon,
                    sign_type: SignDigestType::Eip712,
                    ..
                }
            }
        ));

        let actions = get_actions(include_str!("../../testdata/fetch_response_solana.json"));
        match map_wallet_rpc(TEST_ACCOUNT_SOLANA, &actions[0]).unwrap() {
            PaymentAction::SignTransaction { chain, transaction } => {
                assert_eq!(chain, Chain::Solana);
                let SignableTransaction::Solana { data, output_type } = transaction else {
                    panic!("Expected a Solana transaction");
                };
                assert_eq!(output_type, TransferDataOutputType::EncodedTransaction);
                assert!(!data.transaction.is_empty());
            }
            action => panic!("Expected Solana SignTransaction, got {action:?}"),
        }
    }

    #[test]
    fn test_map_wallet_rpc_action_normalizes_typed_data() {
        let actions = get_actions(include_str!("../../testdata/fetch_response_permit2.json"));
        let typed_data: Value = serde_json::from_str(&sign_message_text(TEST_ACCOUNT_POLYGON, &actions[1])).unwrap();
        assert_eq!(
            typed_data["types"]["EIP712Domain"],
            serde_json::json!([
                { "name": "name", "type": "string" },
                { "name": "chainId", "type": "uint256" },
                { "name": "verifyingContract", "type": "address" }
            ])
        );

        let actions = get_actions(include_str!("../../testdata/fetch_response_transfer_authorization.json"));
        assert_eq!(Value::String(sign_message_text(TEST_ACCOUNT_ETHEREUM, &actions[0])), actions[0].params[1]);
    }

    #[test]
    fn test_map_wallet_rpc_action_rejects() {
        let actions = get_actions(include_str!("../../testdata/fetch_response_permit2.json"));

        let mismatched_account = "eip155:137:0x9999999999999999999999999999999999999999";
        assert!(matches!(
            map_wallet_rpc(mismatched_account, &actions[0]),
            Err(PaymentError::InvalidRequest(message)) if message.contains("mismatch")
        ));
        assert!(matches!(
            map_wallet_rpc(mismatched_account, &actions[1]),
            Err(PaymentError::InvalidRequest(message)) if message.contains("mismatch")
        ));

        let wrong_chain = WalletRpcAction {
            chain_id: "eip155:1".to_string(),
            ..actions[1].clone()
        };
        assert!(map_wallet_rpc(TEST_ACCOUNT_ETHEREUM, &wrong_chain).is_err());

        let unsupported = WalletRpcAction {
            chain_id: "eip155:1".to_string(),
            method: "eth_sign".to_string(),
            params: serde_json::json!([]),
        };
        assert!(matches!(
            map_wallet_rpc(TEST_ACCOUNT_ETHEREUM, &unsupported),
            Err(PaymentError::InvalidRequest(method)) if method == "eth_sign"
        ));

        assert!(matches!(
            map_wallet_rpc("invalid-account", &actions[0]),
            Err(PaymentError::InvalidRequest(message)) if message.contains("Invalid option account")
        ));
    }
}
