use crate::constants::{WALLET_CONNECT_USER_REJECTED_ERROR_CODE, WALLET_CONNECT_USER_REJECTED_ERROR_MESSAGE};
use gem_evm::transaction::decode_calldata;
use std::collections::HashSet;
use std::str::FromStr;

use crate::application::url_host;
use crate::services::collections::{stale, unique};

use chrono::{DateTime, Utc};
use gem_solana::{Pubkey, SolanaTransfer, VersionedTransaction, VersionedTransactionExt};
use gem_sui::tx_builder::validate_wallet_connect_sender;
use gem_ton::models::decode_wallet_connect_transfer;
use gem_tron::{decode_wallet_connect_contract, decode_wallet_connect_owners};
use primitives::WalletConnectionVerificationStatus;
use primitives::{
    Account, ApplicationMetadata, ApplicationMetadataSource, Chain, Platform, Wallet, WalletConnectCAIP2, WalletConnection, WalletConnectionEvents, WalletConnectionMethods, WalletConnectionSession, WalletConnectionState, WalletId,
    WalletType,
};
use primitives::{ChainType, GEM_HOST, GEM_URL_SCHEME, HTTP_URL_SCHEME, HTTPS_URL_SCHEME, WALLET_CONNECT_URL_SCHEME, WalletConnectLink};
use url::Url;

use crate::models::list::{GemListRow, GemListRowTitle};
use crate::services::error::GemServiceError;
use crate::services::error_text::GemErrorText;
use crate::services::transfer::{GemRecipient, GemTransferData};
use crate::services::wallet_connect::model::{
    GemSignerFailure, GemWalletConnectAuthAccount, GemWalletConnectOutcome, GemWalletConnectRejection, GemWalletConnectRejectionReason, GemWalletConnectResponse, GemWalletConnectRpcError, GemWalletConnectTransactionAction,
};
use crate::wallet_connect::WalletConnect;
use gem_wallet_connect::{EvmTransactionKind, WalletConnectResponseType, WalletConnectTransaction};
use num_bigint::BigInt;
use primitives::GasPriceType;
use primitives::TransactionInputType;
use primitives::{Asset, TransactionType, TransferDataExtra, TransferDataOutputAction, TransferDataOutputType};
use std::collections::BTreeMap;
use wallet_connect_protocol::sign::SessionNamespace;
use wallet_connect_protocol::sign::namespaces::NamespaceError;
use wallet_connect_protocol::sign::types::Redirect;

const UNSUPPORTED_CHAINS_ERROR_CODE: i32 = 5100;
const UNSUPPORTED_METHODS_ERROR_CODE: i32 = 5101;
const UNSUPPORTED_ACCOUNTS_ERROR_CODE: i32 = 5103;
const UNSUPPORTED_EVENTS_ERROR_CODE: i32 = 5102;
const SESSION_REQUEST_EXPIRED_ERROR_CODE: i32 = 8000;
const USER_REJECTED_SESSION_ERROR_CODE: i32 = 5000;
const METHOD_NOT_FOUND_ERROR_CODE: i32 = -32601;
const UNRECOGNIZED_CHAIN_ERROR_CODE: i32 = 4902;
const BLOCKED_APP_LINK_SCHEMES: [&str; 33] = [
    GEM_URL_SCHEME,
    WALLET_CONNECT_URL_SCHEME,
    HTTP_URL_SCHEME,
    "javascript",
    "data",
    "file",
    "about",
    "blob",
    "content",
    "intent",
    "ftp",
    "tel",
    "telprompt",
    "sms",
    "smsto",
    "mms",
    "mmsto",
    "mailto",
    "facetime",
    "facetime-audio",
    "itms",
    "itms-apps",
    "itms-appss",
    "itms-services",
    "itms-books",
    "shortcuts",
    "workflow",
    "prefs",
    "app-prefs",
    "app-settings",
    "maps",
    "geo",
    "market",
];

pub fn connection_groups(connections: Vec<WalletConnection>) -> Vec<(Wallet, Vec<WalletConnection>)> {
    let mut groups: Vec<(Wallet, Vec<WalletConnection>)> = Vec::new();
    for connection in connections {
        match groups.iter_mut().find(|(wallet, _)| wallet.id == connection.wallet.id) {
            Some((_, grouped)) => grouped.push(connection),
            None => groups.push((connection.wallet.clone(), vec![connection])),
        }
    }
    groups.sort_by_key(|(wallet, _)| wallet.index);
    for (_, grouped) in groups.iter_mut() {
        grouped.sort_by_key(|connection| std::cmp::Reverse(connection.session.created_at));
    }
    groups
}

pub fn connection_detail_rows(connection: &WalletConnection) -> Vec<GemListRow> {
    vec![
        GemListRow::Text {
            title: GemListRowTitle::Wallet,
            value: connection.wallet.name.clone(),
        },
        GemListRow::Date {
            title: GemListRowTitle::Date,
            date: connection.session.created_at,
        },
    ]
}

pub fn session_account(connection: &WalletConnection, chain: Chain) -> Result<Account, GemServiceError> {
    validate_session_chain(&connection.session, chain)?;
    connection.wallet.accounts.iter().find(|account| account.chain == chain).cloned().ok_or_else(|| GemServiceError::NotFound {
        msg: format!("wallet has no {chain} account"),
    })
}

pub fn add_chain_outcome(connection: &WalletConnection, chain_id: u64) -> GemWalletConnectOutcome {
    match switchable_account(connection, chain_id) {
        Some(_) => GemWalletConnectOutcome::respond_null(),
        None => GemWalletConnectOutcome::respond_error(unrecognized_chain_error()),
    }
}

pub fn switch_chain_outcome(connection: &WalletConnection, chain_id: u64) -> GemWalletConnectOutcome {
    let Some(account) = switchable_account(connection, chain_id) else {
        return GemWalletConnectOutcome::respond_error(unrecognized_chain_error());
    };
    if connection.session.chains.contains(&account.chain) {
        return GemWalletConnectOutcome::respond_null();
    }
    match WalletConnectCAIP2::format_account(account.chain, &account.address) {
        Some(account) => GemWalletConnectOutcome::AddSessionAccount { account },
        None => GemWalletConnectOutcome::respond_error(unrecognized_chain_error()),
    }
}

pub fn chain_id_outcome(chain_id: u64) -> GemWalletConnectOutcome {
    GemWalletConnectOutcome::Respond {
        response: GemWalletConnectResponse::Response {
            value: WalletConnectResponseType::String { value: format!("0x{chain_id:x}") },
        },
    }
}

