use std::path::PathBuf;
use std::sync::Arc;

use gem_derivation::{
    derive_account_from_private_key_value, derive_accounts_from_mnemonic, derive_private_key_from_mnemonic, derive_wallet_id_from_account, import_account_from_private_key,
};
use gem_keystore::{FileKeystore, KeystoreError, KeystoreId};
use primitives::{Account, Chain, WalletId, WalletType};
use signer::encode_private_key;
use zeroize::Zeroizing;

use super::types::{GemImportType, GemKeystoreAccount, GemStoredSecretMigration, GemStoredWallet, GemWalletImport};
use crate::GemstoneError;
use crate::models::transaction::GemSignerInput;
use crate::signer::GemChainSigner;

#[derive(uniffi::Object)]
pub struct GemKeystore {
    inner: FileKeystore,
}

#[uniffi::export]
impl GemKeystore {
    #[uniffi::constructor]
    pub fn new(base_dir: String) -> Result<Arc<Self>, GemstoneError> {
        Ok(Arc::new(Self {
            inner: FileKeystore::open(PathBuf::from(base_dir))?,
        }))
    }

    pub fn preview_import(&self, import: GemImportType) -> Result<GemWalletImport, GemstoneError> {
        match import {
            GemImportType::PrivateKey { value, chain } => {
                let value = Zeroizing::new(value);
                let account = derive_account_from_private_key_value(&value, chain)?;
                let wallet_id = derive_wallet_id_from_account(&account, WalletType::PrivateKey)?;
                Ok(GemWalletImport::new(wallet_id, WalletType::PrivateKey, vec![account]))
            }
            GemImportType::MulticoinPhrase { words, chains } => {
                let (wallet_id, accounts, _phrase) = derive_mnemonic_wallet(words, chains, WalletType::Multicoin, Chain::Ethereum)?;
                Ok(GemWalletImport::new(wallet_id, WalletType::Multicoin, accounts))
            }
            GemImportType::SinglePhrase { words, chain } => {
                let (wallet_id, accounts, _phrase) = derive_mnemonic_wallet(words, vec![chain], WalletType::Single, chain)?;
                Ok(GemWalletImport::new(wallet_id, WalletType::Single, accounts))
            }
        }
    }

    pub fn create_store(&self, import: GemImportType, password: Vec<u8>) -> Result<GemStoredWallet, GemstoneError> {
        let password = Zeroizing::new(password);
        match import {
            GemImportType::PrivateKey { value, chain } => {
                let value = Zeroizing::new(value);
                let imported = import_account_from_private_key(&value, chain)?;
                let wallet_id = derive_wallet_id_from_account(&imported.account, WalletType::PrivateKey)?;
                let meta = self
                    .inner
                    .import_private_key(&imported.private_key, &password, Some(keystore_id_for_wallet(wallet_id.to_string())))?;
                Ok(GemStoredWallet::new(wallet_id, WalletType::PrivateKey, meta.keystore_id, vec![imported.account]))
            }
            GemImportType::MulticoinPhrase { words, chains } => {
                let (wallet_id, accounts, phrase) = derive_mnemonic_wallet(words, chains, WalletType::Multicoin, Chain::Ethereum)?;
                let meta = self.inner.import_mnemonic(&phrase, &password, Some(keystore_id_for_wallet(wallet_id.to_string())))?;
                Ok(GemStoredWallet::new(wallet_id, WalletType::Multicoin, meta.keystore_id, accounts))
            }
            GemImportType::SinglePhrase { words, chain } => {
                let (wallet_id, accounts, phrase) = derive_mnemonic_wallet(words, vec![chain], WalletType::Single, chain)?;
                let meta = self.inner.import_mnemonic(&phrase, &password, Some(keystore_id_for_wallet(wallet_id.to_string())))?;
                Ok(GemStoredWallet::new(wallet_id, WalletType::Single, meta.keystore_id, accounts))
            }
        }
    }

    pub fn add_accounts(&self, keystore_id: String, password: Vec<u8>, chains: Vec<Chain>) -> Result<Vec<GemKeystoreAccount>, GemstoneError> {
        let password = Zeroizing::new(password);
        let phrase = match self.inner.decrypt_mnemonic_with_meta(&keystore_id, &password) {
            Ok((_meta, phrase)) => phrase,
            Err(KeystoreError::CorruptFile(message)) if message == "stored secret is not a mnemonic" => {
                return Err(GemstoneError::from("add_accounts does not support private-key wallets"));
            }
            Err(error) => return Err(error.into()),
        };
        Ok(derive_accounts_from_mnemonic(&phrase, chains)?.into_iter().map(GemKeystoreAccount::from).collect())
    }

