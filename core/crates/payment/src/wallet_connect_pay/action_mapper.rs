use gem_evm::transaction::{EvmTransactionKind, decode_transaction_kind};
use num_bigint::BigUint;
use primitives::swap::ApprovalData;
use primitives::{ValueAccess, WCEthereumTransaction, WalletConnectCAIP2, WalletConnectionMethods};
use serde_json::Value;
use serde_serializers::biguint_from_hex_str;

use crate::error::PaymentError;
use crate::wallet_connect_pay::model::{PaymentAction, PaymentSend, PaymentSign, Quote, WalletRpcAction};
use crate::wallet_connect_pay::typed_data_mapper::map_typed_data;

pub(super) fn map_actions(quote: &Quote, actions: &[WalletRpcAction]) -> Result<PaymentAction, PaymentError> {
    let methods: Vec<Option<WalletConnectionMethods>> = actions.iter().map(|action| action.method()).collect();
    match (actions, methods.as_slice()) {
        ([send], [Some(WalletConnectionMethods::EthSendTransaction)]) => Ok(PaymentAction::Send(map_send(quote, send)?)),
        ([sign], [Some(WalletConnectionMethods::EthSignTypedDataV4)]) => Ok(PaymentAction::Sign(map_sign(quote, sign)?)),
        ([approve, sign], [Some(WalletConnectionMethods::EthSendTransaction), Some(WalletConnectionMethods::EthSignTypedDataV4)]) => {
            let sign = map_sign(quote, sign)?;
            let approval = map_approval(quote, approve)?;
            Ok(PaymentAction::ApproveAndSign { approval, sign })
        }
        ([action], _) => Err(PaymentError::invalid_request(format!("Payment asks for {}", action.method))),
        (actions, _) => Err(PaymentError::invalid_request(format!("Payment asks for {} actions", actions.len()))),
    }
}

fn map_send(quote: &Quote, action: &WalletRpcAction) -> Result<PaymentSend, PaymentError> {
    let transaction = get_transaction(quote, action)?;
    let value = get_value(&transaction)?;
    if value != quote.value {
        return Err(PaymentError::invalid_request(format!("Payment asks to send {value} for a quote of {}", quote.value)));
    }
    Ok(PaymentSend {
        recipient: transaction.to,
        data: transaction.data.unwrap_or_default(),
    })
}

fn map_sign(quote: &Quote, action: &WalletRpcAction) -> Result<PaymentSign, PaymentError> {
    let token = quote.token();
    validate_chain(quote, action)?;
    let signer = action.params.at(0).and_then(Value::string).map_err(PaymentError::invalid_request)?;
    if !signer.eq_ignore_ascii_case(&quote.account.address) {
        return Err(PaymentError::invalid_request("Payment asks to sign from another account"));
    }
    let transfer = map_typed_data(quote.account.chain, action.params.at(1).map_err(PaymentError::invalid_request)?)?;
    if !transfer.token.eq_ignore_ascii_case(token) {
        return Err(PaymentError::invalid_request(format!("Payment asks to sign for token {} on a quote of {token}", transfer.token)));
    }
    if transfer.amount != quote.value {
        return Err(PaymentError::invalid_request(format!("Payment asks to sign {} for a quote of {}", transfer.amount, quote.value)));
    }
    if transfer.from.as_deref().is_some_and(|from| !from.eq_ignore_ascii_case(&quote.account.address)) {
        return Err(PaymentError::invalid_request("Payment asks to transfer from another account"));
    }
    Ok(PaymentSign {
        recipient: transfer.recipient,
        typed_data: transfer.typed_data,
    })
}

fn map_approval(quote: &Quote, action: &WalletRpcAction) -> Result<ApprovalData, PaymentError> {
    let token = quote.token();
    let transaction = get_transaction(quote, action)?;
    if get_value(&transaction)? != BigUint::ZERO {
        return Err(PaymentError::invalid_request("Payment approval sends value"));
    }
    if !transaction.to.eq_ignore_ascii_case(token) {
        return Err(PaymentError::invalid_request(format!("Payment asks to approve {} on a quote of {token}", transaction.to)));
    }
    match decode_transaction_kind(token, transaction.data.as_deref()).map_err(PaymentError::invalid_request)? {
        EvmTransactionKind::TokenApproval(approval) => Ok(approval),
        EvmTransactionKind::Transfer | EvmTransactionKind::ContractCall => Err(PaymentError::invalid_request("Payment approval is not a token approval")),
    }
}