fn switchable_account(connection: &WalletConnection, chain_id: u64) -> Option<&Account> {
    let chain = supported_chains().into_iter().find(|chain| chain.chain_type() == ChainType::Ethereum && chain.network_id_value() == Some(chain_id))?;
    let account = connection.wallet.accounts.iter().find(|account| account.chain == chain)?;
    let is_shared = connection
        .wallet
        .accounts
        .iter()
        .any(|shared| shared.chain.chain_type() == ChainType::Ethereum && connection.session.chains.contains(&shared.chain) && shared.address == account.address);
    is_shared.then_some(account)
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
        local.iter().filter(|session| session.state == WalletConnectionState::Active).map(|session| session.id.clone()),
        remote.iter().map(|session| session.id.clone()),
    )
}

pub fn sessions_to_update(local: &[WalletConnectionSession], remote: Vec<WalletConnectionSession>) -> Vec<WalletConnectionSession> {
    remote.into_iter().filter(|session| local.iter().any(|existing| existing.id == session.id && session_changed(existing, session))).collect()
}

fn session_changed(existing: &WalletConnectionSession, session: &WalletConnectionSession) -> bool {
    existing.state != session.state || existing.chains != session.chains || existing.expire_at != session.expire_at || existing.metadata != session.metadata
}

pub fn session_wallets(wallets: Vec<Wallet>, required: &[Chain], optional: &[Chain]) -> Vec<Wallet> {
    let mut supported: Vec<Wallet> = wallets.into_iter().filter(|wallet| wallet.wallet_type != WalletType::View && supports(wallet, required, optional)).collect();
    supported.sort_by_key(|wallet| wallet.wallet_type.rank());
    supported
}

pub fn default_wallet(wallets: &[Wallet], current_wallet_id: Option<WalletId>) -> Option<Wallet> {
    wallets.iter().find(|wallet| Some(&wallet.id) == current_wallet_id.as_ref()).or_else(|| wallets.first()).cloned()
}

pub fn supported_chains() -> Vec<Chain> {
    crate::config::wallet_connect::get_wallet_connect_config().chains.iter().filter_map(|chain| Chain::from_str(chain).ok()).collect()
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
    unique(chain_ids.iter().filter(|chain_id| parse_chain(chain_id).is_some_and(|chain| chain.chain_type() == ChainType::Ethereum)).cloned())
}

pub fn authentication_accounts(chain_ids: &[String], wallet: &Wallet) -> Vec<GemWalletConnectAuthAccount> {
    authentication_chain_ids(chain_ids)
        .into_iter()
        .filter_map(|chain_id| {
            let chain = parse_chain(&chain_id)?;
            let account = wallet.accounts.iter().find(|account| account.chain == chain)?;
            Some(GemWalletConnectAuthAccount {
                issuer: format!("did:pkh:{chain_id}:{}", account.address),
                account: account.clone(),
                chain_id,
            })
        })
        .collect()
}

pub fn account_chains(accounts: &[String]) -> Vec<Chain> {
    let wallet_connect = WalletConnect::new();
    unique(accounts.iter().filter_map(|account| wallet_connect.parse_account(account.clone())).map(|address| address.chain))
}

