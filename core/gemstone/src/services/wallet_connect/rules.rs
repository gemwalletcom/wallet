use std::collections::HashSet;
use std::str::FromStr;

use crate::services::collections::{stale, unique};

use chrono::{DateTime, Utc};
use primitives::ChainType;
use primitives::WalletConnectionVerificationStatus;
use primitives::{
    Account, ApplicationMetadata, ApplicationMetadataSource, Chain, Wallet, WalletConnection, WalletConnectionEvents, WalletConnectionMethods, WalletConnectionSession,
    WalletConnectionState, WalletId, WalletType,
};

use crate::models::gateway::GemGasPriceType;
use crate::models::transaction::{GemTransactionInputType, GemTransferDataExtra};
use crate::services::error::GemServiceError;
use crate::services::transfer::{GemRecipient, GemTransferData};
use crate::services::wallet_connect::model::{GemWalletConnectRpcError, GemWalletConnectTransactionAction};
use crate::wallet_connect::{EvmTransactionKind, WalletConnect, WalletConnectTransaction, wallet_connect_chain, wallet_connect_namespace};
use num_bigint::BigInt;
use primitives::{Asset, TransactionType, TransferDataOutputAction, TransferDataOutputType};

pub const USER_REJECTED_ERROR_CODE: i32 = 4001;

pub fn session_account(connection: &WalletConnection, chain: Chain) -> Result<Account, GemServiceError> {
    validate_session_chain(&connection.session, chain)?;
    connection
        .wallet
        .accounts
        .iter()
        .find(|account| account.chain == chain)
        .cloned()
        .ok_or_else(|| GemServiceError::NotFound {
            msg: format!("wallet has no {chain} account"),
        })
}

pub fn validate_session_chain(session: &WalletConnectionSession, chain: Chain) -> Result<(), GemServiceError> {
    if session.chains.contains(&chain) {
        return Ok(());
    }
    Err(GemServiceError::InvalidInput {
        msg: format!("chain {chain} is not part of the session"),
    })
}

pub fn sessions_to_delete(local: &[WalletConnectionSession], remote: &[WalletConnectionSession]) -> Vec<String> {
    stale(
        local
            .iter()
            .filter(|session| session.state == WalletConnectionState::Active)
            .map(|session| session.id.clone()),
        remote.iter().map(|session| session.id.clone()),
    )
}

pub fn sessions_to_update(local: &[WalletConnectionSession], remote: Vec<WalletConnectionSession>) -> Vec<WalletConnectionSession> {
    remote
        .into_iter()
        .filter(|session| local.iter().any(|existing| existing.id == session.id && session_changed(existing, session)))
        .collect()
}

fn session_changed(existing: &WalletConnectionSession, session: &WalletConnectionSession) -> bool {
    existing.state != session.state || existing.chains != session.chains || existing.expire_at != session.expire_at || existing.metadata != session.metadata
}

pub fn session_wallets(wallets: Vec<Wallet>, required: &[Chain], optional: &[Chain]) -> Vec<Wallet> {
    let mut supported: Vec<Wallet> = wallets
        .into_iter()
        .filter(|wallet| wallet.wallet_type != WalletType::View && supports(wallet, required, optional))
        .collect();
    supported.sort_by_key(|wallet| wallet.wallet_type.rank());
    supported
}

pub fn default_wallet(wallets: &[Wallet], current_wallet_id: Option<WalletId>) -> Option<Wallet> {
    wallets
        .iter()
        .find(|wallet| Some(&wallet.id) == current_wallet_id.as_ref())
        .or_else(|| wallets.first())
        .cloned()
}

pub fn supported_chains() -> Vec<Chain> {
    crate::config::wallet_connect::get_wallet_connect_config()
        .chains
        .iter()
        .filter_map(|chain| Chain::from_str(chain).ok())
        .collect()
}

pub fn session_chains(wallet: &Wallet, supported: &[Chain]) -> Vec<Chain> {
    let wallet_chains: HashSet<Chain> = wallet.accounts.iter().map(|account| account.chain).collect();
    supported.iter().copied().filter(|chain| wallet_chains.contains(chain)).collect()
}

pub fn parse_chains(chain_ids: &[String]) -> Option<Vec<Chain>> {
    let chains: Vec<Chain> = chain_ids.iter().filter_map(|chain_id| parse_chain(chain_id)).collect();
    (chains.len() == chain_ids.len()).then_some(chains)
}

pub fn parse_known_chains(chain_ids: &[String]) -> Vec<Chain> {
    chain_ids.iter().filter_map(|chain_id| parse_chain(chain_id)).collect()
}

