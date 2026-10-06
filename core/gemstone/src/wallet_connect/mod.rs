use gem_wallet_connect::{
    SignDigestType, WalletConnectAction, WalletConnectRequestHandler, WalletConnectResponseHandler, WalletConnectResponseType, WalletConnectTransaction, WalletConnectTransactionType, config_session_properties, decode_sign_message,
};
use primitives::{Account, Chain, ChainAddress, WalletConnectCAIP2, WalletConnectLink, WalletConnectRequest};
use std::collections::HashMap;

use crate::{GemstoneError, message::sign_type::SignMessage};

pub(crate) mod simulation;

#[uniffi::remote(Enum)]
pub enum WalletConnectLink {
    Connect { uri: String },
    Request,
    Session { topic: String },
}

#[uniffi::remote(Enum)]
pub enum WalletConnectResponseType {
    String { value: String },
    Object { json: String },
}

pub fn wallet_connect_namespace(chain: Chain) -> Option<String> {
    WalletConnectCAIP2::get_namespace(chain)
}

pub fn wallet_connect_chain(chain_id: String) -> Option<Chain> {
    WalletConnectCAIP2::parse_chain_id(chain_id)
}

#[derive(Default)]
pub struct WalletConnect {}

impl WalletConnect {
    pub fn new() -> Self {
        Self {}
    }

    pub fn config_session_properties(&self, properties: HashMap<String, String>, caip2_chains: Vec<String>, accounts: Vec<Account>) -> HashMap<String, String> {
        let chains: Vec<Chain> = caip2_chains.into_iter().filter_map(|caip2| WalletConnectCAIP2::get_chain_from_id(Some(caip2)).ok()).collect();
        config_session_properties(properties, &chains, &accounts)
    }

    pub fn parse_account(&self, account: String) -> Option<ChainAddress> {
        WalletConnectCAIP2::parse_account(account)
    }

    pub fn parse_request(&self, topic: String, method: String, params: String, chain_id: String, domain: String) -> Result<WalletConnectAction, GemstoneError> {
        let request = WalletConnectRequest {
            topic,
            method,
            params,
            chain_id: Some(chain_id),
            domain,
        };
        WalletConnectRequestHandler::parse_request(request).map_err(|e| GemstoneError::AnyError { msg: e })
    }

    pub fn encode_sign_message(&self, chain: Chain, signature: String) -> WalletConnectResponseType {
        WalletConnectResponseHandler::encode_sign_message(chain.chain_type(), signature)
    }

    pub fn encode_sign_transaction(&self, chain: Chain, transaction_id: String) -> WalletConnectResponseType {
        WalletConnectResponseHandler::encode_sign_transaction(chain.chain_type(), transaction_id)
    }

    pub fn encode_sign_all_transactions(&self, signed_transactions: Vec<String>) -> WalletConnectResponseType {
        WalletConnectResponseHandler::encode_sign_all_transactions(signed_transactions)
    }

    pub fn encode_get_accounts(&self, chain: Chain, accounts: Vec<Account>) -> WalletConnectResponseType {
        WalletConnectResponseHandler::encode_get_accounts(chain.chain_type(), &accounts)
    }

    pub fn encode_send_transaction(&self, chain: Chain, transaction_id: String) -> WalletConnectResponseType {
        WalletConnectResponseHandler::encode_send_transaction(chain.chain_type(), transaction_id)
    }

    pub fn decode_sign_message(&self, chain: Chain, sign_type: SignDigestType, data: String) -> SignMessage {
        decode_sign_message(chain, sign_type, data)
    }

    pub fn decode_send_transaction(&self, transaction_type: WalletConnectTransactionType, data: String) -> Result<WalletConnectTransaction, GemstoneError> {
        WalletConnectRequestHandler::decode_send_transaction(transaction_type, data).map_err(|e| GemstoneError::AnyError { msg: e })
    }
}

#[cfg(test)]
mod tests {
    use primitives::{Chain, ChainAddress, SimulationWarning, SimulationWarningType};

    #[test]
    fn parse_chain_id_uses_shared_caip2_parser() {
        assert_eq!(super::wallet_connect_chain("eip155:8453".to_string()), Some(Chain::Base));
        assert_eq!(super::wallet_connect_chain("solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp".to_string()), Some(Chain::Solana));
        assert_eq!(super::wallet_connect_chain("ton:-239".to_string()), Some(Chain::Ton));
        assert_eq!(super::wallet_connect_chain("tron:0x2b6653dc".to_string()), Some(Chain::Tron));
        assert_eq!(super::wallet_connect_chain("eip155:8453:extra".to_string()), None);
        assert_eq!(super::wallet_connect_chain("eip155:99999".to_string()), None);
        assert_eq!(super::wallet_connect_chain("bip122:000000000019d6689c085ae165831e93".to_string()), None);
    }

    #[test]
    fn namespace_comes_from_the_chain_table() {
        assert_eq!(super::wallet_connect_namespace(Chain::Base), Some("eip155".to_string()));
        assert_eq!(super::wallet_connect_namespace(Chain::Solana), Some("solana".to_string()));
    }

    #[test]
    fn parse_account_uses_shared_caip10_parser() {
        let wallet_connect = super::WalletConnect::new();
        let address = "0x0000000000000000000000000000000000000001".to_string();
        let account = format!("eip155:8453:{address}");

        assert_eq!(wallet_connect.parse_account(account), Some(ChainAddress::new(Chain::Base, address)));
        assert_eq!(wallet_connect.parse_account("eip155:8453".to_string()), None);
        assert_eq!(wallet_connect.parse_account("eip155:8453:".to_string()), None);
        assert_eq!(wallet_connect.parse_account("eip155:8453:0x1:extra".to_string()), None);
        assert_eq!(wallet_connect.parse_account("eip155:99999:0x1".to_string()), None);
    }

    #[test]
    fn permit2_sign_message_simulation_matches_permit_warning_behavior() {
        let data = include_str!("../../../crates/gem_evm/testdata/uniswap_permit2.json").to_string();
        let message = super::simulation::parse_eip712_message(&data).unwrap();
        let result = simulation::evm::simulate_eip712_message(Chain::Ethereum, &message);

        assert_eq!(result.warnings.len(), 1);
        assert!(matches!(
            result.warnings.first(),
            Some(SimulationWarning {
                warning: SimulationWarningType::PermitApproval(a),
                ..
            }) if a.value.is_none()
        ));
    }
}
