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
    let token = quote.asset_id.token_id.as_deref().unwrap_or_default();
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
    let token = quote.asset_id.token_id.as_deref().unwrap_or_default();
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
    use primitives::asset_constants::POLYGON_USDT_TOKEN_ID;
    use primitives::contract_constants::UNISWAP_PERMIT2_CONTRACT;
    use primitives::testkit::signer_mock::TEST_EVM_RECIPIENT;
    use primitives::{AssetId, Chain, WalletConnectionMethods, serde_name};

    fn polygon_quote(value: u64) -> Quote {
        Quote::mock(AssetId::from_chain(Chain::Polygon), value)
    }

    fn usdt_quote(value: u64) -> Quote {
        Quote::mock(AssetId::from_token(Chain::Polygon, POLYGON_USDT_TOKEN_ID), value)
    }

    #[test]
    fn test_coin_payment_is_a_send() {
        let quote = polygon_quote(1_000);
        let send = WalletRpcAction::mock_send(&quote, TEST_EVM_RECIPIENT, "0x3e8", "0xabcd");

        assert_eq!(
            map_actions(&quote, std::slice::from_ref(&send)).unwrap(),
            PaymentAction::Send(PaymentSend {
                recipient: TEST_EVM_RECIPIENT.to_string(),
                data: "0xabcd".to_string(),
            })
        );
        assert!(map_actions(&polygon_quote(1_001), std::slice::from_ref(&send)).is_err(), "the value must match the quote");
        assert!(map_actions(&quote, &[WalletRpcAction { chain_id: "eip155".to_string(), ..send.clone() }]).is_err());
        let ethereum = Quote::mock(AssetId::from_chain(Chain::Ethereum), 1_000);
        assert!(
            map_actions(&quote, &[WalletRpcAction::mock_send(&ethereum, TEST_EVM_RECIPIENT, "0x3e8", "0xabcd")]).is_err(),
            "another chain"
        );
        let mut other_chain = send.clone();
        other_chain.params[0]["chainId"] = serde_json::json!(1);
        assert!(map_actions(&quote, &[other_chain]).is_err(), "the transaction's own chain id must match");
        let mut other_account = send;
        other_account.params[0]["from"] = serde_json::json!(TEST_EVM_RECIPIENT);
        assert!(map_actions(&quote, &[other_account]).is_err(), "the transaction must be signed by the quote's account");
    }

    #[test]
    fn test_token_payment_with_allowance_is_a_signature() {
        let quote = usdt_quote(1_000_000);
        let permit = WalletRpcAction::mock_permit(&quote, "1000000");

        let PaymentAction::Sign(sign) = map_actions(&quote, std::slice::from_ref(&permit)).unwrap() else {
            panic!("expected a signature");
        };
        assert_eq!(sign.recipient, TEST_EVM_RECIPIENT);
        assert!(sign.typed_data.contains("EIP712Domain"));

        assert!(map_actions(&usdt_quote(1_000_001), std::slice::from_ref(&permit)).is_err(), "the amount must match the quote");
        assert!(map_actions(&polygon_quote(1_000_000), std::slice::from_ref(&permit)).is_err(), "a coin quote is not paid by a token permit");
        assert!(
            map_actions(
                &quote,
                &[WalletRpcAction {
                    chain_id: "eip155:1".to_string(),
                    ..permit
                }]
            )
            .is_err(),
            "another chain"
        );
    }

    #[test]
    fn test_first_token_payment_approves_permit2_then_signs() {
        let quote = usdt_quote(1_000_000);
        let approve = WalletRpcAction::mock_approve(&quote);
        let permit = WalletRpcAction::mock_permit(&quote, "1000000");

        let PaymentAction::ApproveAndSign { approval, sign } = map_actions(&quote, &[approve.clone(), permit.clone()]).unwrap() else {
            panic!("expected an approval and a signature");
        };
        assert_eq!(approval.token, POLYGON_USDT_TOKEN_ID, "the record's asset id must be the wallet's checksummed token id");
        assert_eq!(approval.spender, UNISWAP_PERMIT2_CONTRACT);
        assert!(approval.is_unlimited);
        assert_eq!(sign.recipient, TEST_EVM_RECIPIENT);

        assert!(map_actions(&quote, &[permit.clone(), approve.clone()]).is_err(), "the approval comes first");
        assert!(
            map_actions(&quote, &[WalletRpcAction::mock_send(&quote, TEST_EVM_RECIPIENT, "0x0", "0xabcd"), permit.clone()]).is_err(),
            "only a token approval precedes the permit"
        );
        assert_eq!(
            map_actions(&quote, &[approve.clone(), permit.clone(), permit.clone()]),
            Err(PaymentError::invalid_request("Payment asks for 3 actions"))
        );
        assert_eq!(
            map_actions(
                &quote,
                &[WalletRpcAction {
                    method: serde_name(&WalletConnectionMethods::PersonalSign).unwrap(),
                    ..permit
                }]
            ),
            Err(PaymentError::invalid_request("Payment asks for personal_sign"))
        );
    }
}
