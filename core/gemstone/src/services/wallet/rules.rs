use gem_keystore::Mnemonic;
use primitives::{Account, Chain, Wallet, WalletId, WalletSource, WalletType};

use super::error::GemWalletImportError;
use super::model::GemWalletImportType;
use crate::address::{checksum_address, validate_address};
use crate::keystore::GemKeystoreAccount;
use crate::signer::decode_private_key;

#[uniffi::export]
impl GemWalletImportType {
    pub fn validated(self) -> Result<Self, GemWalletImportError> {
        match self {
            Self::MulticoinPhrase { words, chains } => Ok(Self::MulticoinPhrase {
                words: validated_words(words)?,
                chains,
            }),
            Self::SinglePhrase { words, chain } => Ok(Self::SinglePhrase {
                words: validated_words(words)?,
                chain,
            }),
            Self::PrivateKey { value, chain } => {
                let value = value.trim().to_string();
                decode_private_key(chain, value.clone()).map_err(|_| GemWalletImportError::InvalidPrivateKey)?;
                Ok(Self::PrivateKey { value, chain })
            }
            Self::Address { address, chain } => {
                let address = checksum_address(address.trim(), chain);
                if !validate_address(&address, chain) {
                    return Err(GemWalletImportError::InvalidAddress);
                }
                Ok(Self::Address { address, chain })
            }
        }
    }
}

fn validated_words(words: Vec<String>) -> Result<Vec<String>, GemWalletImportError> {
    let words: Vec<String> = words.iter().flat_map(|word| word.split_whitespace()).map(|word| word.to_lowercase()).collect();
    let phrase = words.join(" ");
    let invalid = Mnemonic::invalid_words(&phrase);
    if !invalid.is_empty() {
        return Err(GemWalletImportError::InvalidSecretPhraseWords { words: invalid });
    }
    if !Mnemonic::is_valid(&phrase) {
        return Err(GemWalletImportError::InvalidSecretPhrase);
    }
    Ok(words)
}

pub fn view_wallet(name: String, chain: Chain, address: String) -> Wallet {
    Wallet {
        id: WalletId::View(chain, address.clone()),
        external_id: None,
        name,
        index: 0,
        wallet_type: WalletType::View,
        accounts: vec![Account {
            chain,
            address,
            derivation_path: String::new(),
            extended_public_key: Some(String::new()),
        }],
        is_pinned: false,
        image_url: None,
        source: WalletSource::Import,
    }
}

pub fn account(account: GemKeystoreAccount) -> Account {
    Account {
        chain: account.chain,
        address: account.address,
        derivation_path: account.derivation_path,
        extended_public_key: Some(account.public_key.unwrap_or_default()),
    }
}

pub fn next_wallet_index(wallets: &[Wallet]) -> i32 {
    wallets.iter().map(|wallet| wallet.index).max().map(|index| index + 1).unwrap_or(1)
}

pub fn missing_chains(wallet: &Wallet, chains: &[Chain]) -> Vec<Chain> {
    chains.iter().copied().filter(|chain| wallet.account(*chain).is_none()).collect()
}

pub fn wallets_missing_chains(wallets: Vec<Wallet>, chains: &[Chain]) -> Vec<(Wallet, Vec<Chain>)> {
    wallets
        .into_iter()
        .filter(|wallet| wallet.wallet_type == WalletType::Multicoin)
        .filter_map(|wallet| {
            let missing = missing_chains(&wallet, chains);
            (!missing.is_empty()).then_some((wallet, missing))
        })
        .collect()
}

pub fn sorted_wallets(wallets: Vec<Wallet>) -> Vec<Wallet> {
    let mut sorted = wallets;
    sorted.sort_by_key(|wallet| (wallet.wallet_type.rank(), wallet.index));
    sorted
}

pub fn show_collections(wallet: &Wallet) -> bool {
    match wallet.wallet_type {
        WalletType::Multicoin => true,
        WalletType::Single | WalletType::PrivateKey | WalletType::View => wallet.accounts.first().is_some_and(|account| account.chain.is_nft_supported()),
    }
}

pub fn display_account(wallet: &Wallet) -> Option<Account> {
    match wallet.wallet_type {
        WalletType::Multicoin => wallet.account(Chain::Ethereum).cloned().or_else(|| wallet.accounts.first().cloned()),
        _ => wallet.accounts.first().cloned(),
    }
}

