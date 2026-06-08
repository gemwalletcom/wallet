use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use zeroize::{Zeroize, Zeroizing};

#[cfg(feature = "v3")]
use crate::v3::{ReaderV3, SecretV3};

use crate::{KeystoreError, KeystoreId, Mnemonic};

use super::{
    constants::{AES_GCM_TAG_LEN, ENCRYPTED_BODY_CAP, HEADER_LEN_CAP, MAGIC, PREFIX_LEN, VERSION_V4, WHOLE_FILE_CAP},
    crypto::derive_key,
    file_io::{new_secret_file_options, read_capped, set_owner_read_write, sync_directory},
    format::{meta_from_header, parse_v4, validate_payload_kind, validate_v4_password},
    queue,
    types::{CipherParams, FileKeystore, Header, KdfParams, KeystoreFileError, KeystoreInspection, ParsedFile, SecretKind, SecretPayload, StoredSecretMeta},
};

struct OwnedHeader {
    header: Header,
}

struct OwnedFile {
    parsed: OwnedHeader,
    bytes: Vec<u8>,
}

impl FileKeystore {
    pub fn open(base_dir: PathBuf) -> Result<Self, KeystoreError> {
        let keystore = Self {
            base_dir,
            default_kdf: KdfParams::default_argon2id()?,
        };
        fs::create_dir_all(keystore.v4_dir())?;
        Ok(keystore)
    }

    pub fn import_mnemonic(&self, phrase: &str, password: &[u8], keystore_id: Option<String>) -> Result<StoredSecretMeta, KeystoreError> {
        let _queue = queue::lock()?;
        self.import_mnemonic_unlocked(phrase, password, keystore_id)
    }

    fn import_mnemonic_unlocked(&self, phrase: &str, password: &[u8], keystore_id: Option<String>) -> Result<StoredSecretMeta, KeystoreError> {
        validate_v4_password(password)?;
        let phrase = Mnemonic::sanitize(phrase)?;
        let payload = SecretPayload::Mnemonic { phrase };
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
        let meta = meta_from_header(&parsed.parsed.header);
        let phrase = self.decrypt_parsed(&parsed.bytes, Some(&id), password)?.into_mnemonic()?;
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
        let kind = parsed.parsed.header.kind;
        let created_at = parsed.parsed.header.created_at;
        let payload = self.decrypt_parsed(&parsed.bytes, Some(&id), old_password)?;
        let body = self.encrypt_payload(&kind, payload, new_password, Some(id.clone()), Some(created_at))?;
        self.write_new_file(&id, &body, true)?;
        Ok(StoredSecretMeta {
            keystore_id: id.into_string(),
            kind,
            version: VERSION_V4,
            created_at,
        })
    }