    pub fn export_recovery_phrase(&self, keystore_id: String, password: Vec<u8>) -> Result<Vec<String>, GemstoneError> {
        let password = Zeroizing::new(password);
        Ok(self
            .inner
            .decrypt_mnemonic(&keystore_id, &password)?
            .split_whitespace()
            .map(|word| word.to_string())
            .collect())
    }

    pub fn export_private_key(&self, keystore_id: String, chain: Chain, password: Vec<u8>) -> Result<String, GemstoneError> {
        let password = Zeroizing::new(password);
        let private_key = self.load_private_key(&keystore_id, chain, &password)?;
        Ok(encode_private_key(&chain, &private_key)?)
    }

    pub fn migrate_v3(&self, v3_path: String, v3_password: Vec<u8>, new_password: Vec<u8>, keystore_id: String) -> Result<GemStoredSecretMigration, GemstoneError> {
        let v3_password = Zeroizing::new(v3_password);
        let new_password = Zeroizing::new(new_password);
        let meta = self.inner.import_v3(&PathBuf::from(v3_path), &v3_password, &new_password, Some(keystore_id))?;
        Ok(GemStoredSecretMigration {
            keystore_id: meta.keystore_id,
            kind: meta.kind,
        })
    }

    pub fn delete(&self, keystore_id: String) -> Result<bool, GemstoneError> {
        Ok(self.inner.delete(&keystore_id)?)
    }

    pub fn sign(&self, keystore_id: String, chain: Chain, input: GemSignerInput, password: Vec<u8>) -> Result<Vec<String>, GemstoneError> {
        GemChainSigner::new(chain).sign_input(input, self.signing_key(&keystore_id, chain, password)?)
    }

    pub fn sign_auth(&self, keystore_id: String, chain: Chain, hash: Vec<u8>, password: Vec<u8>) -> Result<String, GemstoneError> {
        crate::auth::sign_auth_message_hash(hash, self.signing_key(&keystore_id, chain, password)?)
    }
}

#[cfg(any(test, debug_assertions))]
#[uniffi::export]
impl GemKeystore {
    pub fn private_key(&self, keystore_id: String, chain: Chain, password: Vec<u8>) -> Result<Vec<u8>, GemstoneError> {
        let password = Zeroizing::new(password);
        Ok(self.load_private_key(&keystore_id, chain, &password)?.to_vec())
    }
}

#[uniffi::export]
pub fn keystore_id_for_wallet(wallet_id: String) -> String {
    KeystoreId::from_wallet_id(&wallet_id).into_string()
}

impl GemKeystore {
    pub(crate) fn signing_key(&self, keystore_id: &str, chain: Chain, password: Vec<u8>) -> Result<Zeroizing<Vec<u8>>, GemstoneError> {
        let password = Zeroizing::new(password);
        self.load_private_key(keystore_id, chain, &password)
    }

    fn load_private_key(&self, keystore_id: &str, chain: Chain, password: &[u8]) -> Result<Zeroizing<Vec<u8>>, GemstoneError> {
        match self.inner.decrypt_mnemonic(keystore_id, password) {
            Ok(phrase) => Ok(derive_private_key_from_mnemonic(&phrase, chain)?),
            Err(KeystoreError::CorruptFile(message)) if message == "stored secret is not a mnemonic" => Ok(self.inner.decrypt_private_key(keystore_id, password)?),
            Err(error) => Err(error.into()),
        }
    }
}

fn derive_mnemonic_wallet(
    words: Vec<String>,
    requested_chains: Vec<Chain>,
    wallet_type: WalletType,
    wallet_id_chain: Chain,
) -> Result<(WalletId, Vec<Account>, Zeroizing<String>), GemstoneError> {
    if requested_chains.is_empty() {
        return Err(gem_derivation::AccountDerivationError::invalid_input("mnemonic derivation requires at least one chain").into());
    }

    let words = Zeroizing::new(words);
    let phrase = Zeroizing::new(words.join(" "));
    let mut chains = requested_chains.clone();
    if !chains.contains(&wallet_id_chain) {
        chains.push(wallet_id_chain);
    }

    let derived_accounts = derive_accounts_from_mnemonic(&phrase, chains)?;
    let wallet_id_account = derived_accounts
        .iter()
        .find(|account| account.chain == wallet_id_chain)
        .ok_or_else(|| gem_derivation::AccountDerivationError::unsupported("wallet id account derivation returned no account"))?;
    let wallet_id = derive_wallet_id_from_account(wallet_id_account, wallet_type)?;
    let accounts = derived_accounts.into_iter().filter(|account| requested_chains.contains(&account.chain)).collect();

    Ok((wallet_id, accounts, phrase))
}

#[cfg(test)]
mod migration_tests {
    use std::path::PathBuf;