fn get_transaction(quote: &Quote, action: &WalletRpcAction) -> Result<WCEthereumTransaction, PaymentError> {
    validate_chain(quote, action)?;
    let parameter = action.params.at(0).map_err(PaymentError::invalid_request)?;
    let transaction: WCEthereumTransaction = serde_json::from_value(parameter.clone()).map_err(|error| PaymentError::invalid_request(error.to_string()))?;
    if !quote.account.address.eq_ignore_ascii_case(&transaction.from) {
        return Err(PaymentError::invalid_request("Payment asks to sign from another account"));
    }
    if transaction.chain_id.is_some_and(|chain_id| Some(chain_id) != quote.account.chain.network_id_value()) {
        return Err(PaymentError::invalid_request(format!("Payment transaction is for chain {:?}", transaction.chain_id)));
    }
    Ok(transaction)
}

fn validate_chain(quote: &Quote, action: &WalletRpcAction) -> Result<(), PaymentError> {
    let chain = WalletConnectCAIP2::get_chain_from_id(Some(action.chain_id.clone())).map_err(PaymentError::invalid_request)?;
    if chain != quote.account.chain {
        return Err(PaymentError::invalid_request(format!(
            "Payment asks to sign on {} for an account on {}",
            chain.as_ref(),
            quote.account.chain.as_ref()
        )));
    }
    Ok(())
}

