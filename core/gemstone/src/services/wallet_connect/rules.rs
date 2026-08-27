use std::collections::HashSet;

use crate::services::collections::{stale, unique};

use chrono::{DateTime, Utc};
use primitives::ChainType;
use primitives::{
    Account, ApplicationMetadata, ApplicationMetadataSource, Chain, Wallet, WalletConnection, WalletConnectionEvents, WalletConnectionMethods, WalletConnectionSession,
    WalletConnectionState, WalletId, WalletType,
};

use crate::services::error::GemServiceError;
use crate::wallet_connect::WalletConnect;

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
    let wallet_connect = WalletConnect::new();
    let mut supported: Vec<Wallet> = wallets
        .into_iter()
        .filter(|wallet| wallet.wallet_type != WalletType::View && supports(wallet, required, optional, &wallet_connect))
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

pub fn session_chains(wallet: &Wallet, supported: &[Chain]) -> Vec<Chain> {
    let wallet_chains: HashSet<Chain> = wallet.accounts.iter().map(|account| account.chain).collect();
    supported.iter().copied().filter(|chain| wallet_chains.contains(chain)).collect()
}

pub fn parse_chains(wallet_connect: &WalletConnect, chain_ids: &[String]) -> Option<Vec<Chain>> {
    let chains: Vec<Chain> = chain_ids.iter().filter_map(|chain_id| parse_chain(wallet_connect, chain_id)).collect();
    (chains.len() == chain_ids.len()).then_some(chains)
}

pub fn parse_known_chains(wallet_connect: &WalletConnect, chain_ids: &[String]) -> Vec<Chain> {
    chain_ids.iter().filter_map(|chain_id| parse_chain(wallet_connect, chain_id)).collect()
}

pub fn authentication_chain_ids(wallet_connect: &WalletConnect, chain_ids: &[String]) -> Vec<String> {
    unique(
        chain_ids
            .iter()
            .filter(|chain_id| parse_chain(wallet_connect, chain_id).is_some_and(|chain| chain.chain_type() == ChainType::Ethereum))
            .cloned(),
    )
}

pub fn account_chains(wallet_connect: &WalletConnect, accounts: &[String]) -> Vec<Chain> {
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

fn parse_chain(wallet_connect: &WalletConnect, chain_id: &str) -> Option<Chain> {
    wallet_connect.parse_chain_id(chain_id.to_string()).and_then(|chain| chain.parse().ok())
}

fn supports(wallet: &Wallet, required: &[Chain], optional: &[Chain], wallet_connect: &WalletConnect) -> bool {
    let chains: HashSet<Chain> = wallet
        .accounts
        .iter()
        .map(|account| account.chain)
        .filter(|chain| wallet_connect.get_namespace(chain.as_ref().to_string()).is_some())
        .collect();
    if chains.is_empty() {
        return false;
    }
    if !required.is_empty() {
        return required.iter().all(|chain| chains.contains(chain));
    }
    optional.is_empty() || optional.iter().any(|chain| chains.contains(chain))
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{Account, WalletSource};

    fn wallet(id: &str, wallet_type: WalletType, chains: &[Chain]) -> Wallet {
        Wallet {
            id: WalletId::Multicoin(id.to_string()),
            external_id: None,
            name: id.to_string(),
            index: 0,
            wallet_type,
            accounts: chains
                .iter()
                .map(|chain| Account {
                    chain: *chain,
                    address: "address".to_string(),
                    derivation_path: String::new(),
                    extended_public_key: None,
                })
                .collect(),
            is_pinned: false,
            image_url: None,
            source: WalletSource::Import,
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
        let wallet_connect = WalletConnect::new();
        assert_eq!(parse_chains(&wallet_connect, &["eip155:1".to_string()]), Some(vec![Chain::Ethereum]));
        assert_eq!(parse_chains(&wallet_connect, &["eip155:1".to_string(), "cosmos:unknown-9".to_string()]), None);
        assert_eq!(
            parse_known_chains(&wallet_connect, &["eip155:1".to_string(), "cosmos:unknown-9".to_string()]),
            vec![Chain::Ethereum]
        );
        assert_eq!(
            account_chains(
                &wallet_connect,
                &["eip155:1:0xabc".to_string(), "eip155:137:0xabc".to_string(), "eip155:1:0xdef".to_string()]
            ),
            vec![Chain::Ethereum, Chain::Polygon]
        );
        assert_eq!(
            authentication_chain_ids(
                &wallet_connect,
                &[
                    "eip155:1".to_string(),
                    "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp".to_string(),
                    "eip155:1".to_string(),
                    "eip155:137".to_string()
                ]
            ),
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
}