pub fn authentication_chain_ids(chain_ids: &[String]) -> Vec<String> {
    unique(
        chain_ids
            .iter()
            .filter(|chain_id| parse_chain(chain_id).is_some_and(|chain| chain.chain_type() == ChainType::Ethereum))
            .cloned(),
    )
}

pub fn account_chains(accounts: &[String]) -> Vec<Chain> {
    let wallet_connect = WalletConnect::new();
    unique(
        accounts
            .iter()
            .filter_map(|account| wallet_connect.parse_account(account.clone()))
            .map(|address| address.chain),
    )
}

pub fn application_metadata(name: String, description: String, url: String, icons: Vec<String>) -> ApplicationMetadata {
    let icon = icons
        .iter()
        .find(|icon| [".png", ".jpg", ".jpeg", ".ico"].iter().any(|extension| icon.to_lowercase().contains(extension)))
        .or_else(|| icons.first())
        .cloned()
        .unwrap_or_default();
    let name = if name.trim().is_empty() { short_url(&url) } else { name };
    ApplicationMetadata {
        name,
        description,
        url,
        icon,
        source: ApplicationMetadataSource::WalletConnect,
    }
}

pub fn session(topic: String, chains: Vec<Chain>, expire_at: DateTime<Utc>, metadata: ApplicationMetadata) -> WalletConnectionSession {
    WalletConnectionSession {
        id: topic.clone(),
        session_id: topic,
        state: WalletConnectionState::Active,
        chains,
        created_at: Utc::now(),
        expire_at,
        metadata,
    }
}

pub fn user_rejected_error() -> GemWalletConnectRpcError {
    GemWalletConnectRpcError {
        code: USER_REJECTED_ERROR_CODE,
        message: "User rejected the request".to_string(),
    }
}

pub fn session_methods() -> Vec<String> {
    WalletConnectionMethods::all().iter().filter_map(serde_name).collect()
}

pub fn session_events() -> Vec<String> {
    WalletConnectionEvents::all().iter().filter_map(serde_name).collect()
}

fn serde_name<T: serde::Serialize>(value: &T) -> Option<String> {
    serde_json::to_value(value).ok().and_then(|value| value.as_str().map(String::from))
}

fn short_url(url: &str) -> String {
    url::Url::parse(url)
        .ok()
        .and_then(|url| url.host_str().map(|host| host.trim_start_matches("www.").to_string()))
        .unwrap_or_else(|| url.trim().to_string())
}

fn parse_chain(chain_id: &str) -> Option<Chain> {
    wallet_connect_chain(chain_id.to_string())
}

fn supports(wallet: &Wallet, required: &[Chain], optional: &[Chain]) -> bool {
    let chains: HashSet<Chain> = wallet
        .accounts
        .iter()
        .map(|account| account.chain)
        .filter(|chain| wallet_connect_namespace(*chain).is_some())
        .collect();
    if chains.is_empty() {
        return false;
    }
    if !required.is_empty() {
        return required.iter().all(|chain| chains.contains(chain));
    }
    optional.is_empty() || optional.iter().any(|chain| chains.contains(chain))
}