pub fn application_metadata(name: String, description: String, url: String, icons: Vec<String>) -> ApplicationMetadata {
    let icon = icons
        .iter()
        .find(|icon| [".png", ".jpg", ".jpeg", ".ico"].iter().any(|extension| icon.to_lowercase().contains(extension)))
        .or_else(|| icons.first())
        .cloned()
        .unwrap_or_default();
    let name = if name.trim().is_empty() { url_host(&url) } else { name };
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

pub fn session_rejection(reason: GemWalletConnectRejectionReason) -> GemWalletConnectRejection {
    let (code, message) = match reason {
        GemWalletConnectRejectionReason::UserRejected => (USER_REJECTED_SESSION_ERROR_CODE, "User rejected."),
        GemWalletConnectRejectionReason::UnsupportedChains => (UNSUPPORTED_CHAINS_ERROR_CODE, "Unsupported chains"),
        GemWalletConnectRejectionReason::UnsupportedMethods => (UNSUPPORTED_METHODS_ERROR_CODE, "Unsupported methods"),
        GemWalletConnectRejectionReason::UnsupportedAccounts => (UNSUPPORTED_ACCOUNTS_ERROR_CODE, "Unsupported accounts"),
        GemWalletConnectRejectionReason::UnsupportedEvents => (UNSUPPORTED_EVENTS_ERROR_CODE, "Unsupported events"),
    };
    GemWalletConnectRejection {
        reason,
        code,
        message: message.to_string(),
        deletes_session: true,
    }
}

pub fn namespace_rejection(error: NamespaceError) -> GemWalletConnectRejectionReason {
    match error {
        NamespaceError::UnsupportedChains => GemWalletConnectRejectionReason::UnsupportedChains,
        NamespaceError::UnsupportedAccounts => GemWalletConnectRejectionReason::UnsupportedAccounts,
        NamespaceError::UnsupportedMethods => GemWalletConnectRejectionReason::UnsupportedMethods,
        NamespaceError::UnsupportedEvents => GemWalletConnectRejectionReason::UnsupportedEvents,
    }
}

pub const BITCOIN_ADDRESSES_CHANGED_EVENT: &str = "bip122_addressesChanged";

pub fn bitcoin_address_event_chains(namespaces: &BTreeMap<String, SessionNamespace>) -> Option<Vec<String>> {
    let namespace = namespaces.get(WalletConnectCAIP2::Bip122.as_ref())?;
    if !namespace.events.iter().any(|event| event == BITCOIN_ADDRESSES_CHANGED_EVENT) {
        return None;
    }
    Some(unique(namespace.accounts.iter().filter_map(|account| account.rsplit_once(':').map(|(chain_id, _)| chain_id.to_string()))))
}

pub fn caip2_chain_ids(chain: Chain) -> Vec<String> {
    let Some(namespace) = WalletConnectCAIP2::get_namespace(chain) else {
        return vec![];
    };
    WalletConnectCAIP2::get_references(chain).into_iter().map(|reference| format!("{namespace}:{reference}")).collect()
}

pub fn redirect(platform: Platform) -> String {
    match platform {
        Platform::IOS => WalletConnectLink::app_link(),
        Platform::Android => WalletConnectLink::universal_link(),
    }
}

pub fn dapp_redirect(redirect: Option<&Redirect>, dapp_url: &str) -> Option<String> {
    let redirect = redirect?;
    let dapp_host = Url::parse(dapp_url).ok()?.host_str()?.to_string();
    [redirect.native.as_deref(), redirect.universal.as_deref()]
        .into_iter()
        .flatten()
        .find(|link| is_dapp_redirect(link, &dapp_host))
        .map(str::to_string)
}

fn is_dapp_redirect(link: &str, dapp_host: &str) -> bool {
    let Ok(url) = Url::parse(link) else {
        return false;
    };
    match url.scheme() {
        HTTPS_URL_SCHEME => url.host_str().is_some_and(|host| is_same_or_subdomain(host, dapp_host) && !is_same_or_subdomain(host, GEM_HOST)),
        scheme => !BLOCKED_APP_LINK_SCHEMES.contains(&scheme) && !scheme.starts_with("x-apple"),
    }
}

fn is_same_or_subdomain(host: &str, domain: &str) -> bool {
    host == domain || host.strip_suffix(domain).is_some_and(|prefix| prefix.ends_with('.'))
}

pub fn user_rejected_error() -> GemWalletConnectRpcError {
    GemWalletConnectRpcError {
        code: WALLET_CONNECT_USER_REJECTED_ERROR_CODE,
        message: WALLET_CONNECT_USER_REJECTED_ERROR_MESSAGE.to_string(),
    }
}

pub fn signer_failure(error: GemErrorText) -> GemSignerFailure {
    match error {
        GemErrorText::Cancelled => GemSignerFailure::Reject,
        error => GemSignerFailure::Retry { error },
    }
}

pub fn request_expired_error() -> GemWalletConnectRpcError {
    GemWalletConnectRpcError {
        code: SESSION_REQUEST_EXPIRED_ERROR_CODE,
        message: "Session request expired".to_string(),
    }
}

pub fn is_expired(expiry: Option<u64>, now: DateTime<Utc>) -> bool {
    expiry.is_some_and(|expiry| now.timestamp() >= expiry as i64)
}

pub fn unrecognized_chain_error() -> GemWalletConnectRpcError {
    GemWalletConnectRpcError {
        code: UNRECOGNIZED_CHAIN_ERROR_CODE,
        message: "Unrecognized chain ID".to_string(),
    }
}

pub fn method_not_found_error() -> GemWalletConnectRpcError {
    GemWalletConnectRpcError {
        code: METHOD_NOT_FOUND_ERROR_CODE,
        message: "Method not found".to_string(),
    }
}

pub fn proposal_message_id(proposer_public_key: &str) -> String {
    format!("proposal-{proposer_public_key}")
}

pub fn request_message_id(topic: &str, request_id: &str) -> String {
    format!("request-{topic}-{request_id}")
}

pub fn session_methods() -> Vec<String> {
    WalletConnectionMethods::all().iter().filter_map(serde_name).collect()
}

pub fn authentication_methods() -> Vec<String> {
    WalletConnectionMethods::all().iter().filter(|method| method.chain_type() == ChainType::Ethereum).filter_map(serde_name).collect()
}

pub fn session_events() -> Vec<String> {
    WalletConnectionEvents::all().iter().filter_map(serde_name).collect()
}

fn serde_name<T: serde::Serialize>(value: &T) -> Option<String> {
    serde_json::to_value(value).ok().and_then(|value| value.as_str().map(String::from))
}

fn parse_chain(chain_id: &str) -> Option<Chain> {
    WalletConnectCAIP2::parse_chain_id(chain_id.to_string())
}

fn supports(wallet: &Wallet, required: &[Chain], optional: &[Chain]) -> bool {
    let chains: HashSet<Chain> = wallet.accounts.iter().map(|account| account.chain).filter(|chain| WalletConnectCAIP2::get_namespace(*chain).is_some()).collect();
    if chains.is_empty() {
        return false;
    }
    if !required.is_empty() {
        return required.iter().all(|chain| chains.contains(chain));
    }
    optional.is_empty() || optional.iter().any(|chain| chains.contains(chain))
}

pub fn validate_transaction_sender(transaction: &WalletConnectTransaction, account: &Account) -> Result<(), GemServiceError> {
    match transaction {
        WalletConnectTransaction::Ethereum { data, .. } if !data.from.eq_ignore_ascii_case(&account.address) => Err(GemServiceError::InvalidInput {
            msg: format!("transaction sender {} is not the session account {}", data.from, account.address),
        }),
        WalletConnectTransaction::Tron { data, .. } => {
            let owners = decode_wallet_connect_owners(data).map_err(|error| GemServiceError::InvalidInput { msg: error.to_string() })?;
            match owners.iter().find(|owner| **owner != account.address) {
                Some(owner) => Err(GemServiceError::InvalidInput {
                    msg: format!("transaction sender {owner} is not the session account {}", account.address),
                }),
                None => Ok(()),
            }
        }
        WalletConnectTransaction::Sui { data, .. } => validate_wallet_connect_sender(&data.transaction, &account.address).map_err(|error| GemServiceError::InvalidInput { msg: error.to_string() }),
        WalletConnectTransaction::BitcoinPsbt { request, .. } if request.account != account.address => Err(GemServiceError::InvalidInput {
            msg: format!("transaction sender {} is not the session account {}", request.account, account.address),
        }),
        WalletConnectTransaction::BitcoinTransfer { data } if data.account != account.address => Err(GemServiceError::InvalidInput {
            msg: format!("transaction sender {} is not the session account {}", data.account, account.address),
        }),
        _ => Ok(()),
    }
}

pub fn transfer_data(metadata: ApplicationMetadata, transaction: WalletConnectTransaction, account: &Account, action: GemWalletConnectTransactionAction) -> Result<GemTransferData, GemServiceError> {
    let output_action = match action {
        GemWalletConnectTransactionAction::Sign => TransferDataOutputAction::Sign,
        GemWalletConnectTransactionAction::Send => TransferDataOutputAction::Send,
    };
    let (extra, value) = match transaction {
        WalletConnectTransaction::Ethereum { data, kind } => {
            let value = data.value.map(BigInt::from).unwrap_or(BigInt::ZERO);
            let gas_limit = data.gas_limit.as_deref().or(data.gas.as_deref()).map(hex_value).transpose()?.filter(|gas_limit| *gas_limit > BigInt::ZERO);
            let gas_price = match (data.max_fee_per_gas.as_deref(), data.max_priority_fee_per_gas.as_deref()) {
                (Some(max_fee), Some(priority_fee)) => Some(GasPriceType::Eip1559 {
                    gas_price: hex_value(max_fee)?,
                    priority_fee: hex_value(priority_fee)?,
                }),
                _ => None,
            };
            let (transaction_type, approval) = match kind {
                EvmTransactionKind::Transfer => (TransactionType::Transfer, None),
                EvmTransactionKind::ContractCall => (TransactionType::SmartContractCall, None),
                EvmTransactionKind::TokenApproval(approval) => (TransactionType::TokenApproval, Some(approval)),
            };
            let extra = TransferDataExtra {
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
        WalletConnectTransaction::Solana { data, output_type } => {
            let (to, value) = match solana_native_transfer(&data.decoded, &account.address) {
                Some(transfer) => (transfer.recipient, BigInt::from(transfer.value)),
                None => (String::new(), BigInt::ZERO),
            };
            let extra = TransferDataExtra {
                to,
                ..encoded_extra(data.transaction, output_type, output_action, data.decoded.transaction_type())
            };
            (extra, value)
        }
        WalletConnectTransaction::Sui { data, output_type } => (encoded_extra(data.transaction, output_type, output_action, TransactionType::SmartContractCall), BigInt::ZERO),
        WalletConnectTransaction::Tron { data, output_type } => {
            let (to, value, approval) = match decode_wallet_connect_contract(&data).map_err(|error| GemServiceError::InvalidInput { msg: error.to_string() })? {
                Some(contract) => (contract.to, BigInt::from(contract.value), contract.approval),
                None => (String::new(), BigInt::ZERO, None),
            };
            let transaction_type = if approval.is_some() { TransactionType::TokenApproval } else { TransactionType::SmartContractCall };
            let extra = TransferDataExtra {
                to,
                approval: approval.map(|decoded| decoded.approval),
                ..encoded_extra(data, output_type, output_action, transaction_type)
            };
            (extra, value)
        }
        WalletConnectTransaction::BitcoinPsbt { request, data } => {
            let summary = request.summary(&account.address).map_err(|error| GemServiceError::InvalidInput { msg: error.to_string() })?;
            let extra = TransferDataExtra {
                to: summary.recipient.unwrap_or_default(),
                ..encoded_extra(data, TransferDataOutputType::EncodedTransaction, output_action, TransactionType::Transfer)
            };
            (extra, BigInt::from(summary.value))
        }
        WalletConnectTransaction::BitcoinTransfer { data } => {
            return Ok(GemTransferData {
                recipient: GemRecipient {
                    address: data.recipient_address,
                    name: None,
                    memo: data.memo,
                    references: vec![],
                },
                input_type: TransactionInputType::Transfer { asset: Asset::from_chain(account.chain) },
                value: BigInt::from(data.amount),
                use_max_amount: false,
            });
        }
        WalletConnectTransaction::Ton { data, output_type } => {
            let transfer = decode_wallet_connect_transfer(&data).map_err(|msg| GemServiceError::InvalidInput { msg })?;
            let extra = TransferDataExtra {
                to: transfer.to,
                ..encoded_extra(data, output_type, output_action, TransactionType::SmartContractCall)
            };
            (extra, BigInt::from(transfer.value))
        }
    };
    Ok(GemTransferData {
        recipient: GemRecipient {
            address: extra.to.clone(),
            name: None,
            memo: None,
            references: vec![],
        },
        input_type: TransactionInputType::Generic {
            asset: Asset::from_chain(account.chain),
            metadata,
            extra,
        },
        value,
        use_max_amount: false,
    })
}

fn solana_native_transfer(transaction: &VersionedTransaction, sender: &str) -> Option<SolanaTransfer> {
    let sender = Pubkey::from_base58(sender).ok()?;
    transaction.simple_transfer(&sender).filter(|transfer| transfer.asset_id.is_native())
}

fn encoded_extra(encoded: String, output_type: TransferDataOutputType, output_action: TransferDataOutputAction, transaction_type: TransactionType) -> TransferDataExtra {
    TransferDataExtra {
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

fn hex_value(value: &str) -> Result<BigInt, GemServiceError> {
    let digits = value.trim().trim_start_matches("0x").trim_start_matches("0X");
    if digits.is_empty() {
        return Ok(BigInt::ZERO);
    }
    BigInt::parse_bytes(digits.as_bytes(), 16).ok_or_else(|| GemServiceError::InvalidInput { msg: format!("invalid hex number {value}") })
}

fn hex_to_bytes(value: &str) -> Result<Vec<u8>, GemServiceError> {
    decode_calldata(value).map_err(|_| GemServiceError::InvalidInput { msg: format!("invalid hex data {value}") })
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_bitcoin_addresses_are_announced_only_on_a_bitcoin_session_that_listens() {
        let namespace = |events: &[&str]| {
            BTreeMap::from([(
                "bip122".to_string(),
                SessionNamespace {
                    chains: None,
                    accounts: vec!["bip122:000000000019d6689c085ae165831e93:bc1qaddress".to_string()],
                    methods: vec![],
                    events: events.iter().map(|event| event.to_string()).collect(),
                },
            )])
        };
        assert_eq!(bitcoin_address_event_chains(&namespace(&[BITCOIN_ADDRESSES_CHANGED_EVENT])), Some(vec!["bip122:000000000019d6689c085ae165831e93".to_string()]));
        assert_eq!(bitcoin_address_event_chains(&namespace(&[])), None, "a dapp that did not ask for the event never gets it");
        assert_eq!(bitcoin_address_event_chains(&BTreeMap::new()), None);
        assert!(session_events().contains(&BITCOIN_ADDRESSES_CHANGED_EVENT.to_string()), "the event is offered when a dapp asks for it");
    }

    #[test]
    fn test_session_rejection_names_the_wire_values() {
        let rejection = session_rejection(GemWalletConnectRejectionReason::UnsupportedChains);
        assert_eq!(rejection.reason, GemWalletConnectRejectionReason::UnsupportedChains);
        assert_eq!(rejection.code, 5100);
        assert_eq!(rejection.message, "Unsupported chains");
        assert!(rejection.deletes_session);

        assert_eq!(session_rejection(GemWalletConnectRejectionReason::UserRejected).code, USER_REJECTED_SESSION_ERROR_CODE);
        assert_eq!(session_rejection(GemWalletConnectRejectionReason::UnsupportedMethods).code, 5101);
        assert_eq!(session_rejection(GemWalletConnectRejectionReason::UnsupportedEvents).code, 5102);
        assert_eq!(session_rejection(GemWalletConnectRejectionReason::UnsupportedAccounts).code, 5103);
    }

    use super::*;
    use gem_wallet_connect::{WCEthereumTransactionData, WCSuiTransactionData};
    use num_bigint::BigUint;
    use primitives::Account;

    use crate::testkit::mock_wc_ethereum_transaction_data;

    #[test]
    fn test_connections_group_by_wallet_in_wallet_order_and_newest_first() {
        let second = Wallet {
            name: "second".to_string(),
            index: 2,
            ..Wallet::mock_with_id(WalletId::Multicoin("second".to_string()), &[Chain::Ethereum])
        };
        let first = Wallet {
            name: "first".to_string(),
            index: 1,
            ..Wallet::mock_with_id(WalletId::Multicoin("first".to_string()), &[Chain::Ethereum])
        };

        let groups = connection_groups(
            [(&second, "old-second", 30), (&first, "old-first", 20), (&second, "new-second", 1), (&first, "new-first", 5)]
                .into_iter()
                .map(|(wallet, id, minutes)| WalletConnection {
                    session: WalletConnectionSession {
                        created_at: Utc::now() - chrono::Duration::minutes(minutes),
                        ..WalletConnectionSession::mock(id, &[Chain::Ethereum])
                    },
                    wallet: wallet.clone(),
                })
                .collect(),
        );

        let titles: Vec<&str> = groups.iter().map(|(wallet, _)| wallet.name.as_str()).collect();
        assert_eq!(titles, vec!["first", "second"], "sections follow the order the wallets are listed in");
        let ids: Vec<Vec<&str>> = groups.iter().map(|(_, connections)| connections.iter().map(|connection| connection.session.id.as_str()).collect()).collect();
        assert_eq!(ids, vec![vec!["new-first", "old-first"], vec!["new-second", "old-second"]], "the newest connection leads each section");
    }

    #[test]
    fn test_authentication_offers_one_account_per_requested_chain_the_wallet_holds() {
        let wallet = Wallet::mock_with_chains(&[Chain::Ethereum, Chain::Solana]);

        let accounts = authentication_accounts(&["eip155:1".to_string(), "eip155:137".to_string(), "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp".to_string(), "eip155:1".to_string()], &wallet);

        assert_eq!(
            accounts.iter().map(|account| account.chain_id.as_str()).collect::<Vec<_>>(),
            vec!["eip155:1"],
            "a repeated chain is asked once, a chain the wallet holds no account for is dropped, and only ethereum chains authenticate"
        );
        assert_eq!(accounts[0].issuer, "did:pkh:eip155:1:address");
        assert!(accounts.iter().all(|account| account.account.chain == Chain::Ethereum));
    }

    #[test]
    fn test_session_account_requires_session_chain_and_account() {
        let connection = WalletConnection {
            session: WalletConnectionSession::mock("topic", &[Chain::Ethereum, Chain::Solana]),
            wallet: Wallet::mock_with_chains(&[Chain::Ethereum]),
        };

        assert_eq!(
            connection_detail_rows(&connection),
            vec![
                GemListRow::Text {
                    title: GemListRowTitle::Wallet,
                    value: connection.wallet.name.clone(),
                },
                GemListRow::Date {
                    title: GemListRowTitle::Date,
                    date: connection.session.created_at,
                },
            ]
        );
        assert_eq!(session_account(&connection, Chain::Ethereum).unwrap().chain, Chain::Ethereum);
        assert!(session_account(&connection, Chain::Solana).is_err());
        assert!(session_account(&connection, Chain::Bitcoin).is_err());
        assert!(validate_session_chain(&connection.session, Chain::Solana).is_ok());
    }

    #[test]
    fn test_switch_chain_adds_a_supported_network_to_a_session_that_already_has_the_ethereum_address() {
        let wallet = Wallet::mock_with_chains(&[Chain::Ethereum, Chain::Blast, Chain::Solana]);
        let connection = |chains: &[Chain]| WalletConnection {
            session: WalletConnectionSession::mock("topic", chains),
            wallet: wallet.clone(),
        };
        let unrecognized = GemWalletConnectOutcome::respond_error(unrecognized_chain_error());

        assert_eq!(
            switch_chain_outcome(&connection(&[Chain::Ethereum]), 81457),
            GemWalletConnectOutcome::AddSessionAccount { account: "eip155:81457:address".to_string() }
        );
        assert_eq!(switch_chain_outcome(&connection(&[Chain::Ethereum, Chain::Blast]), 81457), GemWalletConnectOutcome::respond_null(), "already in the session");
        assert_eq!(switch_chain_outcome(&connection(&[Chain::Ethereum]), 3_735_928_559), unrecognized, "unknown network");
        assert_eq!(switch_chain_outcome(&connection(&[Chain::Ethereum]), 56), unrecognized, "the wallet has no account there");
        assert_eq!(switch_chain_outcome(&connection(&[Chain::Solana]), 81457), unrecognized, "the dapp never got the Ethereum address");
        assert_eq!(
            switch_chain_outcome(
                &WalletConnection {
                    session: WalletConnectionSession::mock("topic", &[Chain::Ethereum]),
                    wallet: Wallet::mock_with_accounts(vec![Account::mock(Chain::Ethereum, "0xshared"), Account::mock(Chain::Blast, "0xother")]),
                },
                81457
            ),
            unrecognized,
            "an address the dapp was never given is not added"
        );

        assert_eq!(add_chain_outcome(&connection(&[Chain::Ethereum]), 81457), GemWalletConnectOutcome::respond_null());
        assert_eq!(add_chain_outcome(&connection(&[Chain::Ethereum]), 3_735_928_559), unrecognized);
    }

    #[test]
    fn test_sessions_sync_rules() {
        let local = vec![
            WalletConnectionSession::mock("active-kept", &[Chain::Ethereum]),
            WalletConnectionSession::mock("active-gone", &[Chain::Ethereum]),
            WalletConnectionSession {
                state: WalletConnectionState::Started,
                ..WalletConnectionSession::mock("started-gone", &[Chain::Ethereum])
            },
        ];
        let remote = vec![WalletConnectionSession::mock("active-kept", &[Chain::Ethereum, Chain::Solana]), WalletConnectionSession::mock("unknown", &[Chain::Ethereum])];

        assert_eq!(sessions_to_delete(&local, &remote), vec!["active-gone".to_string()]);
        let updates = sessions_to_update(&local, remote);
        assert_eq!(updates.iter().map(|session| session.id.as_str()).collect::<Vec<_>>(), vec!["active-kept"]);
        assert_eq!(updates[0].chains, vec![Chain::Ethereum, Chain::Solana]);
        assert!(sessions_to_delete(&local, &[]).contains(&"active-gone".to_string()));
        assert!(!sessions_to_delete(&local, &[]).contains(&"started-gone".to_string()));
    }

    #[test]
    fn test_session_wallets() {
        let multicoin = Wallet {
            name: "multi".to_string(),
            ..Wallet::mock_with_chains(&[Chain::Ethereum, Chain::Solana])
        };
        let single = Wallet {
            name: "single".to_string(),
            ..Wallet::mock_with_type(WalletType::Single, &[Chain::Ethereum])
        };
        let view = Wallet::mock_with_type(WalletType::View, &[Chain::Ethereum]);
        let litecoin_only = Wallet::mock_with_type(WalletType::Single, &[Chain::Litecoin]);
        let wallets = vec![single, view, litecoin_only, multicoin];

        let required = session_wallets(wallets.clone(), &[Chain::Ethereum, Chain::Solana], &[]);
        assert_eq!(required.iter().map(|wallet| wallet.name.as_str()).collect::<Vec<_>>(), vec!["multi"]);

        let optional = session_wallets(wallets.clone(), &[], &[Chain::Ethereum]);
        assert_eq!(optional.iter().map(|wallet| wallet.name.as_str()).collect::<Vec<_>>(), vec!["multi", "single"]);

        let any = session_wallets(wallets, &[], &[]);
        assert_eq!(any.len(), 2, "a wallet with no WalletConnect network is not offered");

        let bitcoin = Wallet {
            name: "bitcoin".to_string(),
            ..Wallet::mock_with_type(WalletType::Single, &[Chain::Bitcoin])
        };
        assert_eq!(session_wallets(vec![bitcoin], &[Chain::Bitcoin], &[]).iter().map(|wallet| wallet.name.as_str()).collect::<Vec<_>>(), vec!["bitcoin"]);
    }

    #[test]
    fn test_default_wallet_prefers_current() {
        let first = Wallet {
            name: "first".to_string(),
            ..Wallet::mock_with_id(WalletId::Multicoin("first".to_string()), &[Chain::Ethereum])
        };
        let second = Wallet {
            name: "second".to_string(),
            ..Wallet::mock_with_id(WalletId::Multicoin("second".to_string()), &[Chain::Ethereum])
        };
        let wallets = vec![first, second.clone()];
        assert_eq!(default_wallet(&wallets, Some(second.id)).map(|wallet| wallet.name), Some("second".to_string()));
        assert_eq!(default_wallet(&wallets, Some(WalletId::Multicoin("other".to_string()))).map(|wallet| wallet.name), Some("first".to_string()));
        assert!(default_wallet(&[], None).is_none());
    }

    #[test]
    fn test_session_chains_keeps_supported_order() {
        let wallet = Wallet::mock_with_chains(&[Chain::Solana, Chain::Ethereum, Chain::Bitcoin]);
        assert_eq!(session_chains(&wallet, &[Chain::Ethereum, Chain::Solana, Chain::Tron]), vec![Chain::Ethereum, Chain::Solana]);
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
            authentication_chain_ids(&["eip155:1".to_string(), "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp".to_string(), "eip155:1".to_string(), "eip155:137".to_string()]),
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
        assert!(!session_methods().contains(&"eth_sendRawTransaction".to_string()));
        assert!(session_events().contains(&"accountsChanged".to_string()));
    }

    #[test]
    fn test_authentication_methods_are_the_evm_session_methods() {
        let methods = authentication_methods();
        assert!(methods.contains(&"personal_sign".to_string()));
        assert!(methods.contains(&"eth_sendTransaction".to_string()));
        assert!(
            !methods
                .iter()
                .any(|method| method.starts_with("solana_") || method.starts_with("sui_") || method.starts_with("ton_") || method.starts_with("tron_"))
        );
        assert!(methods.iter().all(|method| session_methods().contains(method)));
    }

    #[test]
    fn test_validate_transaction_sender_binds_an_evm_request_to_the_session_account() {
        let account = Account::mock(Chain::Ethereum, "0xAbC");
        let matching = WalletConnectTransaction::Ethereum {
            data: WCEthereumTransactionData {
                from: "0xabc".to_string(),
                ..mock_wc_ethereum_transaction_data()
            },
            kind: EvmTransactionKind::Transfer,
        };
        let other = WalletConnectTransaction::Ethereum {
            data: WCEthereumTransactionData {
                from: "0xother".to_string(),
                ..mock_wc_ethereum_transaction_data()
            },
            kind: EvmTransactionKind::Transfer,
        };

        assert!(validate_transaction_sender(&matching, &account).is_ok(), "EVM addresses compare without case");
        assert!(validate_transaction_sender(&other, &account).is_err(), "a dapp cannot simulate for one account and sign with another");
    }

    #[test]
    fn test_validate_transaction_sender_binds_a_tron_request_to_the_session_account() {
        let transaction = WalletConnectTransaction::Tron {
            data: include_str!("../../../../crates/gem_wallet_connect/testdata/tron_send_transaction.json").to_string(),
            output_type: TransferDataOutputType::EncodedTransaction,
        };

        assert!(validate_transaction_sender(&transaction, &Account::mock(Chain::Tron, "TJoSEwEqt7cT3TUwmEoUYnYs5cZR3xSukM")).is_ok());
        assert!(
            validate_transaction_sender(&transaction, &Account::mock(Chain::Tron, "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t")).is_err(),
            "the review reads as a send from the session account, so the transaction must be one"
        );
    }

    #[test]
    fn test_validate_transaction_sender_binds_a_sui_request_to_the_session_account() {
        let session_account = Account::mock(Chain::Sui, "0x93f65b8c16c263343bbf66cf9f8eef69cb1dbc92d13f0c331b0dcaeb76b4aab6");
        let other_account = Account::mock(Chain::Sui, "0xa9bd0493f9bd1f792a4aedc1f99d54535a75a46c38fd56a8f2c6b7c8d75817a1");
        let sui_transaction = |transaction: &str| WalletConnectTransaction::Sui {
            data: WCSuiTransactionData {
                transaction: transaction.to_string(),
                wallet_address: session_account.address.clone(),
            },
            output_type: TransferDataOutputType::EncodedTransaction,
        };
        let bcs = sui_transaction(include_str!("../../../../crates/gem_sui/testdata/wc_sign_transaction_bcs.txt"));
        let json = sui_transaction(include_str!("../../../../crates/gem_sui/testdata/wc_sign_transaction_cetus.json"));

        assert!(validate_transaction_sender(&bcs, &session_account).is_ok());
        assert!(
            validate_transaction_sender(&bcs, &other_account).is_err(),
            "the review reads as a send from the session account, so the transaction must be one"
        );
        assert!(validate_transaction_sender(&json, &other_account).is_ok());
        assert!(validate_transaction_sender(&json, &session_account).is_err());
    }

    #[test]
    fn test_transfer_data_maps_tron_approval() {
        let data = include_str!("../../../../crates/gem_wallet_connect/testdata/tron_send_transaction.json");
        for action in [GemWalletConnectTransactionAction::Sign, GemWalletConnectTransactionAction::Send] {
            let transfer = transfer_data(
                application_metadata("app".into(), String::new(), "https://app.example".into(), vec![]),
                WalletConnectTransaction::Tron {
                    data: data.to_string(),
                    output_type: TransferDataOutputType::EncodedTransaction,
                },
                &Account::mock(Chain::Tron, "sender"),
                action,
            )
            .unwrap();
            assert_eq!(transfer.recipient.address, "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t");
            assert_eq!(transfer.value, BigInt::ZERO);
            let extra = transfer.input_type.get_generic_data().unwrap();
            assert_eq!(extra.transaction_type, TransactionType::TokenApproval);
            assert_eq!(extra.approval, gem_tron::decode_wallet_connect_approval(data).unwrap().map(|decoded| decoded.approval));
            assert_eq!(extra.data.as_deref(), Some(data.as_bytes()));
            assert_eq!(extra.output_type, TransferDataOutputType::EncodedTransaction);
            assert_eq!(
                extra.output_action,
                match action {
                    GemWalletConnectTransactionAction::Sign => TransferDataOutputAction::Sign,
                    GemWalletConnectTransactionAction::Send => TransferDataOutputAction::Send,
                }
            );
        }
    }

    #[test]
    fn test_transfer_data_maps_tron_outflow() {
        let transaction: serde_json::Value = serde_json::from_str(include_str!("../../../../crates/gem_tron/testdata/wallet_connect_transfer_transaction.json")).unwrap();
        let data = serde_json::json!({ "transaction": transaction }).to_string();
        let transfer = transfer_data(
            application_metadata("app".into(), String::new(), "https://app.example".into(), vec![]),
            WalletConnectTransaction::Tron {
                data,
                output_type: TransferDataOutputType::EncodedTransaction,
            },
            &Account::mock(Chain::Tron, "sender"),
            GemWalletConnectTransactionAction::Send,
        )
        .unwrap();

        assert_eq!(transfer.value, BigInt::from(2_000_000));
        assert_eq!(transfer.recipient.address, "THTR75o8xXAgCTQqpiot2AFRAjvW1tSbVV");
        assert_eq!(transfer.input_type.get_generic_data().unwrap().transaction_type, TransactionType::SmartContractCall);
    }

    #[test]
    fn test_transfer_data_maps_solana_native_outflow() {
        let sender = Pubkey::new([1; 32]);
        let recipient = Pubkey::new([5; 32]);
        let transfer = |instruction: gem_solana::Instruction| {
            let mut builder = gem_solana::TransactionBuilder::new(sender, [7; 32]);
            builder.add_instruction(instruction);
            let solana = WalletConnectTransaction::Solana {
                data: gem_wallet_connect::WCSolanaTransactionData {
                    transaction: String::new(),
                    decoded: builder.build().unwrap(),
                },
                output_type: TransferDataOutputType::EncodedTransaction,
            };
            let metadata = application_metadata("app".into(), String::new(), "https://app.example".into(), vec![]);
            transfer_data(metadata, solana, &Account::mock(Chain::Solana, &sender.to_base58()), GemWalletConnectTransactionAction::Send).unwrap()
        };

        let outflow = transfer(gem_solana::system::transfer(&sender, &recipient, 1_000_000_000));
        assert_eq!(outflow.value, BigInt::from(1_000_000_000));
        assert_eq!(outflow.recipient.address, recipient.to_base58());

        let inflow = transfer(gem_solana::system::transfer(&recipient, &sender, 1_000_000_000));
        assert_eq!(inflow.value, BigInt::ZERO, "coins another signer sends to the wallet never count as leaving it");
        assert_eq!(inflow.recipient.address, "");

        let mint = Pubkey::new([3; 32]);
        let token_program = Pubkey::from_base58(gem_solana::TOKEN_PROGRAM).unwrap();
        let token = transfer(gem_solana::token::transfer_checked_with_program_id(&Pubkey::new([2; 32]), &mint, &Pubkey::new([4; 32]), &sender, 19_000_000, 6, &token_program));
        assert_eq!(token.value, BigInt::ZERO, "a token amount is never shown as the native coin");
        assert_eq!(token.recipient.address, "");
    }

    #[test]
    fn test_transfer_data_maps_ton_connect_outflow() {
        let data = include_str!("../../../../crates/gem_ton/testdata/ton_connect_send_transaction.json");
        let metadata = application_metadata("Demo DApp".into(), String::new(), "https://demo.ton-connect.io".into(), vec![]);
        let transaction = WalletConnectTransaction::Ton {
            data: data.to_string(),
            output_type: TransferDataOutputType::EncodedTransaction,
        };

        let transfer = transfer_data(metadata.clone(), transaction, &Account::mock(Chain::Ton, "sender"), GemWalletConnectTransactionAction::Send).unwrap();

        assert_eq!(transfer.value, BigInt::from(5_000_000));
        assert_eq!(transfer.recipient.address, "EQCKWpx7cNMpvmcN5ObM5lLUZHZRFKqYA4xmw9jOry0ZsF9M");
        let extra = transfer.input_type.get_generic_data().unwrap();
        assert_eq!(extra.data.as_deref(), Some(data.as_bytes()));
        assert_eq!(extra.output_action, TransferDataOutputAction::Send);

        let invalid = WalletConnectTransaction::Ton {
            data: r#"{"messages":[{"address":"a","amount":"lots"}]}"#.to_string(),
            output_type: TransferDataOutputType::EncodedTransaction,
        };
        assert!(
            transfer_data(metadata, invalid, &Account::mock(Chain::Ton, "sender"), GemWalletConnectTransactionAction::Send).is_err(),
            "an amount the review cannot show is never sent to the signer"
        );
    }

    #[test]
    fn test_transfer_data_drops_an_empty_evm_gas_limit() {
        let gas_limit = |gas: &str| {
            let transaction = WalletConnectTransaction::Ethereum {
                data: WCEthereumTransactionData {
                    gas: Some(gas.to_string()),
                    ..mock_wc_ethereum_transaction_data()
                },
                kind: EvmTransactionKind::Transfer,
            };
            let transfer = transfer_data(ApplicationMetadata::mock(), transaction, &Account::mock(Chain::Ethereum, "0xfrom"), GemWalletConnectTransactionAction::Send).unwrap();
            let TransactionInputType::Generic { extra, .. } = transfer.input_type else {
                panic!("expected a generic input");
            };
            extra.gas_limit
        };

        assert_eq!(gas_limit("0x30d40"), Some(BigInt::from(200_000)));
        assert_eq!(gas_limit("0x"), None, "an empty gas limit leaves the estimate in place");
        assert_eq!(gas_limit("0x0"), None, "a zero gas limit is never signed");
    }

    #[test]
    fn test_transfer_data_maps_evm_and_encoded_transactions() {
        let metadata = application_metadata("app".into(), String::new(), "https://app.example".into(), vec![]);
        let evm = WalletConnectTransaction::Ethereum {
            data: gem_wallet_connect::WCEthereumTransactionData {
                chain_id: Some(1),
                from: "0xfrom".to_string(),
                to: "0xto".to_string(),
                value: Some(BigUint::from(16u32)),
                gas: Some("0x5208".to_string()),
                gas_limit: None,
                gas_price: None,
                max_fee_per_gas: Some("0x64".to_string()),
                max_priority_fee_per_gas: Some("0x2".to_string()),
                nonce: None,
                data: Some("0xdeadbeef".to_string()),
            },
            kind: EvmTransactionKind::TokenApproval(primitives::swap::ApprovalData::mock()),
        };

        let transfer = transfer_data(metadata.clone(), evm, &Account::mock(Chain::Ethereum, "0xfrom"), GemWalletConnectTransactionAction::Send).unwrap();

        assert_eq!(transfer.value, BigInt::from(16));
        assert_eq!(transfer.recipient.address, "0xto");
        let TransactionInputType::Generic { asset, extra, .. } = transfer.input_type else {
            panic!("expected a generic input");
        };
        assert_eq!(asset.id, primitives::AssetId::from_chain(Chain::Ethereum));
        assert_eq!(extra.gas_limit, Some(BigInt::from(21000)));
        assert!(matches!(extra.gas_price, Some(GasPriceType::Eip1559 { ref gas_price, ref priority_fee }) if *gas_price == BigInt::from(100) && *priority_fee == BigInt::from(2)));
        assert_eq!(extra.data, Some(vec![0xde, 0xad, 0xbe, 0xef]));
        assert_eq!(extra.transaction_type, TransactionType::TokenApproval);
        assert!(extra.approval.is_some());
        assert_eq!(extra.output_action, TransferDataOutputAction::Send);

        let solana = WalletConnectTransaction::Solana {
            data: gem_wallet_connect::WCSolanaTransactionData {
                transaction: "AQID".to_string(),
                decoded: gem_solana::TransactionBuilder::new(Pubkey::new([1; 32]), [7; 32]).build().unwrap(),
            },
            output_type: TransferDataOutputType::Signature,
        };
        let transfer = transfer_data(metadata, solana, &Account::mock(Chain::Solana, "sender"), GemWalletConnectTransactionAction::Sign).unwrap();
        assert_eq!(transfer.value, BigInt::from(0));
        assert_eq!(transfer.recipient.address, "");
        let TransactionInputType::Generic { extra, .. } = transfer.input_type else {
            panic!("expected a generic input");
        };
        assert_eq!(extra.data, Some(b"AQID".to_vec()));
        assert_eq!(extra.output_type, TransferDataOutputType::Signature);
        assert_eq!(extra.output_action, TransferDataOutputAction::Sign);
        assert_eq!(extra.transaction_type, TransactionType::SmartContractCall);
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
    fn test_only_a_cancelled_signature_answers_the_dapp() {
        assert_eq!(signer_failure(GemErrorText::Cancelled), GemSignerFailure::Reject);
        assert_eq!(
            signer_failure(GemErrorText::NetworkOffline),
            GemSignerFailure::Retry { error: GemErrorText::NetworkOffline },
            "a signature that failed on its own leaves the request open to try again"
        );
    }

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

    #[test]
    fn test_a_dapp_is_opened_only_through_its_own_app_or_site() {
        let redirect = |native: Option<&str>, universal: Option<&str>| {
            dapp_redirect(
                Some(&Redirect {
                    native: native.map(str::to_string),
                    universal: universal.map(str::to_string),
                }),
                "https://dapp.example",
            )
        };

        assert_eq!(redirect(Some("kotlin-dapp-wc://request"), Some("https://dapp.example/app")), Some("kotlin-dapp-wc://request".to_string()));
        assert_eq!(redirect(None, Some("https://dapp.example/app")), Some("https://dapp.example/app".to_string()));
        assert_eq!(redirect(None, Some("https://app.dapp.example/")), Some("https://app.dapp.example/".to_string()), "a page on the dapp's own site");
        assert_eq!(
            redirect(Some("not a url"), Some("https://dapp.example/app")),
            Some("https://dapp.example/app".to_string()),
            "an unreadable app link falls back to the site"
        );
        for blocked in [
            "https://other.example/app",
            "https://dapp.example.evil.example/",
            "https://notdapp.example/",
            "gem://wc?uri=wc:other@2",
            "wc:other@2?symKey=abc",
            "https://gemwallet.com/tokens/bitcoin",
            "http://dapp.example",
            "javascript:alert(1)",
            "data:text/html,hi",
            "file:///etc/hosts",
            "intent://scan#Intent;end",
            "tel:+15555550100",
            "sms:+15555550100",
            "mailto:support@dapp.example",
            "itms-services://?action=download-manifest&url=https://dapp.example/app.plist",
            "shortcuts://run-shortcut?name=x",
            "market://details?id=com.example",
            "x-apple-health://",
        ] {
            assert_eq!(redirect(Some(blocked), None), None, "{blocked} must not be opened for a dapp");
        }
        assert_eq!(redirect(None, None), None);
        assert_eq!(dapp_redirect(None, "https://dapp.example"), None);
        assert_eq!(
            dapp_redirect(
                Some(&Redirect {
                    native: None,
                    universal: Some("https://gemwallet.com/wc".to_string()),
                }),
                "https://gemwallet.com",
            ),
            None,
            "a dapp claiming Gem's own site still cannot steer the user into Gem"
        );
    }

    #[test]
    fn test_supported_chains_are_the_configured_chains_that_parse() {
        let configured = crate::config::wallet_connect::get_wallet_connect_config().chains;
        let chains = super::supported_chains();

        assert!(!chains.is_empty());
        assert!(chains.len() <= configured.len());
        assert!(chains.iter().all(|chain| configured.contains(&chain.to_string())));
    }
}
