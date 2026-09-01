use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use zeroize::Zeroizing;

#[cfg(feature = "v3")]
use crate::v3::{ReaderV3, SecretV3};

use crate::{KeystoreError, KeystoreId};

use super::{
    constants::{FILE_EXTENSION, WHOLE_FILE_CAP},
    file_io::{new_secret_file_options, read_capped, set_owner_read_write, sync_directory},
    format::{meta_from_header, parse_v4, validate_v4_password},
    keystore::Keystore,
    queue,
    secret::{decrypt_secret, encrypt_secret},
    types::{FileKeystore, KdfParams, KeystoreEntryError, KeystoreInspection, ParsedFile, SecretPayload, StoredSecretMeta},
};

impl FileKeystore {
    pub fn open(base_dir: PathBuf) -> Result<Self, KeystoreError> {
        let keystore = Self {
            base_dir,
            default_kdf: KdfParams::default_argon2id()?,
        };
        fs::create_dir_all(&keystore.base_dir)?;
        Ok(keystore)
    }

    pub fn inspect_path(path: &Path) -> Result<KeystoreInspection, KeystoreError> {
        let _queue = queue::lock()?;
        let bytes = read_capped(path, WHOLE_FILE_CAP)?;
        let parsed = parse_v4(&bytes)?;
        Ok(KeystoreInspection {
            meta: Some(meta_from_header(&parsed.header)),
            authenticated: false,
            file_len: bytes.len() as u64,
            ciphertext_len: parsed.ciphertext.len() as u64,
            tag_len: parsed.header.cipher.tag_len(),
            warnings: Vec::new(),
        })
    }

    pub fn verify_path(path: &Path, password: &[u8]) -> Result<StoredSecretMeta, KeystoreError> {
        let _queue = queue::lock()?;
        let bytes = read_capped(path, WHOLE_FILE_CAP)?;
        let parsed = parse_v4(&bytes)?;
        let meta = meta_from_header(&parsed.header);
        let _payload = decrypt_secret(parsed, None, password)?;
        Ok(meta)
    }

    #[cfg(feature = "v3")]
    pub fn import_v3(&self, v3_path: &Path, v3_password: &[u8], new_password: &[u8], keystore_id: Option<String>) -> Result<StoredSecretMeta, KeystoreError> {
        let _queue = queue::lock()?;
        // Idempotent retry: authenticate an existing staged v4 file by id+password; replace it only when corrupt.
        if let Some(parsed_id) = keystore_id
            .as_deref()
            .and_then(|id| KeystoreId::parse(id).ok())
            .filter(|parsed_id| self.path_for_id(parsed_id).exists())
        {
            match self.verify_unlocked(parsed_id.as_str(), new_password) {
                Ok(meta) => return Ok(meta),
                Err(KeystoreError::CorruptFile(_)) => fs::remove_file(self.path_for_id(&parsed_id))?,
                Err(error) => return Err(error),
            }
        }
        let secret = ReaderV3::decrypt_path(v3_path, v3_password)?;
        match &secret {
            SecretV3::Mnemonic(phrase) => self.import_mnemonic_unlocked(phrase, new_password, keystore_id),
            SecretV3::PrivateKey(private_key) => self.import_private_key_unlocked(private_key, new_password, keystore_id),
        }
    }

    #[cfg(feature = "v3")]
    pub fn delete_v3(&self, legacy_id: &str) -> Result<bool, KeystoreError> {
        if legacy_id.is_empty() {
            return Err(KeystoreError::invalid_input("legacy keystore id"));
        }
        let _queue = queue::lock()?;
        let legacy_id = legacy_id.to_lowercase();
        let mut deleted = false;
        for entry in fs::read_dir(&self.base_dir)? {
            let path = entry?.path();
            if !path.is_file() || !is_v3_file(&path, &legacy_id) {
                continue;
            }
            fs::remove_file(&path)?;
            deleted = true;
        }
        if deleted {
            sync_directory(&self.base_dir)?;
        }
        Ok(deleted)
    }

    fn import_mnemonic_unlocked(&self, phrase: &str, password: &[u8], keystore_id: Option<String>) -> Result<StoredSecretMeta, KeystoreError> {
        self.import_payload_unlocked(SecretPayload::mnemonic(phrase)?, password, keystore_id)
    }

