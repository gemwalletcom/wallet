use num_bigint::BigUint;
use num_traits::Num;
use primitives::{ChainAddress, PaymentAction, WalletConnectCAIP2};
use serde::Deserialize;
use serde_json::Value;

use crate::error::PaymentError;
use crate::wallet_connect_pay::account::{get_account, is_supported};
use crate::wallet_connect_pay::model::WalletRpcAction;

const METHOD_ETHEREUM_SEND_TRANSACTION: &str = "eth_sendTransaction";

const HEX_PREFIX: &str = "0x";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EthereumTransaction {
    from: String,
    to: String,
    #[serde(default)]
    value: Option<String>,
    #[serde(default)]
    data: Option<String>,
}

pub fn map_wallet_rpc(account: &str, quoted_value: &BigUint, action: &WalletRpcAction) -> Result<PaymentAction, PaymentError> {
    let account = signer(account, &action.chain_id)?;

    let (action, value) = match action.method.as_str() {
        METHOD_ETHEREUM_SEND_TRANSACTION => map_send(&account, first_parameter(&action.params)?)?,
        method => return Err(PaymentError::InvalidRequest(format!("Payment asks for {method}"))),
    };
    if &value != quoted_value {
        return Err(PaymentError::InvalidRequest(format!("Payment asks to send {value} for a quote of {quoted_value}")));
    }
    Ok(action)
}

fn signer(account: &str, chain_id: &str) -> Result<ChainAddress, PaymentError> {
    let account = get_account(account).ok_or_else(|| PaymentError::InvalidRequest("Payment quote has no account".to_string()))?;
    if !is_supported(account.chain) {
        return Err(PaymentError::InvalidRequest(format!("Unsupported chain: {}", account.chain.as_ref())));
    }
    let chain = WalletConnectCAIP2::parse_chain_id(chain_id.to_string()).ok_or_else(|| PaymentError::InvalidRequest(format!("Unsupported chain: {chain_id}")))?;
    if chain != account.chain {
        return Err(PaymentError::InvalidRequest(format!(
            "Payment asks to sign on {} for an account on {}",
            chain.as_ref(),
            account.chain.as_ref()
        )));
    }
    Ok(account)
}

fn map_send(account: &ChainAddress, parameter: &Value) -> Result<(PaymentAction, BigUint), PaymentError> {
    let transaction: EthereumTransaction = deserialize(parameter)?;
    if !account.address.eq_ignore_ascii_case(&transaction.from) {
        return Err(PaymentError::InvalidRequest("Payment asks to sign from another account".to_string()));
    }
    let value = hex_value(transaction.value.as_deref().unwrap_or_default())?;
    Ok((
        PaymentAction::Send {
            chain: account.chain,
            recipient: transaction.to,
            value: value.clone(),
            data: transaction.data.unwrap_or_default(),
        },
        value,
    ))
}

fn first_parameter(params: &Value) -> Result<&Value, PaymentError> {
    match params {
        Value::Array(parameters) => parameters
            .first()
            .ok_or_else(|| PaymentError::InvalidRequest("Payment action has no parameters".to_string())),
        parameter => Ok(parameter),
    }
}

fn deserialize<T: for<'a> Deserialize<'a>>(parameter: &Value) -> Result<T, PaymentError> {
    serde_json::from_value(parameter.clone()).map_err(|error| PaymentError::InvalidRequest(error.to_string()))
}