pub fn next_current_wallet(wallets: &[Wallet]) -> Option<WalletId> {
    wallets
        .iter()
        .min_by_key(|wallet| (wallet.wallet_type.rank(), wallet.index))
        .map(|wallet| wallet.id.clone())
}

pub fn legacy_keystore_id(wallet: &Wallet) -> String {
    wallet.external_id.clone().unwrap_or_else(|| wallet.id.id())
}

pub fn existing_wallet(wallets: &[Wallet], wallet_id: &WalletId, wallet_type: WalletType) -> Option<Wallet> {
    wallets.iter().find(|wallet| wallet.id == *wallet_id && wallet.wallet_type == wallet_type).cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    const PHRASE: &str = "test test test test test test test test test test test junk";

    #[test]
    fn test_validate_import_phrase() {
        let words = |phrase: &str| phrase.split(' ').map(String::from).collect::<Vec<_>>();
        let validated = GemWalletImportType::SinglePhrase {
            words: vec![format!(" {PHRASE} ")],
            chain: Chain::Ethereum,
        }
        .validated()
        .unwrap();
        assert!(matches!(validated, GemWalletImportType::SinglePhrase { words: validated, .. } if validated == words(PHRASE)));

        assert_eq!(
            GemWalletImportType::MulticoinPhrase {
                words: words("test test test test test test test test test test test nope"),
                chains: vec![Chain::Ethereum],
            }
            .validated()
            .unwrap_err(),
            GemWalletImportError::InvalidSecretPhraseWords { words: vec!["nope".into()] }
        );
        assert_eq!(
            GemWalletImportType::MulticoinPhrase {
                words: words("test test test test test test test test test test test test"),
                chains: vec![Chain::Ethereum],
            }
            .validated()
            .unwrap_err(),
            GemWalletImportError::InvalidSecretPhrase
        );
    }

    #[test]
    fn test_validate_import_address_and_private_key() {
        let address = "0xd8da6bf26964af9d7eed9e03e53415d37aa96045";
        let validated = GemWalletImportType::Address {
            address: format!(" {address} "),
            chain: Chain::Ethereum,
        }
        .validated()
        .unwrap();
        assert!(matches!(validated, GemWalletImportType::Address { address, .. } if address == "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"));
        assert_eq!(
            GemWalletImportType::Address {
                address: "not an address".into(),
                chain: Chain::Ethereum,
            }
            .validated()
            .unwrap_err(),
            GemWalletImportError::InvalidAddress
        );
        assert_eq!(
            GemWalletImportType::PrivateKey {
                value: "zz".into(),
                chain: Chain::Ethereum,
            }
            .validated()
            .unwrap_err(),
            GemWalletImportError::InvalidPrivateKey
        );
        assert!(
            GemWalletImportType::PrivateKey {
                value: "0x4c0883a69102937d6231471b5dbb6204fe5129617082792ae468d01a3f362318".into(),
                chain: Chain::Ethereum,
            }
            .validated()
            .is_ok()
        );
    }

    fn wallet(id: WalletId, wallet_type: WalletType, chains: &[Chain]) -> Wallet {
        Wallet {
            id,
            wallet_type,
            ..Wallet::mock_with_accounts(Account::mock_chains(chains, "address"))
        }
    }

    #[test]
    fn test_wallets_missing_chains() {
        let multicoin = wallet(WalletId::Multicoin("0x1".to_string()), WalletType::Multicoin, &[Chain::Ethereum]);
        let complete = wallet(WalletId::Multicoin("0x2".to_string()), WalletType::Multicoin, &[Chain::Ethereum, Chain::Bitcoin]);
        let single = wallet(WalletId::Single(Chain::Ethereum, "0x3".to_string()), WalletType::Single, &[Chain::Ethereum]);

        let result = wallets_missing_chains(vec![multicoin.clone(), complete, single], &[Chain::Ethereum, Chain::Bitcoin]);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0.id, multicoin.id);
        assert_eq!(result[0].1, vec![Chain::Bitcoin]);
    }

    #[test]
    fn test_existing_wallet_requires_same_type() {
        let id = WalletId::Single(Chain::Bitcoin, "bc1".to_string());
        let wallets = vec![wallet(id.clone(), WalletType::Single, &[Chain::Bitcoin])];

        assert!(existing_wallet(&wallets, &id, WalletType::Single).is_some());
        assert!(existing_wallet(&wallets, &id, WalletType::PrivateKey).is_none());
        assert!(existing_wallet(&wallets, &WalletId::Multicoin("0x".to_string()), WalletType::Multicoin).is_none());
    }

    #[test]
    fn test_next_current_wallet_prefers_multicoin_then_lowest_index() {
        let mut view = wallet(WalletId::View(Chain::Ethereum, "0xv".to_string()), WalletType::View, &[Chain::Ethereum]);
        view.index = 0;
        let mut second = wallet(WalletId::Multicoin("0x2".to_string()), WalletType::Multicoin, &[Chain::Ethereum]);
        second.index = 2;
        let mut first = wallet(WalletId::Multicoin("0x1".to_string()), WalletType::Multicoin, &[Chain::Ethereum]);
        first.index = 1;

        assert_eq!(next_current_wallet(&[view, second, first.clone()]), Some(first.id));
        assert_eq!(next_current_wallet(&[]), None);
    }

    #[test]
    fn test_show_collections_follows_the_first_account_chain_outside_multicoin() {
        assert!(show_collections(&wallet(WalletId::Multicoin("0x1".to_string()), WalletType::Multicoin, &[Chain::Bitcoin])));
        assert!(show_collections(&wallet(
            WalletId::Single(Chain::Ethereum, "0x2".to_string()),
            WalletType::Single,
            &[Chain::Ethereum]
        )));
        assert!(!show_collections(&wallet(
            WalletId::Single(Chain::Bitcoin, "0x3".to_string()),
            WalletType::Single,
            &[Chain::Bitcoin]
        )));
        assert!(!show_collections(&wallet(WalletId::View(Chain::Ethereum, "0x4".to_string()), WalletType::View, &[])));
    }

    #[test]
    fn test_view_wallet() {
        let result = view_wallet("Watch".to_string(), Chain::Ethereum, "0xabc".to_string());
        assert_eq!(result.id, WalletId::View(Chain::Ethereum, "0xabc".to_string()));
        assert_eq!(result.wallet_type, WalletType::View);
        assert_eq!(result.accounts.len(), 1);
        assert_eq!(result.accounts[0].address, "0xabc");
    }

    #[test]
    fn test_next_wallet_index_uses_highest_index() {
        assert_eq!(next_wallet_index(&[]), 1);
        let mut first = wallet(WalletId::Multicoin("1".into()), WalletType::Multicoin, &[]);
        first.index = 3;
        let mut second = wallet(WalletId::Multicoin("2".into()), WalletType::Single, &[]);
        second.index = 1;
        assert_eq!(next_wallet_index(&[first, second]), 4);
    }

    #[test]
    fn test_wallets_sort_by_type_then_index() {
        let mut watch = wallet(WalletId::View(Chain::Ethereum, "0xv".to_string()), WalletType::View, &[Chain::Ethereum]);
        watch.index = 0;
        let mut second = wallet(WalletId::Multicoin("0x2".to_string()), WalletType::Multicoin, &[Chain::Ethereum]);
        second.index = 2;
        let mut first = wallet(WalletId::Multicoin("0x1".to_string()), WalletType::Multicoin, &[Chain::Ethereum]);
        first.index = 1;

        let sorted = sorted_wallets(vec![watch.clone(), second.clone(), first.clone()]);

        assert_eq!(sorted.iter().map(|wallet| wallet.id.clone()).collect::<Vec<_>>(), vec![first.id, second.id, watch.id]);
    }

    #[test]
    fn test_multicoin_wallets_display_their_ethereum_account() {
        let multicoin = wallet(WalletId::Multicoin("0x1".to_string()), WalletType::Multicoin, &[Chain::Bitcoin, Chain::Ethereum]);
        let single = wallet(WalletId::Multicoin("0x2".to_string()), WalletType::Single, &[Chain::Bitcoin]);

        assert_eq!(display_account(&multicoin).map(|account| account.chain), Some(Chain::Ethereum));
        assert_eq!(display_account(&single).map(|account| account.chain), Some(Chain::Bitcoin));
    }
}