    fn import_private_key_unlocked(&self, private_key: &[u8], password: &[u8], keystore_id: Option<String>) -> Result<StoredSecretMeta, KeystoreError> {
        self.import_payload_unlocked(SecretPayload::private_key(private_key)?, password, keystore_id)
    }

    fn import_payload_unlocked(&self, payload: SecretPayload, password: &[u8], keystore_id: Option<String>) -> Result<StoredSecretMeta, KeystoreError> {
        let id = match keystore_id {
            Some(keystore_id) => KeystoreId::parse(&keystore_id)?,
            None => KeystoreId::new(),
        };
        if self.path_for_id(&id).exists() {
            return self.verify_unlocked(id.as_str(), password);
        }
        let body = encrypt_secret(&self.default_kdf, payload, password, &id)?;
        self.write_new_file(&id, &body, false)?;
        self.get_meta_unlocked(id.as_str())?.ok_or(KeystoreError::NotFound)
    }

    fn get_meta_unlocked(&self, keystore_id: &str) -> Result<Option<StoredSecretMeta>, KeystoreError> {
        let id = KeystoreId::parse(keystore_id)?;
        let path = self.path_for_id(&id);
        if !path.exists() {
            return Ok(None);
        }
        let bytes = read_capped(&path, WHOLE_FILE_CAP)?;
        Ok(Some(parsed_meta(&bytes, &id)?))
    }

    fn verify_unlocked(&self, keystore_id: &str, password: &[u8]) -> Result<StoredSecretMeta, KeystoreError> {
        let id = KeystoreId::parse(keystore_id)?;
        let parsed = self.read_parsed_by_id(&id)?;
        let meta = meta_from_header(&parsed.header);
        let _payload = decrypt_secret(parsed, Some(&id), password)?;
        Ok(meta)
    }

    fn decrypt_payload_unlocked(&self, keystore_id: &str, password: &[u8]) -> Result<SecretPayload, KeystoreError> {
        let id = KeystoreId::parse(keystore_id)?;
        let parsed = self.read_parsed_by_id(&id)?;
        decrypt_secret(parsed, Some(&id), password)
    }

    fn read_parsed_by_id(&self, id: &KeystoreId) -> Result<ParsedFile, KeystoreError> {
        let path = self.path_for_id(id);
        let bytes = read_capped(&path, WHOLE_FILE_CAP)?;
        parse_v4(&bytes)
    }

    fn write_new_file(&self, id: &KeystoreId, bytes: &[u8], replace: bool) -> Result<(), KeystoreError> {
        fs::create_dir_all(&self.base_dir)?;
        let path = self.path_for_id(id);
        if !replace && path.exists() {
            return Err(KeystoreError::AlreadyExists);
        }
        let temp_path = self.base_dir.join(format!("{}.{FILE_EXTENSION}.tmp.{}", id.as_str(), KeystoreId::new()));
        let options = new_secret_file_options()?;
        let write_result = (|| -> Result<(), KeystoreError> {
            let mut file = options.open(&temp_path)?;
            set_owner_read_write(&temp_path)?;
            file.write_all(bytes)?;
            file.sync_all()?;
            fs::rename(&temp_path, &path)?;
            sync_directory(&self.base_dir)?;
            Ok(())
        })();
        if write_result.is_err() {
            let _ = fs::remove_file(&temp_path);
        }
        write_result
    }

    fn path_for_id(&self, id: &KeystoreId) -> PathBuf {
        self.base_dir.join(format!("{}.{FILE_EXTENSION}", id.as_str()))
    }
}

impl Keystore for FileKeystore {
    fn import_mnemonic(&self, phrase: &str, password: &[u8], keystore_id: Option<String>) -> Result<StoredSecretMeta, KeystoreError> {
        let _queue = queue::lock()?;
        self.import_mnemonic_unlocked(phrase, password, keystore_id)
    }

    fn import_private_key(&self, private_key: &[u8], password: &[u8], keystore_id: Option<String>) -> Result<StoredSecretMeta, KeystoreError> {
        let _queue = queue::lock()?;
        self.import_private_key_unlocked(private_key, password, keystore_id)
    }

    fn decrypt_mnemonic(&self, keystore_id: &str, password: &[u8]) -> Result<Zeroizing<String>, KeystoreError> {
        let _queue = queue::lock()?;
        self.decrypt_payload_unlocked(keystore_id, password)?.into_mnemonic()
    }