pub fn transfer_data(
    chain: Chain,
    metadata: ApplicationMetadata,
    transaction: WalletConnectTransaction,
    action: GemWalletConnectTransactionAction,
) -> Result<GemTransferData, GemServiceError> {
    let output_action = match action {
        GemWalletConnectTransactionAction::Sign => TransferDataOutputAction::Sign,
        GemWalletConnectTransactionAction::Send => TransferDataOutputAction::Send,
    };
    let (extra, value) = match transaction {
        WalletConnectTransaction::Ethereum { data, kind } => {
            let value = data.value.as_deref().map(hex_to_decimal).transpose()?.unwrap_or_else(|| "0".to_string());
            let gas_limit = data.gas_limit.as_deref().or(data.gas.as_deref()).map(hex_to_decimal).transpose()?;
            let gas_price = match (data.max_fee_per_gas.as_deref(), data.max_priority_fee_per_gas.as_deref()) {
                (Some(max_fee), Some(priority_fee)) => Some(GemGasPriceType::Eip1559 {
                    gas_price: hex_to_decimal(max_fee)?,
                    priority_fee: hex_to_decimal(priority_fee)?,
                }),
                _ => None,
            };
            let (transaction_type, approval) = match kind {
                EvmTransactionKind::Transfer => (TransactionType::Transfer, None),
                EvmTransactionKind::ContractCall => (TransactionType::SmartContractCall, None),
                EvmTransactionKind::TokenApproval { approval } => (TransactionType::TokenApproval, Some(approval)),
            };
            let extra = GemTransferDataExtra {
                to: data.to,
                gas_limit,
                gas_price,
                data: data.data.as_deref().map(hex_to_bytes).transpose()?,
                output_type: TransferDataOutputType::EncodedTransaction,
                output_action,
                transaction_type,
                approval,
            };
            (extra, value)
        }
        WalletConnectTransaction::Solana {
            data,
            output_type,
            transaction_type,
        } => (encoded_extra(data.transaction, output_type, output_action, transaction_type), "0".to_string()),
        WalletConnectTransaction::Sui { data, output_type } => (
            encoded_extra(data.transaction, output_type, output_action, TransactionType::SmartContractCall),
            "0".to_string(),
        ),
        WalletConnectTransaction::Ton { data, output_type } | WalletConnectTransaction::Tron { data, output_type } => {
            (encoded_extra(data, output_type, output_action, TransactionType::SmartContractCall), "0".to_string())
        }
    };
    Ok(GemTransferData {
        recipient: GemRecipient {
            address: extra.to.clone(),
            name: None,
            memo: None,
            references: vec![],
        },
        input_type: GemTransactionInputType::Generic {
            asset: Asset::from_chain(chain),
            metadata,
            extra,
        },
        value,
        use_max_amount: false,
        minimum_value: None,
    })
}

fn encoded_extra(encoded: String, output_type: TransferDataOutputType, output_action: TransferDataOutputAction, transaction_type: TransactionType) -> GemTransferDataExtra {
    GemTransferDataExtra {
        to: String::new(),
        gas_limit: None,
        gas_price: None,
        data: Some(encoded.into_bytes()),
        output_type,
        output_action,
        transaction_type,
        approval: None,
    }
}

fn hex_to_decimal(value: &str) -> Result<String, GemServiceError> {
    let digits = value.trim().trim_start_matches("0x").trim_start_matches("0X");
    if digits.is_empty() {
        return Ok("0".to_string());
    }
    BigInt::parse_bytes(digits.as_bytes(), 16)
        .map(|value| value.to_string())
        .ok_or_else(|| GemServiceError::InvalidInput {
            msg: format!("invalid hex number {value}"),
        })
}

