use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use zeroize::{Zeroize, Zeroizing};

#[cfg(feature = "v3")]
use crate::v3::{ReaderV3, SecretV3};

use crate::{KeystoreError, KeystoreId, Mnemonic};

use super::{
    constants::{AES_GCM_TAG_LEN, ENCRYPTED_BODY_CAP, FILE_EXTENSION, VERSION_V4, WHOLE_FILE_CAP},
    crypto::derive_key,
    file_io::{new_secret_file_options, read_capped, set_owner_read_write, sync_directory},
    format::{authenticated_bytes, encode_v4, meta_from_header, parse_v4, validate_v4_password},
    queue,
    types::{CipherParams, FileKeystore, Header, KdfParams, KeystoreFileError, KeystoreInspection, ParsedFile, SecretKind, SecretPayload, StoredSecretMeta},
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

    pub fn import_mnemonic(&self, phrase: &str, password: &[u8], keystore_id: Option<String>) -> Result<StoredSecretMeta, KeystoreError> {
        let _queue = queue::lock()?;
        self.import_mnemonic_unlocked(phrase, password, keystore_id)
    }

    fn import_mnemonic_unlocked(&self, phrase: &str, password: &[u8], keystore_id: Option<String>) -> Result<StoredSecretMeta, KeystoreError> {
        validate_v4_password(password)?;
        let phrase = Mnemonic::clean(phrase)?;
        let payload = SecretPayload::Mnemonic { phrase: phrase.to_string() };
        self.import_payload_unlocked(SecretKind::Mnemonic, payload, password, keystore_id)
    }

    pub fn import_private_key(&self, private_key: &[u8], password: &[u8], keystore_id: Option<String>) -> Result<StoredSecretMeta, KeystoreError> {
        let _queue = queue::lock()?;
        self.import_private_key_unlocked(private_key, password, keystore_id)
    }

    fn import_private_key_unlocked(&self, private_key: &[u8], password: &[u8], keystore_id: Option<String>) -> Result<StoredSecretMeta, KeystoreError> {
        validate_v4_password(password)?;
        if private_key.is_empty() || private_key.len() > ENCRYPTED_BODY_CAP {
            return Err(KeystoreError::invalid_input("private key"));
        }
        let payload = SecretPayload::PrivateKey { bytes: private_key.to_vec() };
        self.import_payload_unlocked(SecretKind::PrivateKey, payload, password, keystore_id)
    }

    pub fn decrypt_mnemonic(&self, keystore_id: &str, password: &[u8]) -> Result<Zeroizing<String>, KeystoreError> {
        let _queue = queue::lock()?;
        self.decrypt_payload_unlocked(keystore_id, password)?.into_mnemonic()
    }

    pub fn decrypt_mnemonic_with_meta(&self, keystore_id: &str, password: &[u8]) -> Result<(StoredSecretMeta, Zeroizing<String>), KeystoreError> {
        let _queue = queue::lock()?;
        let id = KeystoreId::parse(keystore_id)?;
        let parsed = self.read_parsed_by_id(&id)?;
        let meta = meta_from_header(&parsed.header);
        let phrase = decrypt_file(parsed, Some(&id), password)?.into_mnemonic()?;
        Ok((meta, phrase))
    }

    pub fn decrypt_private_key(&self, keystore_id: &str, password: &[u8]) -> Result<Zeroizing<Vec<u8>>, KeystoreError> {
        let _queue = queue::lock()?;
        self.decrypt_payload_unlocked(keystore_id, password)?.into_private_key()
    }

    pub fn change_password(&self, keystore_id: &str, old_password: &[u8], new_password: &[u8]) -> Result<StoredSecretMeta, KeystoreError> {
        let _queue = queue::lock()?;
        validate_v4_password(new_password)?;
        let id = KeystoreId::parse(keystore_id)?;
        let parsed = self.read_parsed_by_id(&id)?;
        let kind = parsed.header.kind;
        let payload = decrypt_file(parsed, Some(&id), old_password)?;
        let body = self.encrypt_payload(&kind, payload, new_password, Some(id.clone()))?;
        self.write_new_file(&id, &body, true)?;
        Ok(StoredSecretMeta {
            keystore_id: id.into_string(),
            kind,
            version: VERSION_V4,
        })
    }

    pub fn delete(&self, keystore_id: &str) -> Result<bool, KeystoreError> {
        let _queue = queue::lock()?;
        let id = KeystoreId::parse(keystore_id)?;
        let path = self.path_for_id(&id);
        match fs::remove_file(path) {
            Ok(()) => {
                sync_directory(&self.base_dir)?;
                Ok(true)
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(error.into()),
        }
    }

    pub fn get_meta(&self, keystore_id: &str) -> Result<Option<StoredSecretMeta>, KeystoreError> {
        let _queue = queue::lock()?;
        self.get_meta_unlocked(keystore_id)
    }

    fn get_meta_unlocked(&self, keystore_id: &str) -> Result<Option<StoredSecretMeta>, KeystoreError> {
        let id = KeystoreId::parse(keystore_id)?;
        let path = self.path_for_id(&id);
        if !path.exists() {
            return Ok(None);
        }
        let bytes = read_capped(&path, WHOLE_FILE_CAP)?;
        let parsed = parse_v4(&bytes)?;
        if parsed.header.keystore_id != id.as_str() {
            return Err(KeystoreError::corrupt_file("keystore id does not match filename"));
        }
        Ok(Some(meta_from_header(&parsed.header)))
    }

    pub fn list(&self) -> Result<Vec<Result<StoredSecretMeta, KeystoreFileError>>, KeystoreError> {
        let _queue = queue::lock()?;
        let mut entries = Vec::new();
        for entry in fs::read_dir(&self.base_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|extension| extension.to_str()) != Some(FILE_EXTENSION) {
                continue;
            }
            let result = listed_meta(&path).map_err(|error| KeystoreFileError {
                path: path.clone(),
                error: error.to_string(),
            });
            entries.push(result);
        }
        Ok(entries)
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
        let _payload = decrypt_file(parsed, None, password)?;
        Ok(meta)
    }

    pub fn verify(&self, keystore_id: &str, password: &[u8]) -> Result<StoredSecretMeta, KeystoreError> {
        let _queue = queue::lock()?;
        self.verify_unlocked(keystore_id, password)
    }

    fn verify_unlocked(&self, keystore_id: &str, password: &[u8]) -> Result<StoredSecretMeta, KeystoreError> {
        let id = KeystoreId::parse(keystore_id)?;
        let parsed = self.read_parsed_by_id(&id)?;
        let meta = meta_from_header(&parsed.header);
        let _payload = decrypt_file(parsed, Some(&id), password)?;
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

    fn import_payload_unlocked(&self, kind: SecretKind, payload: SecretPayload, password: &[u8], keystore_id: Option<String>) -> Result<StoredSecretMeta, KeystoreError> {
        let id = match keystore_id {
            Some(keystore_id) => KeystoreId::parse(&keystore_id)?,
            None => KeystoreId::new(),
        };
        if self.path_for_id(&id).exists() {
            return self.verify_unlocked(id.as_str(), password);
        }
        let body = self.encrypt_payload(&kind, payload, password, Some(id.clone()))?;
        self.write_new_file(&id, &body, false)?;
        self.get_meta_unlocked(id.as_str())?.ok_or(KeystoreError::NotFound)
    }

    fn encrypt_payload(&self, kind: &SecretKind, payload: SecretPayload, password: &[u8], keystore_id: Option<KeystoreId>) -> Result<Vec<u8>, KeystoreError> {
        validate_v4_password(password)?;
        let id = keystore_id.unwrap_or_default();
        let header = Header {
            keystore_id: id.into_string(),
            kind: *kind,
            kdf: self.default_kdf.with_random_salt()?,
            cipher: CipherParams::random_aes256_gcm()?,
        };
        let aad = authenticated_bytes(&header)?;
        let mut body = payload.into_bytes();
        if body.len() + AES_GCM_TAG_LEN as usize > ENCRYPTED_BODY_CAP {
            return Err(KeystoreError::corrupt_file("payload too large"));
        }
        let key = derive_key(password, &header.kdf)?;
        super::crypto::encrypt_aes256_gcm(key.as_ref(), header.cipher.nonce(), &aad, &mut body)?;
        let bytes = encode_v4(&header, &body)?;
        // Hex doubles the ciphertext; reject anything the read cap would refuse later.
        if bytes.len() > WHOLE_FILE_CAP {
            return Err(KeystoreError::corrupt_file("payload too large"));
        }
        Ok(bytes)
    }

    fn decrypt_payload_unlocked(&self, keystore_id: &str, password: &[u8]) -> Result<SecretPayload, KeystoreError> {
        let id = KeystoreId::parse(keystore_id)?;
        let parsed = self.read_parsed_by_id(&id)?;
        decrypt_file(parsed, Some(&id), password)
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
        let options = new_secret_file_options();
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

#[cfg(test)]
impl FileKeystore {
    pub(super) fn open_with_kdf(base_dir: PathBuf, default_kdf: KdfParams) -> Result<Self, KeystoreError> {
        let keystore = Self { base_dir, default_kdf };
        fs::create_dir_all(&keystore.base_dir)?;
        Ok(keystore)
    }
}

fn decrypt_file(parsed: ParsedFile, expected_id: Option<&KeystoreId>, password: &[u8]) -> Result<SecretPayload, KeystoreError> {
    validate_v4_password(password)?;
    if let Some(expected_id) = expected_id
        && parsed.header.keystore_id != expected_id.as_str()
    {
        return Err(KeystoreError::corrupt_file("authenticated keystore id does not match filename"));
    }
    let ParsedFile { header, ciphertext: mut body } = parsed;
    let aad = authenticated_bytes(&header)?;
    let key = derive_key(password, &header.kdf)?;
    // On tag failure aes-gcm leaves unauthenticated plaintext in the buffer, so zeroize before bailing.
    if let Err(error) = super::crypto::decrypt_aes256_gcm(key.as_ref(), header.cipher.nonce(), &aad, &mut body) {
        body.zeroize();
        return Err(error);
    }
    SecretPayload::from_bytes(header.kind, body)
}

fn listed_meta(path: &Path) -> Result<StoredSecretMeta, KeystoreError> {
    let expected_id = keystore_id_from_path(path)?;
    let bytes = read_capped(path, WHOLE_FILE_CAP)?;
    let parsed = parse_v4(&bytes)?;
    if parsed.header.keystore_id != expected_id.as_str() {
        return Err(KeystoreError::corrupt_file("keystore id does not match filename"));
    }
    Ok(meta_from_header(&parsed.header))
}

fn keystore_id_from_path(path: &Path) -> Result<KeystoreId, KeystoreError> {
    let file_stem = path
        .file_stem()
        .and_then(|file_stem| file_stem.to_str())
        .ok_or_else(|| KeystoreError::corrupt_file("invalid keystore filename"))?;
    KeystoreId::parse(file_stem).map_err(|_| KeystoreError::corrupt_file("invalid keystore filename"))
}