    fn decrypt_private_key(&self, keystore_id: &str, password: &[u8]) -> Result<Zeroizing<Vec<u8>>, KeystoreError> {
        let _queue = queue::lock()?;
        self.decrypt_payload_unlocked(keystore_id, password)?.into_private_key()
    }

    fn change_password(&self, keystore_id: &str, old_password: &[u8], new_password: &[u8]) -> Result<StoredSecretMeta, KeystoreError> {
        let _queue = queue::lock()?;
        validate_v4_password(new_password)?;
        let id = KeystoreId::parse(keystore_id)?;
        let parsed = self.read_parsed_by_id(&id)?;
        let meta = meta_from_header(&parsed.header);
        let payload = decrypt_secret(parsed, Some(&id), old_password)?;
        let body = encrypt_secret(&self.default_kdf, payload, new_password, &id)?;
        self.write_new_file(&id, &body, true)?;
        Ok(meta)
    }

    fn verify(&self, keystore_id: &str, password: &[u8]) -> Result<StoredSecretMeta, KeystoreError> {
        let _queue = queue::lock()?;
        self.verify_unlocked(keystore_id, password)
    }

    fn get_meta(&self, keystore_id: &str) -> Result<Option<StoredSecretMeta>, KeystoreError> {
        let _queue = queue::lock()?;
        self.get_meta_unlocked(keystore_id)
    }

    fn list(&self) -> Result<Vec<Result<StoredSecretMeta, KeystoreEntryError>>, KeystoreError> {
        let _queue = queue::lock()?;
        let mut entries = Vec::new();
        for entry in fs::read_dir(&self.base_dir)? {
            let path = entry?.path();
            if path.extension().and_then(|extension| extension.to_str()) != Some(FILE_EXTENSION) {
                continue;
            }
            let result = listed_meta(&path).map_err(|error| KeystoreEntryError {
                entry: path.display().to_string(),
                error: error.to_string(),
            });
            entries.push(result);
        }
        Ok(entries)
    }

    fn delete(&self, keystore_id: &str) -> Result<bool, KeystoreError> {
        let _queue = queue::lock()?;
        let id = KeystoreId::parse(keystore_id)?;
        match fs::remove_file(self.path_for_id(&id)) {
            Ok(()) => {
                sync_directory(&self.base_dir)?;
                Ok(true)
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(error.into()),
        }
    }
}

#[cfg(test)]
impl FileKeystore {
    pub(super) fn open_with_kdf(base_dir: PathBuf, default_kdf: KdfParams) -> Result<Self, KeystoreError> {
        let keystore = Self { base_dir, default_kdf };
        fs::create_dir_all(&keystore.base_dir)?;
        Ok(keystore)
    }
}

#[cfg(feature = "v3")]
fn is_v3_file(path: &Path, legacy_id: &str) -> bool {
    let name = path.file_name().and_then(|name| name.to_str()).unwrap_or_default().to_lowercase();
    name == legacy_id || name.ends_with(legacy_id) || ReaderV3::file_id(path).is_some_and(|id| id.to_lowercase() == legacy_id)
}

fn parsed_meta(bytes: &[u8], expected_id: &KeystoreId) -> Result<StoredSecretMeta, KeystoreError> {
    let parsed = parse_v4(bytes)?;
    if parsed.header.keystore_id != expected_id.as_str() {
        return Err(KeystoreError::corrupt_file("keystore id does not match filename"));
    }
    Ok(meta_from_header(&parsed.header))
}

fn listed_meta(path: &Path) -> Result<StoredSecretMeta, KeystoreError> {
    let expected_id = keystore_id_from_path(path)?;
    let bytes = read_capped(path, WHOLE_FILE_CAP)?;
    parsed_meta(&bytes, &expected_id)
}

fn keystore_id_from_path(path: &Path) -> Result<KeystoreId, KeystoreError> {
    let file_stem = path
        .file_stem()
        .and_then(|file_stem| file_stem.to_str())
        .ok_or_else(|| KeystoreError::corrupt_file("invalid keystore filename"))?;
    KeystoreId::parse(file_stem).map_err(|_| KeystoreError::corrupt_file("invalid keystore filename"))
}
