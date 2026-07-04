use gem_wallet_connect::{
    SignDigestType as WcSignDigestType, SignMessageValidation, WCEthereumTransactionData as WcEthereumTransactionData, WalletConnectRequestHandler,
    WalletConnectTransaction as WcWalletConnectTransaction, WalletConnectTransactionType as WcWalletConnectTransactionType, decode_sign_message, validate_send_transaction,
    validate_sign_message,
};
use primitives::{Chain, SimulationWarning, hex};

use crate::message::sign_type::{SignDigestType, SignMessage};

pub fn decode_message(chain: Chain, sign_type: SignDigestType, data: String) -> SignMessage {
    let sign_type: WcSignDigestType = sign_type.into();
    let result = decode_sign_message(chain, sign_type, data);

    SignMessage {
        chain: result.chain,
        sign_type: result.sign_type.into(),
        data: result.data,
    }
}

pub(super) fn parse_eip712_message(data: &str) -> Option<gem_evm::eip712::EIP712Message> {
    serde_json::from_str(data).ok().and_then(|value| gem_evm::eip712::parse_eip712_json(&value).ok())
}

pub(super) fn sign_message_validation_warnings(chain: Chain, sign_type: &WcSignDigestType, data: &str, session_domain: &str) -> Vec<SimulationWarning> {
    let input = SignMessageValidation {
        chain,
        sign_type,
        data,
        session_domain,
    };

    validate_sign_message(&input).err().into_iter().map(SimulationWarning::validation_error).collect()
}

pub(super) fn send_transaction_validation_warnings(transaction_type: &WcWalletConnectTransactionType, data: &str) -> Vec<SimulationWarning> {
    validate_send_transaction(transaction_type, data)
        .err()
        .into_iter()
        .map(SimulationWarning::validation_error)
        .collect()
}

pub(super) fn decode_ethereum_transaction(data: &str) -> Option<WcEthereumTransactionData> {
    match WalletConnectRequestHandler::decode_send_transaction(WcWalletConnectTransactionType::Ethereum, data.to_string()).ok()? {
        WcWalletConnectTransaction::Ethereum { data } => Some(data),
        _ => None,
    }
}

pub(super) fn decode_ethereum_calldata(transaction: &WcEthereumTransactionData) -> Vec<u8> {
    transaction.data.as_deref().and_then(|calldata| hex::decode_hex(calldata).ok()).unwrap_or_default()
}

pub(super) fn decode_solana_transaction(transaction_type: &WcWalletConnectTransactionType, data: &str) -> Option<String> {
    match WalletConnectRequestHandler::decode_send_transaction(transaction_type.clone(), data.to_string()).ok()? {
        WcWalletConnectTransaction::Solana { data, .. } => Some(data.transaction),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_ethereum_transaction_with_calldata_decodes_bytes() {
        let data = serde_json::json!({
            "from": "0xF977814e90dA44bFA03b6295A0616a897441aceC",
            "to": "0x1111111111111111111111111111111111111111",
            "data": "0xa9059cbb",
        })
        .to_string();

        let transaction = decode_ethereum_transaction(&data).unwrap();

        assert_eq!(decode_ethereum_calldata(&transaction), vec![0xa9, 0x05, 0x9c, 0xbb]);
    }
}
