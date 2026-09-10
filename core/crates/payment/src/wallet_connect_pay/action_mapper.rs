use num_bigint::BigUint;
use primitives::hex::decode_hex;
use primitives::{ChainAddress, WCEthereumTransaction, WalletConnectCAIP2};
use serde::Deserialize;
use serde_json::Value;

use crate::error::PaymentError;
use crate::wallet_connect_pay::model::{PaymentAction, WalletRpcAction};

const METHOD_ETHEREUM_SEND_TRANSACTION: &str = "eth_sendTransaction";

pub(super) fn map_wallet_rpc(account: &ChainAddress, quoted_value: &BigUint, action: &WalletRpcAction) -> Result<PaymentAction, PaymentError> {
    validate_chain(account, &action.chain_id)?;

    let payment = match action.method.as_str() {
        METHOD_ETHEREUM_SEND_TRANSACTION => map_send(account, get_first_parameter(&action.params)?)?,
        method => {
            return Err(PaymentError::InvalidRequest {
                reason: format!("Payment asks for {method}"),
            });
        }
    };
    if &payment.value != quoted_value {
        return Err(PaymentError::InvalidRequest {
            reason: format!("Payment asks to send {} for a quote of {quoted_value}", payment.value),
        });
    }
    Ok(payment)
}

fn validate_chain(account: &ChainAddress, chain_id: &str) -> Result<(), PaymentError> {
    let chain = WalletConnectCAIP2::parse_chain_id(chain_id.to_string()).ok_or_else(|| PaymentError::InvalidRequest {
        reason: format!("Unsupported chain: {chain_id}"),
    })?;
    if chain != account.chain {
        return Err(PaymentError::InvalidRequest {
            reason: format!("Payment asks to sign on {} for an account on {}", chain.as_ref(), account.chain.as_ref()),
        });
    }
    Ok(())
}

fn map_send(account: &ChainAddress, parameter: &Value) -> Result<PaymentAction, PaymentError> {
    let transaction: WCEthereumTransaction = deserialize(parameter)?;
    if !account.address.eq_ignore_ascii_case(&transaction.from) {
        return Err(PaymentError::InvalidRequest {
            reason: "Payment asks to sign from another account".to_string(),
        });
    }
    Ok(PaymentAction {
        value: get_value(transaction.value.as_deref().unwrap_or_default())?,
        account: account.clone(),
        recipient: transaction.to,
        data: transaction.data.unwrap_or_default(),
    })
}

fn get_first_parameter(params: &Value) -> Result<&Value, PaymentError> {
    match params {
        Value::Array(parameters) => parameters.first().ok_or_else(|| PaymentError::InvalidRequest {
            reason: "Payment action has no parameters".to_string(),
        }),
        parameter => Ok(parameter),
    }
}

fn deserialize<T: for<'a> Deserialize<'a>>(parameter: &Value) -> Result<T, PaymentError> {
    serde_json::from_value(parameter.clone()).map_err(|error| PaymentError::InvalidRequest { reason: error.to_string() })
}

fn get_value(value: &str) -> Result<BigUint, PaymentError> {
    decode_hex(value).map(|bytes| BigUint::from_bytes_be(&bytes)).map_err(|error| PaymentError::InvalidRequest {
        reason: format!("Invalid payment value: {error}"),
    })
}