    pub fn delete(&self, keystore_id: &str) -> Result<bool, KeystoreError> {
        let _queue = queue::lock()?;
        let id = KeystoreId::parse(keystore_id)?;
        let path = self.path_for_id(&id);
        match fs::remove_file(path) {
            Ok(()) => Ok(true),
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
        for entry in fs::read_dir(self.v4_dir())? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|extension| extension.to_str()) != Some("gemk") {
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
            header_len: parsed.header_len,
            ciphertext_len: parsed.body.len() as u64,
            tag_len: parsed.header.cipher.tag_len(),
            warnings: Vec::new(),
        })
    }

    pub fn verify_path(path: &Path, password: &[u8]) -> Result<StoredSecretMeta, KeystoreError> {
        let _queue = queue::lock()?;
        let bytes = read_capped(path, WHOLE_FILE_CAP)?;
        let parsed = parse_v4(&bytes)?;
        let payload = decrypt_parsed_bytes(&bytes, &parsed, None, password)?;
        validate_payload_kind(&parsed.header.kind, &payload)?;
        Ok(meta_from_header(&parsed.header))
    }

    pub fn verify(&self, keystore_id: &str, password: &[u8]) -> Result<StoredSecretMeta, KeystoreError> {
        let _queue = queue::lock()?;
        self.verify_unlocked(keystore_id, password)
    }

    fn verify_unlocked(&self, keystore_id: &str, password: &[u8]) -> Result<StoredSecretMeta, KeystoreError> {
        let id = KeystoreId::parse(keystore_id)?;
        let parsed = self.read_parsed_by_id(&id)?;
        let payload = self.decrypt_parsed(&parsed.bytes, Some(&id), password)?;
        validate_payload_kind(&parsed.parsed.header.kind, &payload)?;
        Ok(meta_from_header(&parsed.parsed.header))
    }

    #[cfg(feature = "v3")]
    pub fn import_v3(&self, v3_path: &Path, v3_password: &[u8], new_password: &[u8], keystore_id: Option<String>) -> Result<StoredSecretMeta, KeystoreError> {
        let _queue = queue::lock()?;
        // Idempotent retry: authenticate an existing staged v4 file by id+password instead of failing.
        if let Some(parsed_id) = keystore_id
            .as_deref()
            .and_then(|id| KeystoreId::parse(id).ok())
            .filter(|parsed_id| self.path_for_id(parsed_id).exists())
        {
            return self.verify_unlocked(parsed_id.as_str(), new_password);
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
        let body = self.encrypt_payload(&kind, payload, password, Some(id.clone()), None)?;
        self.write_new_file(&id, &body, false)?;
        self.get_meta_unlocked(id.as_str())?.ok_or(KeystoreError::NotFound)
    }

    fn encrypt_payload(
        &self,
        kind: &SecretKind,
        payload: SecretPayload,
        password: &[u8],
        keystore_id: Option<KeystoreId>,
        created_at: Option<i64>,
    ) -> Result<Vec<u8>, KeystoreError> {
        validate_v4_password(password)?;
        let id = keystore_id.unwrap_or_default();
        let created_at = match created_at {
            Some(created_at) => created_at,
            None => {
                let seconds = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|error| KeystoreError::io(error.to_string()))?
                    .as_secs();
                i64::try_from(seconds).map_err(|_| KeystoreError::io("system time overflow"))?
            }
        };
        let header = Header {
            keystore_id: id.into_string(),
            kind: *kind,
            created_at,
            kdf: self.default_kdf.with_random_salt()?,
            cipher: CipherParams::random_aes256_gcm()?,
        };
        let header_bytes = borsh::to_vec(&header).map_err(|error| KeystoreError::corrupt_file(error.to_string()))?;
        if header_bytes.len() > HEADER_LEN_CAP {
            return Err(KeystoreError::corrupt_file("header too large"));
        }
        let mut bytes = Vec::with_capacity(PREFIX_LEN + header_bytes.len() + AES_GCM_TAG_LEN as usize);
        bytes.extend_from_slice(MAGIC);
        bytes.push(VERSION_V4);
        bytes.extend_from_slice(&(header_bytes.len() as u32).to_be_bytes());
        bytes.extend_from_slice(&header_bytes);
        let aad_len = bytes.len();
        let mut body = borsh::to_vec(&payload).map_err(|error| KeystoreError::corrupt_file(error.to_string()))?;
        if body.len() + AES_GCM_TAG_LEN as usize > ENCRYPTED_BODY_CAP {
            body.zeroize();
            return Err(KeystoreError::corrupt_file("payload too large"));
        }
        let key = derive_key(password, &header.kdf)?;
        if let Err(error) = super::crypto::encrypt_aes256_gcm(key.as_ref(), header.cipher.nonce(), &bytes[..aad_len], &mut body) {
            body.zeroize();
            return Err(error);
        }
        bytes.append(&mut body);
        Ok(bytes)
    }

    fn decrypt_payload_unlocked(&self, keystore_id: &str, password: &[u8]) -> Result<SecretPayload, KeystoreError> {
        let id = KeystoreId::parse(keystore_id)?;
        let parsed = self.read_parsed_by_id(&id)?;
        self.decrypt_parsed(&parsed.bytes, Some(&id), password)
    }

    fn decrypt_parsed(&self, bytes: &[u8], expected_id: Option<&KeystoreId>, password: &[u8]) -> Result<SecretPayload, KeystoreError> {
        let parsed = parse_v4(bytes)?;
        decrypt_parsed_bytes(bytes, &parsed, expected_id, password)
    }

    fn read_parsed_by_id(&self, id: &KeystoreId) -> Result<OwnedFile, KeystoreError> {
        let path = self.path_for_id(id);
        let bytes = read_capped(&path, WHOLE_FILE_CAP)?;
        let header = {
            let parsed = parse_v4(&bytes)?;
            parsed.header.clone()
        };
        Ok(OwnedFile {
            parsed: OwnedHeader { header },
            bytes,
        })
    }

    fn write_new_file(&self, id: &KeystoreId, bytes: &[u8], replace: bool) -> Result<(), KeystoreError> {
        fs::create_dir_all(self.v4_dir())?;
        let path = self.path_for_id(id);
        if !replace && path.exists() {
            return Err(KeystoreError::AlreadyExists);
        }
        let temp_path = self.v4_dir().join(format!("{}.gemk.tmp.{}", id.as_str(), KeystoreId::new()));
        let options = new_secret_file_options();
        let write_result = (|| -> Result<(), KeystoreError> {
            let mut file = options.open(&temp_path)?;
            set_owner_read_write(&temp_path)?;
            file.write_all(bytes)?;
            file.sync_all()?;
            fs::rename(&temp_path, &path)?;
            sync_directory(&self.v4_dir())?;
            Ok(())
        })();
        if write_result.is_err() {
            let _ = fs::remove_file(&temp_path);
        }
        write_result
    }

    fn path_for_id(&self, id: &KeystoreId) -> PathBuf {
        self.v4_dir().join(format!("{}.gemk", id.as_str()))
    }

    fn v4_dir(&self) -> PathBuf {
        self.base_dir.join("v4")
    }
}

#[cfg(test)]
impl FileKeystore {
    pub(super) fn open_with_kdf(base_dir: PathBuf, default_kdf: KdfParams) -> Result<Self, KeystoreError> {
        let keystore = Self { base_dir, default_kdf };
        fs::create_dir_all(keystore.v4_dir())?;
        Ok(keystore)
    }
}

fn decrypt_parsed_bytes(bytes: &[u8], parsed: &ParsedFile<'_>, expected_id: Option<&KeystoreId>, password: &[u8]) -> Result<SecretPayload, KeystoreError> {
    validate_v4_password(password)?;
    if let Some(expected_id) = expected_id
        && parsed.header.keystore_id != expected_id.as_str()
    {
        return Err(KeystoreError::corrupt_file("authenticated keystore id does not match filename"));
    }
    let key = derive_key(password, &parsed.header.kdf)?;
    let mut body = parsed.body.to_vec();
    // On tag failure aes-gcm leaves unauthenticated plaintext in the buffer, so zeroize before bailing.
    if let Err(error) = super::crypto::decrypt_aes256_gcm(key.as_ref(), parsed.header.cipher.nonce(), &bytes[..parsed.header_end], &mut body) {
        body.zeroize();
        return Err(error);
    }
    let payload = match borsh::from_slice::<SecretPayload>(&body) {
        Ok(payload) => payload,
        Err(error) => {
            body.zeroize();
            return Err(KeystoreError::corrupt_file(error.to_string()));
        }
    };
    body.zeroize();
    validate_payload_kind(&parsed.header.kind, &payload)?;
    Ok(payload)
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
