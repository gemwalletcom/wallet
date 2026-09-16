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
    if !quote.asset_id.is_native() {
        return Err(PaymentError::invalid_request("Payment asks to send a coin for a token quote"));
    }
    let transaction = get_transaction(quote, action)?;
    let value = get_value(&transaction)?;
    if value != quote.value {
        return Err(PaymentError::invalid_request(format!("Payment asks to send {value} for a quote of {}", quote.value)));
    }
    Ok(PaymentSend {
        recipient: transaction.to,
        value,
        data: transaction.data.unwrap_or_default(),
    })
}

fn map_sign(quote: &Quote, action: &WalletRpcAction) -> Result<PaymentSign, PaymentError> {
    let token = get_quote_token(quote)?;
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
    let token = get_quote_token(quote)?;
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

fn get_quote_token(quote: &Quote) -> Result<&str, PaymentError> {
    quote.asset_id.token_id.as_deref().ok_or_else(|| PaymentError::invalid_request("Payment asks to sign for a coin quote"))
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
    biguint_from_hex_str(transaction.value.as_deref().unwrap_or_default()).map_err(|error| PaymentError::invalid_request(format!("Invalid payment value: {error}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{AssetId, Chain, ChainAddress, WalletConnectionMethods, serde_name};

    const ADDRESS: &str = "0x1085c5f70F7F7591D97da281A64688385455c2bD";
    const ROUTER: &str = "0x0000000000a84d1a9b0063a910315c7ffa9cd248";
    const PERMIT2: &str = "0x000000000022d473030f116ddee9f6b43ac78ba3";
    const USDT_POLYGON: &str = "0xc2132D05D31c914a87C6611C10748AEb04B58e8F";
    const APPROVE_PERMIT2_MAX: &str = "0x095ea7b3000000000000000000000000000000000022d473030f116ddee9f6b43ac78ba3ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";

    fn quote(asset_id: AssetId, value: u64) -> Quote {
        Quote {
            id: "opt_1".to_string(),
            account: ChainAddress::new(Chain::Polygon, ADDRESS.to_string()),
            asset_id,
            value: BigUint::from(value),
            collect_data_url: None,
            actions: Vec::new(),
        }
    }

    fn polygon_quote(value: u64) -> Quote {
        quote(AssetId::from_chain(Chain::Polygon), value)
    }

    fn usdt_quote(value: u64) -> Quote {
        quote(AssetId::from_token(Chain::Polygon, USDT_POLYGON), value)
    }

    fn send(from: &str, value: &str, data: &str) -> WalletRpcAction {
        WalletRpcAction::mock(
            WalletConnectionMethods::EthSendTransaction,
            "eip155:137",
            serde_json::json!([{"from": from, "to": ROUTER, "value": value, "data": data}]),
        )
    }

    fn approve() -> WalletRpcAction {
        WalletRpcAction::mock(
            WalletConnectionMethods::EthSendTransaction,
            "eip155:137",
            serde_json::json!([{"from": ADDRESS, "to": USDT_POLYGON.to_lowercase(), "value": "0x0", "data": APPROVE_PERMIT2_MAX}]),
        )
    }

    fn permit(amount: &str) -> WalletRpcAction {
        let typed_data = serde_json::json!({
            "domain": {"name": "Permit2", "chainId": "0x89", "verifyingContract": PERMIT2},
            "types": {
                "PermitTransferFrom": [{"name": "permitted", "type": "TokenPermissions"}, {"name": "spender", "type": "address"}, {"name": "nonce", "type": "uint256"}, {"name": "deadline", "type": "uint256"}],
                "TokenPermissions": [{"name": "token", "type": "address"}, {"name": "amount", "type": "uint256"}]
            },
            "primaryType": "PermitTransferFrom",
            "message": {"permitted": {"token": USDT_POLYGON, "amount": amount}, "spender": ROUTER, "nonce": "0x08", "deadline": "1785175272"}
        });
        WalletRpcAction::mock(WalletConnectionMethods::EthSignTypedDataV4, "eip155:137", serde_json::json!([ADDRESS, typed_data.to_string()]))
    }

    #[test]
    fn test_coin_payment_is_a_send() {
        let action = map_actions(&polygon_quote(1_000), &[send(ADDRESS, "0x3e8", "0xabcd")]).unwrap();

        assert_eq!(
            action,
            PaymentAction::Send(PaymentSend {
                recipient: ROUTER.to_string(),
                value: BigUint::from(1_000u32),
                data: "0xabcd".to_string(),
            })
        );
        assert!(map_actions(&polygon_quote(1_001), &[send(ADDRESS, "0x3e8", "0xabcd")]).is_err());
        let other_chain = WalletRpcAction {
            params: serde_json::json!([{"from": ADDRESS, "to": ROUTER, "value": "0x3e8", "data": "0xabcd", "chainId": 1}]),
            ..send(ADDRESS, "0x3e8", "0xabcd")
        };
        assert!(map_actions(&polygon_quote(1_000), &[other_chain]).is_err());
        assert!(map_actions(&polygon_quote(1_000), &[WalletRpcAction { chain_id: "eip155".to_string(), ..send(ADDRESS, "0x3e8", "0xabcd") }]).is_err());
        assert!(map_actions(&polygon_quote(1_000), &[send(ROUTER, "0x3e8", "0xabcd")]).is_err());
        assert!(map_actions(&usdt_quote(1_000), &[send(ADDRESS, "0x3e8", "0xabcd")]).is_err());
    }

    #[test]
    fn test_token_payment_with_allowance_is_a_signature() {
        let action = map_actions(&usdt_quote(1_000_000), &[permit("1000000")]).unwrap();

        let PaymentAction::Sign(sign) = action else {
            panic!("expected a signature, got {action:?}");
        };
        assert!(sign.recipient.eq_ignore_ascii_case(ROUTER));
        assert!(sign.typed_data.contains("EIP712Domain"));

        assert!(map_actions(&usdt_quote(1_000_001), &[permit("1000000")]).is_err());
        assert!(map_actions(&polygon_quote(1_000_000), &[permit("1000000")]).is_err());
        let other_chain = WalletRpcAction {
            chain_id: "eip155:1".to_string(),
            ..permit("1000000")
        };
        assert!(map_actions(&usdt_quote(1_000_000), &[other_chain]).is_err());
    }

    #[test]
    fn test_first_token_payment_approves_permit2_then_signs() {
        let action = map_actions(&usdt_quote(1_000_000), &[approve(), permit("1000000")]).unwrap();

        let PaymentAction::ApproveAndSign { approval, sign } = action else {
            panic!("expected an approval and a signature, got {action:?}");
        };
        assert_eq!(approval.token, USDT_POLYGON, "the record's asset id must be the wallet's checksummed token id");
        assert_eq!(approval.spender, "0x000000000022D473030F116dDEE9F6B43aC78BA3");
        assert!(approval.is_unlimited);
        assert!(sign.recipient.eq_ignore_ascii_case(ROUTER));

        assert!(map_actions(&usdt_quote(1_000_000), &[permit("1000000"), approve()]).is_err());
        assert!(map_actions(&usdt_quote(1_000_000), &[send(ADDRESS, "0x0", "0xabcd"), permit("1000000")]).is_err());
        assert_eq!(
            map_actions(&usdt_quote(1_000_000), &[approve(), permit("1000000"), permit("1000000")]),
            Err(PaymentError::invalid_request("Payment asks for 3 actions"))
        );
        assert_eq!(
            map_actions(
                &usdt_quote(1_000_000),
                &[WalletRpcAction {
                    method: serde_name(&WalletConnectionMethods::PersonalSign).unwrap_or_default(),
                    ..permit("1000000")
                }]
            ),
            Err(PaymentError::invalid_request("Payment asks for personal_sign"))
        );
    }
}
