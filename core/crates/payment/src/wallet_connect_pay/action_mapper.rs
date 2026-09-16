use gem_evm::transaction::{EvmTransactionKind, decode_transaction_kind};
use num_bigint::BigUint;
use primitives::hex::decode_hex;
use primitives::swap::ApprovalData;
use primitives::{WCEthereumTransaction, WalletConnectCAIP2};
use serde_json::Value;

use crate::error::PaymentError;
use crate::wallet_connect_pay::model::{PaymentAction, PaymentSend, PaymentSign, Quote, WalletRpcAction};
use crate::wallet_connect_pay::typed_data_mapper::map_typed_data;

const METHOD_ETHEREUM_SEND_TRANSACTION: &str = "eth_sendTransaction";
const METHOD_ETHEREUM_SIGN_TYPED_DATA: &str = "eth_signTypedData_v4";

pub(super) fn map_actions(quote: &Quote, actions: &[WalletRpcAction]) -> Result<PaymentAction, PaymentError> {
    match actions {
        [send] if send.method == METHOD_ETHEREUM_SEND_TRANSACTION => Ok(PaymentAction::Send(map_send(quote, send)?)),
        [sign] if sign.method == METHOD_ETHEREUM_SIGN_TYPED_DATA => Ok(PaymentAction::Sign(map_sign(quote, sign)?)),
        [approve, sign] if approve.method == METHOD_ETHEREUM_SEND_TRANSACTION && sign.method == METHOD_ETHEREUM_SIGN_TYPED_DATA => {
            let sign = map_sign(quote, sign)?;
            let approval = map_approval(quote, approve)?;
            Ok(PaymentAction::ApproveAndSign { approval, sign })
        }
        [action] => Err(PaymentError::invalid_request(format!("Payment asks for {}", action.method))),
        actions => Err(PaymentError::invalid_request(format!("Payment asks for {} actions", actions.len()))),
    }
}

fn map_send(quote: &Quote, action: &WalletRpcAction) -> Result<PaymentSend, PaymentError> {
    if !quote.asset_id.is_native() {
        return Err(PaymentError::invalid_request("Payment asks to send a coin for a token quote"));
    }
    let transaction = get_transaction(quote, action)?;
    let value = get_value(transaction.value.as_deref().unwrap_or_default())?;
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
    let [signer, typed_data] = get_parameters(&action.params, 2)? else {
        return Err(PaymentError::invalid_request("Payment signature has no parameters"));
    };
    if !signer.as_str().is_some_and(|signer| signer.eq_ignore_ascii_case(&quote.account.address)) {
        return Err(PaymentError::invalid_request("Payment asks to sign from another account"));
    }
    let transfer = map_typed_data(quote.account.chain, typed_data)?;
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
    if get_value(transaction.value.as_deref().unwrap_or_default())? != BigUint::ZERO {
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
    let [parameter] = get_parameters(&action.params, 1)? else {
        return Err(PaymentError::invalid_request("Payment action has no parameters"));
    };
    let transaction: WCEthereumTransaction = serde_json::from_value(parameter.clone()).map_err(|error| PaymentError::invalid_request(error.to_string()))?;
    if !quote.account.address.eq_ignore_ascii_case(&transaction.from) {
        return Err(PaymentError::invalid_request("Payment asks to sign from another account"));
    }
    Ok(transaction)
}

fn validate_chain(quote: &Quote, action: &WalletRpcAction) -> Result<(), PaymentError> {
    let chain = WalletConnectCAIP2::parse_chain_id(action.chain_id.clone()).ok_or_else(|| PaymentError::invalid_request(format!("Unsupported chain: {}", action.chain_id)))?;
    if chain != quote.account.chain {
        return Err(PaymentError::invalid_request(format!(
            "Payment asks to sign on {} for an account on {}",
            chain.as_ref(),
            quote.account.chain.as_ref()
        )));
    }
    Ok(())
}

fn get_parameters(params: &Value, count: usize) -> Result<&[Value], PaymentError> {
    match params {
        Value::Array(parameters) if parameters.len() >= count => Ok(&parameters[..count]),
        Value::Array(_) => Err(PaymentError::invalid_request("Payment action has no parameters")),
        parameter if count == 1 => Ok(std::slice::from_ref(parameter)),
        _ => Err(PaymentError::invalid_request("Payment action has no parameters")),
    }
}

fn get_value(value: &str) -> Result<BigUint, PaymentError> {
    decode_hex(value)
        .map(|bytes| BigUint::from_bytes_be(&bytes))
        .map_err(|error| PaymentError::invalid_request(format!("Invalid payment value: {error}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{AssetId, Chain, ChainAddress};

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
        WalletRpcAction {
            chain_id: "eip155:137".to_string(),
            method: METHOD_ETHEREUM_SEND_TRANSACTION.to_string(),
            params: serde_json::json!([{"from": from, "to": ROUTER, "value": value, "data": data}]),
        }
    }

    fn approve() -> WalletRpcAction {
        WalletRpcAction {
            chain_id: "eip155:137".to_string(),
            method: METHOD_ETHEREUM_SEND_TRANSACTION.to_string(),
            params: serde_json::json!([{"from": ADDRESS, "to": USDT_POLYGON.to_lowercase(), "value": "0x0", "data": APPROVE_PERMIT2_MAX}]),
        }
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
        WalletRpcAction {
            chain_id: "eip155:137".to_string(),
            method: METHOD_ETHEREUM_SIGN_TYPED_DATA.to_string(),
            params: serde_json::json!([ADDRESS, typed_data.to_string()]),
        }
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
                    method: "personal_sign".to_string(),
                    ..permit("1000000")
                }]
            ),
            Err(PaymentError::invalid_request("Payment asks for personal_sign"))
        );
    }
}
