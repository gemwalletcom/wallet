use zeroize::Zeroizing;

use crate::KeystoreError;

use super::types::{KeystoreEntryError, StoredSecretMeta};

pub trait Keystore {
    fn import_mnemonic(&self, phrase: &str, password: &[u8], keystore_id: Option<String>) -> Result<StoredSecretMeta, KeystoreError>;
    fn import_private_key(&self, private_key: &[u8], password: &[u8], keystore_id: Option<String>) -> Result<StoredSecretMeta, KeystoreError>;
    fn decrypt_mnemonic(&self, keystore_id: &str, password: &[u8]) -> Result<Zeroizing<String>, KeystoreError>;
    fn decrypt_private_key(&self, keystore_id: &str, password: &[u8]) -> Result<Zeroizing<Vec<u8>>, KeystoreError>;
    fn change_password(&self, keystore_id: &str, old_password: &[u8], new_password: &[u8]) -> Result<StoredSecretMeta, KeystoreError>;
    fn verify(&self, keystore_id: &str, password: &[u8]) -> Result<StoredSecretMeta, KeystoreError>;
    fn get_meta(&self, keystore_id: &str) -> Result<Option<StoredSecretMeta>, KeystoreError>;
    fn list(&self) -> Result<Vec<Result<StoredSecretMeta, KeystoreEntryError>>, KeystoreError>;
    fn delete(&self, keystore_id: &str) -> Result<bool, KeystoreError>;
}