fn get_value(transaction: &WCEthereumTransaction) -> Result<BigUint, PaymentError> {
    biguint_from_hex_str(transaction.value.as_deref().unwrap_or_default())
        .map_err(|error| PaymentError::invalid_request(format!("Invalid payment value: {error}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wallet_connect_pay::testkit::{
        FETCH_RECEIVE_WITH_AUTHORIZATION, FETCH_SEND, OPTIONS, OPTIONS_WITHOUT_ALLOWANCE, PYUSD_TOKEN_ID, TEST_ACCOUNT, TEST_ACCOUNT_WITHOUT_ALLOWANCE,
        TEST_AUTHORIZATION_RECIPIENT, TEST_PERMIT_SPENDER, TEST_ROUTER, fetch_actions, quote, quote_actions,
    };
    use primitives::asset_constants::{ETHEREUM_USDT_ASSET_ID, ETHEREUM_USDT_TOKEN_ID};
    use primitives::contract_constants::UNISWAP_PERMIT2_CONTRACT;
    use primitives::{AssetId, Chain, WalletConnectionMethods, serde_name};

    fn typed_data(action: &WalletRpcAction) -> String {
        serde_json::from_str::<Value>(action.params[1].as_str().unwrap()).unwrap().to_string()
    }

    fn with_chain_id(action: &WalletRpcAction, chain_id: &str) -> WalletRpcAction {
        WalletRpcAction {
            chain_id: chain_id.to_string(),
            ..action.clone()
        }
    }

    fn with_transaction(action: &WalletRpcAction, field: &str, value: Value) -> WalletRpcAction {
        let mut action = action.clone();
        action.params[0][field] = value;
        action
    }

    #[test]
    fn test_map_actions() {
        let coin = quote(OPTIONS, TEST_ACCOUNT, &AssetId::from_chain(Chain::Optimism));
        let send = fetch_actions(FETCH_SEND);
        assert_eq!(
            map_actions(&coin, &send),
            Ok(PaymentAction::Send(PaymentSend {
                recipient: TEST_ROUTER.to_string(),
                data: send[0].params[0]["data"].as_str().unwrap().to_string(),
            }))
        );

        let usdt = quote(OPTIONS, TEST_ACCOUNT, &ETHEREUM_USDT_ASSET_ID);
        let permit = quote_actions(&usdt);
        assert_eq!(
            map_actions(&usdt, &permit),
            Ok(PaymentAction::Sign(PaymentSign {
                recipient: TEST_PERMIT_SPENDER.to_string(),
                typed_data: typed_data(&permit[0]),
            }))
        );

        let unapproved = quote(OPTIONS_WITHOUT_ALLOWANCE, TEST_ACCOUNT_WITHOUT_ALLOWANCE, &ETHEREUM_USDT_ASSET_ID);
        let approve_and_permit = quote_actions(&unapproved);
        assert_eq!(
            map_actions(&unapproved, &approve_and_permit),
            Ok(PaymentAction::ApproveAndSign {
                approval: ApprovalData {
                    token: ETHEREUM_USDT_TOKEN_ID.to_string(),
                    spender: UNISWAP_PERMIT2_CONTRACT.to_string(),
                    value: BigUint::from_bytes_be(&[0xff; 32]),
                    is_unlimited: true,
                },
                sign: PaymentSign {
                    recipient: TEST_PERMIT_SPENDER.to_string(),
                    typed_data: typed_data(&approve_and_permit[1]),
                },
            })
        );

        let pyusd = quote(OPTIONS_WITHOUT_ALLOWANCE, TEST_ACCOUNT_WITHOUT_ALLOWANCE, &AssetId::from_token(Chain::Ethereum, PYUSD_TOKEN_ID));
        let authorization = fetch_actions(FETCH_RECEIVE_WITH_AUTHORIZATION);
        assert_eq!(
            map_actions(&pyusd, &authorization),
            Ok(PaymentAction::Sign(PaymentSign {
                recipient: TEST_AUTHORIZATION_RECIPIENT.to_string(),
                typed_data: typed_data(&authorization[0]),
            }))
        );

        assert_eq!(
            map_actions(&unapproved, &[approve_and_permit[1].clone(), approve_and_permit[0].clone()]),
            Err(PaymentError::invalid_request("Payment asks for 2 actions"))
        );
        assert_eq!(
            map_actions(&unapproved, &[approve_and_permit[0].clone(), approve_and_permit[1].clone(), approve_and_permit[1].clone()]),
            Err(PaymentError::invalid_request("Payment asks for 3 actions"))
        );
        assert_eq!(
            map_actions(
                &usdt,
                &[WalletRpcAction {
                    method: serde_name(&WalletConnectionMethods::PersonalSign).unwrap(),
                    ..permit[0].clone()
                }]
            ),
            Err(PaymentError::invalid_request("Payment asks for personal_sign"))
        );
    }

    #[test]
    fn test_map_send() {
        let coin = quote(OPTIONS, TEST_ACCOUNT, &AssetId::from_chain(Chain::Optimism));
        let send = &fetch_actions(FETCH_SEND)[0];

        assert_eq!(
            map_send(&Quote { value: BigUint::from(1u32), ..coin.clone() }, send),
            Err(PaymentError::invalid_request("Payment asks to send 41877035785636 for a quote of 1"))
        );
        assert_eq!(
            map_send(&coin, &with_chain_id(send, "eip155:1")),
            Err(PaymentError::invalid_request("Payment asks to sign on ethereum for an account on optimism"))
        );
        assert_eq!(
            map_send(&coin, &with_chain_id(send, "eip155")),
            Err(PaymentError::invalid_request("Invalid chain ID format"))
        );
        assert_eq!(
            map_send(&coin, &with_transaction(send, "chainId", Value::from(1))),
            Err(PaymentError::invalid_request("Payment transaction is for chain Some(1)"))
        );
        assert_eq!(
            map_send(&coin, &with_transaction(send, "from", Value::from(TEST_ACCOUNT_WITHOUT_ALLOWANCE))),
            Err(PaymentError::invalid_request("Payment asks to sign from another account"))
        );
        assert_eq!(
            map_send(&coin, &with_transaction(send, "value", Value::from("0xzz"))),
            Err(PaymentError::invalid_request("Invalid payment value: Invalid hex string: 0xzz"))
        );
    }

    #[test]
    fn test_map_sign() {
        let usdt = quote(OPTIONS, TEST_ACCOUNT, &ETHEREUM_USDT_ASSET_ID);
        let permit = &quote_actions(&usdt)[0];
        let mut other_signer = permit.clone();
        other_signer.params[0] = Value::from(TEST_ACCOUNT_WITHOUT_ALLOWANCE);

        assert_eq!(
            map_sign(&Quote { value: BigUint::from(1u32), ..usdt.clone() }, permit),
            Err(PaymentError::invalid_request("Payment asks to sign 100000 for a quote of 1"))
        );
        assert_eq!(
            map_sign(&quote(OPTIONS, TEST_ACCOUNT, &AssetId::from_chain(Chain::Ethereum)), permit),
            Err(PaymentError::invalid_request(format!("Payment asks to sign for token {ETHEREUM_USDT_TOKEN_ID} on a quote of ")))
        );
        assert_eq!(
            map_sign(&usdt, &with_chain_id(permit, "eip155:56")),
            Err(PaymentError::invalid_request("Payment asks to sign on smartchain for an account on ethereum"))
        );
        assert_eq!(map_sign(&usdt, &other_signer), Err(PaymentError::invalid_request("Payment asks to sign from another account")));
    }

    #[test]
    fn test_map_approval() {
        let unapproved = quote(OPTIONS_WITHOUT_ALLOWANCE, TEST_ACCOUNT_WITHOUT_ALLOWANCE, &ETHEREUM_USDT_ASSET_ID);
        let approve = &quote_actions(&unapproved)[0];

        assert_eq!(
            map_approval(&unapproved, &with_transaction(approve, "value", Value::from("0x1"))),
            Err(PaymentError::invalid_request("Payment approval sends value"))
        );
        assert_eq!(
            map_approval(&unapproved, &with_transaction(approve, "to", Value::from(TEST_ROUTER))),
            Err(PaymentError::invalid_request(format!("Payment asks to approve {TEST_ROUTER} on a quote of {ETHEREUM_USDT_TOKEN_ID}")))
        );
        assert_eq!(
            map_approval(&unapproved, &with_transaction(approve, "data", Value::from("0x"))),
            Err(PaymentError::invalid_request("Payment approval is not a token approval"))
        );
    }
}