fn hex_to_bytes(value: &str) -> Result<Vec<u8>, GemServiceError> {
    let digits = value.trim().trim_start_matches("0x").trim_start_matches("0X");
    hex::decode(digits).map_err(|_| GemServiceError::InvalidInput {
        msg: format!("invalid hex data {value}"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::Account;

    fn wallet(id: &str, wallet_type: WalletType, chains: &[Chain]) -> Wallet {
        Wallet {
            id: WalletId::Multicoin(id.to_string()),
            name: id.to_string(),
            wallet_type,
            ..Wallet::mock_with_accounts(Account::mock_chains(chains, "address"))
        }
    }

    fn session_with(id: &str, state: WalletConnectionState, chains: &[Chain]) -> WalletConnectionSession {
        WalletConnectionSession {
            state,
            ..session(
                id.to_string(),
                chains.to_vec(),
                Utc::now(),
                application_metadata("app".into(), String::new(), "https://app.example".into(), vec![]),
            )
        }
    }

    #[test]
    fn test_session_account_requires_session_chain_and_account() {
        let connection = WalletConnection {
            session: session_with("topic", WalletConnectionState::Active, &[Chain::Ethereum, Chain::Solana]),
            wallet: wallet("multi", WalletType::Multicoin, &[Chain::Ethereum]),
        };

        assert_eq!(session_account(&connection, Chain::Ethereum).unwrap().chain, Chain::Ethereum);
        assert!(session_account(&connection, Chain::Solana).is_err());
        assert!(session_account(&connection, Chain::Bitcoin).is_err());
        assert!(validate_session_chain(&connection.session, Chain::Solana).is_ok());
    }

    #[test]
    fn test_sessions_sync_rules() {
        let local = vec![
            session_with("active-kept", WalletConnectionState::Active, &[Chain::Ethereum]),
            session_with("active-gone", WalletConnectionState::Active, &[Chain::Ethereum]),
            session_with("started-gone", WalletConnectionState::Started, &[Chain::Ethereum]),
        ];
        let remote = vec![
            session_with("active-kept", WalletConnectionState::Active, &[Chain::Ethereum, Chain::Solana]),
            session_with("unknown", WalletConnectionState::Active, &[Chain::Ethereum]),
        ];

        assert_eq!(sessions_to_delete(&local, &remote), vec!["active-gone".to_string()]);
        let updates = sessions_to_update(&local, remote);
        assert_eq!(updates.iter().map(|session| session.id.as_str()).collect::<Vec<_>>(), vec!["active-kept"]);
        assert_eq!(updates[0].chains, vec![Chain::Ethereum, Chain::Solana]);
        assert!(sessions_to_delete(&local, &[]).contains(&"active-gone".to_string()));
        assert!(!sessions_to_delete(&local, &[]).contains(&"started-gone".to_string()));
    }

    #[test]
    fn test_session_wallets() {
        let multicoin = wallet("multi", WalletType::Multicoin, &[Chain::Ethereum, Chain::Solana]);
        let single = wallet("single", WalletType::Single, &[Chain::Ethereum]);
        let view = wallet("view", WalletType::View, &[Chain::Ethereum]);
        let bitcoin_only = wallet("btc", WalletType::Single, &[Chain::Bitcoin]);
        let wallets = vec![single.clone(), view, bitcoin_only, multicoin.clone()];

        let required = session_wallets(wallets.clone(), &[Chain::Ethereum, Chain::Solana], &[]);
        assert_eq!(required.iter().map(|wallet| wallet.name.as_str()).collect::<Vec<_>>(), vec!["multi"]);

        let optional = session_wallets(wallets.clone(), &[], &[Chain::Ethereum]);
        assert_eq!(optional.iter().map(|wallet| wallet.name.as_str()).collect::<Vec<_>>(), vec!["multi", "single"]);

        let any = session_wallets(wallets, &[], &[]);
        assert_eq!(any.len(), 2);
    }

    #[test]
    fn test_default_wallet_prefers_current() {
        let first = wallet("first", WalletType::Multicoin, &[Chain::Ethereum]);
        let second = wallet("second", WalletType::Multicoin, &[Chain::Ethereum]);
        let wallets = vec![first.clone(), second.clone()];
        assert_eq!(default_wallet(&wallets, Some(second.id.clone())).map(|wallet| wallet.name), Some("second".to_string()));
        assert_eq!(
            default_wallet(&wallets, Some(WalletId::Multicoin("other".to_string()))).map(|wallet| wallet.name),
            Some("first".to_string())
        );
        assert!(default_wallet(&[], None).is_none());
    }

    #[test]
    fn test_session_chains_keeps_supported_order() {
        let wallet = wallet("w", WalletType::Multicoin, &[Chain::Solana, Chain::Ethereum, Chain::Bitcoin]);
        assert_eq!(
            session_chains(&wallet, &[Chain::Ethereum, Chain::Solana, Chain::Tron]),
            vec![Chain::Ethereum, Chain::Solana]
        );
    }

    #[test]
    fn test_parse_chains_and_metadata() {
        assert_eq!(parse_chains(&["eip155:1".to_string()]), Some(vec![Chain::Ethereum]));
        assert_eq!(parse_chains(&["eip155:1".to_string(), "cosmos:unknown-9".to_string()]), None);
        assert_eq!(parse_known_chains(&["eip155:1".to_string(), "cosmos:unknown-9".to_string()]), vec![Chain::Ethereum]);
        assert_eq!(
            account_chains(&["eip155:1:0xabc".to_string(), "eip155:137:0xabc".to_string(), "eip155:1:0xdef".to_string()]),
            vec![Chain::Ethereum, Chain::Polygon]
        );
        assert_eq!(
            authentication_chain_ids(&[
                "eip155:1".to_string(),
                "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp".to_string(),
                "eip155:1".to_string(),
                "eip155:137".to_string()
            ]),
            vec!["eip155:1".to_string(), "eip155:137".to_string()]
        );

        let metadata = application_metadata(
            " ".to_string(),
            "desc".to_string(),
            "https://www.app.example.com/path".to_string(),
            vec!["https://x/icon.svg".to_string(), "https://x/icon.PNG".to_string()],
        );
        assert_eq!(metadata.name, "app.example.com");
        assert_eq!(metadata.icon, "https://x/icon.PNG");
        assert!(session_methods().contains(&"personal_sign".to_string()));
        assert!(session_events().contains(&"accountsChanged".to_string()));
    }

    #[test]
    fn test_transfer_data_maps_evm_and_encoded_transactions() {
        let metadata = application_metadata("app".into(), String::new(), "https://app.example".into(), vec![]);
        let evm = WalletConnectTransaction::Ethereum {
            data: crate::wallet_connect::WCEthereumTransactionData {
                chain_id: Some(1),
                from: "0xfrom".to_string(),
                to: "0xto".to_string(),
                value: Some("0x10".to_string()),
                gas: Some("0x5208".to_string()),
                gas_limit: None,
                gas_price: None,
                max_fee_per_gas: Some("0x64".to_string()),
                max_priority_fee_per_gas: Some("0x2".to_string()),
                nonce: None,
                data: Some("0xdeadbeef".to_string()),
            },
            kind: EvmTransactionKind::TokenApproval {
                approval: primitives::swap::ApprovalData::mock(),
            },
        };

        let transfer = transfer_data(Chain::Ethereum, metadata.clone(), evm, GemWalletConnectTransactionAction::Send).unwrap();

        assert_eq!(transfer.value, "16");
        assert_eq!(transfer.recipient.address, "0xto");
        let GemTransactionInputType::Generic { asset, extra, .. } = transfer.input_type else {
            panic!("expected a generic input");
        };
        assert_eq!(asset.id, primitives::AssetId::from_chain(Chain::Ethereum));
        assert_eq!(extra.gas_limit.as_deref(), Some("21000"));
        assert!(matches!(extra.gas_price, Some(GemGasPriceType::Eip1559 { ref gas_price, ref priority_fee }) if gas_price == "100" && priority_fee == "2"));
        assert_eq!(extra.data, Some(vec![0xde, 0xad, 0xbe, 0xef]));
        assert_eq!(extra.transaction_type, TransactionType::TokenApproval);
        assert!(extra.approval.is_some());
        assert_eq!(extra.output_action, TransferDataOutputAction::Send);

        let solana = WalletConnectTransaction::Solana {
            data: crate::wallet_connect::WCSolanaTransactionData { transaction: "AQID".to_string() },
            output_type: TransferDataOutputType::Signature,
            transaction_type: TransactionType::Swap,
        };
        let transfer = transfer_data(Chain::Solana, metadata, solana, GemWalletConnectTransactionAction::Sign).unwrap();
        assert_eq!(transfer.value, "0");
        assert_eq!(transfer.recipient.address, "");
        let GemTransactionInputType::Generic { extra, .. } = transfer.input_type else {
            panic!("expected a generic input");
        };
        assert_eq!(extra.data, Some(b"AQID".to_vec()));
        assert_eq!(extra.output_type, TransferDataOutputType::Signature);
        assert_eq!(extra.output_action, TransferDataOutputAction::Sign);
        assert_eq!(extra.transaction_type, TransactionType::Swap);
    }
}

pub fn is_origin_rejected(status: &WalletConnectionVerificationStatus) -> bool {
    match status {
        WalletConnectionVerificationStatus::Invalid | WalletConnectionVerificationStatus::Malicious => true,
        WalletConnectionVerificationStatus::Unknown | WalletConnectionVerificationStatus::Verified => false,
    }
}

pub fn record_seen_message(seen: &mut Vec<String>, message_id: String, limit: usize) -> bool {
    if seen.contains(&message_id) {
        return false;
    }
    if seen.len() >= limit {
        seen.remove(0);
    }
    seen.push(message_id);
    true
}

#[cfg(test)]
mod message_tests {
    use super::*;

    #[test]
    fn test_a_repeated_message_is_only_processed_once() {
        let mut seen = Vec::new();

        assert!(record_seen_message(&mut seen, "a".to_string(), 3));
        assert!(!record_seen_message(&mut seen, "a".to_string(), 3), "a relay retry must not reach the signer twice");
        assert!(record_seen_message(&mut seen, "b".to_string(), 3));
    }

    #[test]
    fn test_the_seen_list_evicts_oldest_first_and_stays_bounded() {
        let mut seen = Vec::new();
        for id in ["a", "b", "c", "d"] {
            record_seen_message(&mut seen, id.to_string(), 3);
        }

        assert_eq!(seen, vec!["b".to_string(), "c".to_string(), "d".to_string()]);
        assert!(record_seen_message(&mut seen, "a".to_string(), 3), "an evicted id is no longer remembered");
    }

    #[test]
    fn test_only_invalid_and_malicious_origins_are_rejected() {
        assert!(is_origin_rejected(&WalletConnectionVerificationStatus::Invalid));
        assert!(is_origin_rejected(&WalletConnectionVerificationStatus::Malicious));
        assert!(!is_origin_rejected(&WalletConnectionVerificationStatus::Verified));
        assert!(!is_origin_rejected(&WalletConnectionVerificationStatus::Unknown));
    }
}