fn hex_value(value: &str) -> Result<BigUint, PaymentError> {
    if value.is_empty() {
        return Ok(BigUint::ZERO);
    }
    let invalid = || PaymentError::InvalidRequest(format!("Invalid payment value: {value}"));
    let digits = value.strip_prefix(HEX_PREFIX).ok_or_else(invalid)?;
    if digits.is_empty() {
        return Ok(BigUint::ZERO);
    }
    BigUint::from_str_radix(digits, 16).map_err(|_| invalid())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wallet_connect_pay::model::{FetchActionsResponse, WalletConnectPayAction};
    use crate::wallet_connect_pay::testkit::{FETCH_RESPONSE_PERMIT2, TEST_ACCOUNT_ETHEREUM, TEST_ACCOUNT_POLYGON, TEST_ADDRESS, TEST_EVM_RECIPIENT};
    use primitives::Chain;

    fn wallet_rpc(fixture: &str) -> WalletRpcAction {
        let response: FetchActionsResponse = serde_json::from_str(fixture).unwrap();
        match response.actions.into_iter().next().unwrap() {
            WalletConnectPayAction::WalletRpc(action) => action,
            WalletConnectPayAction::Build(_) => panic!("fixture is not a wallet rpc action"),
        }
    }

    fn ethereum_transfer(transaction: serde_json::Value) -> WalletRpcAction {
        WalletRpcAction {
            chain_id: "eip155:1".to_string(),
            method: METHOD_ETHEREUM_SEND_TRANSACTION.to_string(),
            params: serde_json::json!([transaction]),
        }
    }

    #[test]
    fn test_map_a_native_send() {
        let action = ethereum_transfer(serde_json::json!({
            "from": TEST_ADDRESS,
            "to": TEST_EVM_RECIPIENT,
            "value": "0x3c4f4c72b6b800",
            "data": "0x"
        }));

        assert_eq!(
            map_wallet_rpc(TEST_ACCOUNT_ETHEREUM, &16975688363325440u64.into(), &action).unwrap(),
            PaymentAction::Send {
                chain: Chain::Ethereum,
                recipient: TEST_EVM_RECIPIENT.to_string(),
                value: 16975688363325440u64.into(),
                data: "0x".to_string(),
            }
        );
        assert!(map_wallet_rpc(TEST_ACCOUNT_ETHEREUM, &16975688363325441u64.into(), &action).is_err());
    }

    #[test]
    fn test_map_a_native_send_without_call_data() {
        let action = ethereum_transfer(serde_json::json!({
            "from": TEST_ADDRESS,
            "to": TEST_EVM_RECIPIENT,
            "value": "0x1"
        }));

        assert!(map_wallet_rpc(TEST_ACCOUNT_ETHEREUM, &1u32.into(), &action).is_ok());
    }

    #[test]
    fn test_refuses_a_call_paying_a_different_value_than_quoted() {
        let approval = wallet_rpc(FETCH_RESPONSE_PERMIT2);
        assert_eq!(
            map_wallet_rpc(TEST_ACCOUNT_POLYGON, &1u32.into(), &approval),
            Err(PaymentError::InvalidRequest("Payment asks to send 0 for a quote of 1".to_string()))
        );

        let refused = |method: &str| {
            let action = WalletRpcAction {
                chain_id: "eip155:1".to_string(),
                method: method.to_string(),
                params: serde_json::json!([TEST_ACCOUNT_ETHEREUM, "{}"]),
            };
            map_wallet_rpc(TEST_ACCOUNT_ETHEREUM, &1u32.into(), &action)
        };

        assert_eq!(
            refused("eth_signTypedData_v4"),
            Err(PaymentError::InvalidRequest("Payment asks for eth_signTypedData_v4".to_string()))
        );
        assert_eq!(
            refused("solana_signTransaction"),
            Err(PaymentError::InvalidRequest("Payment asks for solana_signTransaction".to_string()))
        );
        assert_eq!(refused("personal_sign"), Err(PaymentError::InvalidRequest("Payment asks for personal_sign".to_string())));
    }

    #[test]
    fn test_refuses_a_transfer_from_another_account() {
        let action = ethereum_transfer(serde_json::json!({
            "from": TEST_EVM_RECIPIENT,
            "to": TEST_EVM_RECIPIENT,
            "value": "0x1"
        }));

        assert_eq!(
            map_wallet_rpc(TEST_ACCOUNT_ETHEREUM, &16975688363325440u64.into(), &action),
            Err(PaymentError::InvalidRequest("Payment asks to sign from another account".to_string()))
        );
    }

    #[test]
    fn test_refuses_an_action_on_another_chain_than_its_account() {
        let action = ethereum_transfer(serde_json::json!({
            "from": TEST_ADDRESS,
            "to": TEST_EVM_RECIPIENT,
            "value": "0x1"
        }));

        assert_eq!(
            map_wallet_rpc(TEST_ACCOUNT_POLYGON, &1u32.into(), &action),
            Err(PaymentError::InvalidRequest("Payment asks to sign on ethereum for an account on polygon".to_string()))
        );
    }

    #[test]
    fn test_reads_a_hex_value() {
        assert_eq!(hex_value("0x0").unwrap(), BigUint::ZERO);
        assert_eq!(hex_value("").unwrap(), BigUint::ZERO);
        assert_eq!(hex_value("0x").unwrap(), BigUint::ZERO);
        assert_eq!(hex_value("0x3c4f4c72b6b800").unwrap(), 16975688363325440u64.into());
        assert_eq!(
            hex_value("0xffffffffffffffffffffffffffffffffff").unwrap(),
            BigUint::from_str_radix("ffffffffffffffffffffffffffffffffff", 16).unwrap()
        );

        assert!(hex_value("0xzz").is_err());
        assert!(hex_value("1e18").is_err());
    }
}