    use primitives::{Chain, hex};

    use super::GemKeystore;

    const V3_MNEMONIC: &str = include_str!("../../../crates/gem_keystore/testdata/v3_ios_mnemonic.json");
    const V3_PRIVATE_KEY: &str = include_str!("../../../crates/gem_keystore/testdata/v3_ios_private_key.json");
    const V3_PASSWORD: &[u8] = b"000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";
    const NEW_PASSWORD: &[u8] = b"raw-v4-password-bytes";
    const KEYSTORE_ID: &str = "70b3b599-4bf1-4f7a-9ad4-a7746cc38ab3";
    const EXPECTED_PHRASE: &str =
        "dignity possible oppose wolf early kingdom essay arctic ten fence prepare mango source federal chief south dynamic rebuild wear envelope bulb picnic own scorpion";
    const EXPECTED_PRIVATE_KEY: &str = "ae8794f84919b14ff9d1f0f7cf490a4c04e608de16864f53fe8b40af127b9da3";
    const EXPECTED_ETHEREUM_ADDRESS: &str = "0x5a8f70b44aFa00Cb70615D9c9CCb9A24933ED2D3";

    fn prepare(name: &str, fixture: &str) -> (PathBuf, String) {
        let base = std::env::temp_dir().join(format!("gemstone_migration_{name}"));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).unwrap();
        let v3_path = base.join("legacy.json");
        std::fs::write(&v3_path, fixture).unwrap();
        (base, v3_path.to_string_lossy().into_owned())
    }

    #[test]
    fn migrate_v3_mnemonic_round_trip_and_idempotent() {
        let (base, v3_path) = prepare("mnemonic", V3_MNEMONIC);
        let keystore = GemKeystore::new(base.to_string_lossy().into_owned()).unwrap();

        let migration = keystore
            .migrate_v3(v3_path.clone(), V3_PASSWORD.to_vec(), NEW_PASSWORD.to_vec(), KEYSTORE_ID.to_string())
            .unwrap();
        assert_eq!(migration.keystore_id, KEYSTORE_ID);

        assert_eq!(
            keystore.export_recovery_phrase(KEYSTORE_ID.to_string(), NEW_PASSWORD.to_vec()).unwrap().join(" "),
            EXPECTED_PHRASE
        );
        let accounts = keystore.add_accounts(KEYSTORE_ID.to_string(), NEW_PASSWORD.to_vec(), vec![Chain::Ethereum]).unwrap();
        assert_eq!(accounts[0].address, EXPECTED_ETHEREUM_ADDRESS);

        let again = keystore.migrate_v3(v3_path, V3_PASSWORD.to_vec(), NEW_PASSWORD.to_vec(), KEYSTORE_ID.to_string()).unwrap();
        assert_eq!(again.keystore_id, KEYSTORE_ID);
        assert_eq!(
            keystore.export_recovery_phrase(KEYSTORE_ID.to_string(), NEW_PASSWORD.to_vec()).unwrap().join(" "),
            EXPECTED_PHRASE
        );

        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn migrate_v3_private_key_round_trip_and_idempotent() {
        let (base, v3_path) = prepare("private_key", V3_PRIVATE_KEY);
        let keystore = GemKeystore::new(base.to_string_lossy().into_owned()).unwrap();

        let migration = keystore
            .migrate_v3(v3_path.clone(), V3_PASSWORD.to_vec(), NEW_PASSWORD.to_vec(), KEYSTORE_ID.to_string())
            .unwrap();
        assert_eq!(migration.keystore_id, KEYSTORE_ID);
        assert_eq!(
            hex::encode(keystore.private_key(KEYSTORE_ID.to_string(), Chain::Ethereum, NEW_PASSWORD.to_vec()).unwrap()),
            EXPECTED_PRIVATE_KEY
        );
        let account = keystore
            .preview_import(super::GemImportType::PrivateKey {
                value: EXPECTED_PRIVATE_KEY.to_string(),
                chain: Chain::Ethereum,
            })
            .unwrap();
        assert_eq!(account.accounts[0].address, EXPECTED_ETHEREUM_ADDRESS);

        // idempotent re-run
        let again = keystore.migrate_v3(v3_path, V3_PASSWORD.to_vec(), NEW_PASSWORD.to_vec(), KEYSTORE_ID.to_string()).unwrap();
        assert_eq!(again.keystore_id, KEYSTORE_ID);
        assert_eq!(
            hex::encode(keystore.private_key(KEYSTORE_ID.to_string(), Chain::Ethereum, NEW_PASSWORD.to_vec()).unwrap()),
            EXPECTED_PRIVATE_KEY
        );

        let _ = std::fs::remove_dir_all(&base);
    }
}
