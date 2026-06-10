use std::convert::TryInto;

use super::{
    constants::{ENCRYPTED_BODY_CAP, HEADER_LEN_CAP, HEADER_LEN_END, HEADER_LEN_OFFSET, MAGIC, PASSWORD_CAP, PREFIX_LEN, VERSION_V4, WHOLE_FILE_CAP},
    types::{Header, ParsedFile, SecretKind, SecretPayload, StoredSecretMeta},
};
use crate::{KeystoreError, KeystoreId};

pub(super) fn parse_v4(bytes: &[u8]) -> Result<ParsedFile<'_>, KeystoreError> {
    validate_file_len(bytes)?;
    validate_magic(bytes)?;
    validate_version(bytes)?;
    let header_len = read_header_len(bytes)?;
    let header_end = checked_header_end(header_len)?;
    let header_bytes = read_header_bytes(bytes, header_end)?;
    let body = read_body(bytes, header_end)?;
    validate_body_len(body)?;
    let header = parse_header(header_bytes)?;
    validate_header(&header)?;
    validate_body_tag(body, &header)?;
    Ok(ParsedFile {
        header,
        header_len,
        header_end,
        body,
    })
}

fn validate_file_len(bytes: &[u8]) -> Result<(), KeystoreError> {
    if bytes.len() > WHOLE_FILE_CAP {
        return Err(KeystoreError::corrupt_file("file too large"));
    }
    if bytes.len() < PREFIX_LEN {
        return Err(KeystoreError::corrupt_file("file too short"));
    }
    Ok(())
}

fn validate_magic(bytes: &[u8]) -> Result<(), KeystoreError> {
    let magic = bytes.get(..MAGIC.len()).ok_or_else(|| KeystoreError::corrupt_file("missing magic"))?;
    if magic != MAGIC.as_slice() {
        return Err(KeystoreError::corrupt_file("invalid magic"));
    }
    Ok(())
}

fn validate_version(bytes: &[u8]) -> Result<(), KeystoreError> {
    let version = *bytes.get(MAGIC.len()).ok_or_else(|| KeystoreError::corrupt_file("missing version"))?;
    if version != VERSION_V4 {
        return Err(KeystoreError::unsupported("version"));
    }
    Ok(())
}

fn read_header_len(bytes: &[u8]) -> Result<u32, KeystoreError> {
    let header_len_bytes: [u8; 4] = bytes
        .get(HEADER_LEN_OFFSET..HEADER_LEN_END)
        .ok_or_else(|| KeystoreError::corrupt_file("missing header length"))?
        .try_into()
        .map_err(|_| KeystoreError::corrupt_file("invalid header length"))?;
    Ok(u32::from_be_bytes(header_len_bytes))
}

fn checked_header_end(header_len: u32) -> Result<usize, KeystoreError> {
    let header_len_usize = usize::try_from(header_len).map_err(|_| KeystoreError::corrupt_file("header length overflow"))?;
    if header_len_usize > HEADER_LEN_CAP {
        return Err(KeystoreError::corrupt_file("header too large"));
    }
    PREFIX_LEN
        .checked_add(header_len_usize)
        .ok_or_else(|| KeystoreError::corrupt_file("header length overflow"))
}

fn read_header_bytes(bytes: &[u8], header_end: usize) -> Result<&[u8], KeystoreError> {
    bytes.get(PREFIX_LEN..header_end).ok_or_else(|| KeystoreError::corrupt_file("truncated header"))
}

fn read_body(bytes: &[u8], header_end: usize) -> Result<&[u8], KeystoreError> {
    bytes.get(header_end..).ok_or_else(|| KeystoreError::corrupt_file("missing encrypted body"))
}

fn validate_body_len(body: &[u8]) -> Result<(), KeystoreError> {
    if body.len() > ENCRYPTED_BODY_CAP {
        return Err(KeystoreError::corrupt_file("encrypted body too large"));
    }
    Ok(())
}

fn parse_header(header_bytes: &[u8]) -> Result<Header, KeystoreError> {
    borsh::from_slice::<Header>(header_bytes).map_err(|error| KeystoreError::corrupt_file(error.to_string()))
}

fn validate_body_tag(body: &[u8], header: &Header) -> Result<(), KeystoreError> {
    if body.len() < usize::from(header.cipher.tag_len()) {
        return Err(KeystoreError::corrupt_file("encrypted body shorter than tag"));
    }
    Ok(())
}

pub(super) fn validate_payload_kind(kind: &SecretKind, payload: &SecretPayload) -> Result<(), KeystoreError> {
    match (kind, payload) {
        (SecretKind::Mnemonic, SecretPayload::Mnemonic { .. }) | (SecretKind::PrivateKey, SecretPayload::PrivateKey { .. }) => Ok(()),
        (SecretKind::Mnemonic, SecretPayload::PrivateKey { .. }) | (SecretKind::PrivateKey, SecretPayload::Mnemonic { .. }) => {
            Err(KeystoreError::corrupt_file("header kind does not match payload"))
        }
    }
}

pub(super) fn meta_from_header(header: &Header) -> StoredSecretMeta {
    StoredSecretMeta {
        keystore_id: header.keystore_id.clone(),
        kind: header.kind,
        version: VERSION_V4,
    }
}

pub(crate) fn validate_v4_password(password: &[u8]) -> Result<(), KeystoreError> {
    if password.is_empty() || password.len() > PASSWORD_CAP {
        return Err(KeystoreError::invalid_input("password input"));
    }
    Ok(())
}

fn validate_header(header: &Header) -> Result<(), KeystoreError> {
    KeystoreId::parse(&header.keystore_id).map_err(|_| KeystoreError::corrupt_file("invalid keystore id"))?;
    header.kdf.validate()?;
    header.cipher.validate()?;
    Ok(())
}
