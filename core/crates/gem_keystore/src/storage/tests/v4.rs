use std::fs;
use std::sync::{Arc, Barrier};
use std::thread;

use crate::{KeystoreError, KeystoreId};

use super::super::{
    constants::{AES_GCM_TAG_LEN, HEADER_LEN_CAP, MAGIC, PREFIX_LEN, VERSION_V4},
    format::parse_v4,
    types::{FileKeystore, KdfParams, SecretKind},
};
use super::testkit::{PHRASE, assert_verify_path_error, new_keystore_id, test_keystore, v4_path, write_tampered};

#[test]
fn test_v4_mnemonic_roundtrip() {
    let (_dir, keystore) = test_keystore();
    let password = b"password";
    let meta = keystore.import_mnemonic(PHRASE, password, None).unwrap();
    assert_eq!(meta.kind, SecretKind::Mnemonic);
    assert_eq!(KeystoreId::parse(&meta.keystore_id).unwrap().as_str(), meta.keystore_id);
    assert_eq!(keystore.decrypt_mnemonic(&meta.keystore_id, password).unwrap().as_str(), PHRASE);
    assert_eq!(keystore.decrypt_mnemonic(&meta.keystore_id, b"wrong").unwrap_err(), KeystoreError::AuthenticationFailed);
}

#[test]
fn test_v4_private_key_roundtrip_and_meta() {
    let (_dir, keystore) = test_keystore();
    let password = b"password";
    let private_key = [7u8; 32];
    let meta = keystore.import_private_key(&private_key, password, None).unwrap();
    assert_eq!(meta.kind, SecretKind::PrivateKey);
    assert_eq!(keystore.decrypt_private_key(&meta.keystore_id, password).unwrap().as_slice(), private_key);
    assert_eq!(keystore.get_meta(&meta.keystore_id).unwrap().unwrap().kind, SecretKind::PrivateKey);
    assert!(keystore.delete(&meta.keystore_id).unwrap());
    assert!(!keystore.delete(&meta.keystore_id).unwrap());
}

#[test]
fn test_v4_rejects_invalid_ids_before_path_construction() {
    let (_dir, keystore) = test_keystore();
    let password = b"password";
    let invalid_ids = [
        "",
        "../secret",
        "550e8400-e29b-11d4-a716-446655440000",
        "550E8400-E29B-41D4-A716-446655440000",
        "550e8400-e29b-41d4-a716-446655440000.gemk",
    ];
    for id in invalid_ids {
        assert_eq!(
            keystore.import_mnemonic(PHRASE, password, Some(id.to_string())).unwrap_err(),
            KeystoreError::invalid_input("keystore id")
        );
        assert_eq!(keystore.get_meta(id).unwrap_err(), KeystoreError::invalid_input("keystore id"));
    }
}

#[test]
fn test_v4_header_filename_mismatch_fails_after_authentication() {
    let (dir, keystore) = test_keystore();
    let password = b"password";
    let id_a = KeystoreId::new();
    let id_b = KeystoreId::new();
    let meta = keystore.import_mnemonic(PHRASE, password, Some(id_a.to_string())).unwrap();
    fs::copy(v4_path(&dir, &meta.keystore_id), dir.path().join("v4").join(format!("{}.gemk", id_b.as_str()))).unwrap();
    assert_eq!(
        keystore.get_meta(id_b.as_str()).unwrap_err(),
        KeystoreError::corrupt_file("keystore id does not match filename")
    );
    let listed = keystore.list().unwrap();
    let listed_error = listed.iter().find_map(|result| result.as_ref().err()).unwrap();
    assert_eq!(listed_error.error, "Corrupt keystore file: keystore id does not match filename");
    assert_eq!(
        keystore.verify(id_b.as_str(), password).unwrap_err(),
        KeystoreError::corrupt_file("authenticated keystore id does not match filename")
    );
}

#[test]
fn test_v4_change_password_and_list_inspect() {
    let (dir, keystore) = test_keystore();
    let old_password = b"old-password";
    let new_password = b"new-password";
    let meta = keystore.import_mnemonic(PHRASE, old_password, None).unwrap();
    let changed = keystore.change_password(&meta.keystore_id, old_password, new_password).unwrap();
    assert_eq!(changed.keystore_id, meta.keystore_id);
    assert_eq!(keystore.decrypt_mnemonic(&meta.keystore_id, old_password).unwrap_err(), KeystoreError::AuthenticationFailed);
    assert_eq!(keystore.decrypt_mnemonic(&meta.keystore_id, new_password).unwrap().as_str(), PHRASE);

    let listed = keystore.list().unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].as_ref().unwrap().keystore_id, meta.keystore_id);

    let inspected = FileKeystore::inspect_path(&v4_path(&dir, &meta.keystore_id)).unwrap();
    assert_eq!(inspected.meta.unwrap().keystore_id, meta.keystore_id);
    assert!(!inspected.authenticated);
    assert!(inspected.ciphertext_len >= u64::from(AES_GCM_TAG_LEN));

    let path = v4_path(&dir, &meta.keystore_id);
    assert_eq!(FileKeystore::verify_path(&path, new_password).unwrap().keystore_id, meta.keystore_id);
    assert_eq!(FileKeystore::verify_path(&path, old_password).unwrap_err(), KeystoreError::AuthenticationFailed);
}

#[test]
fn test_v4_password_bounds() {
    let (_dir, keystore) = test_keystore();
    assert_eq!(keystore.import_mnemonic(PHRASE, b"", None).unwrap_err(), KeystoreError::invalid_input("password input"));
    assert_eq!(
        keystore.import_private_key(&[], b"password", None).unwrap_err(),
        KeystoreError::invalid_input("private key")
    );
}

#[test]
fn test_v4_rejects_roundtrip_tampering() {
    let (dir, keystore) = test_keystore();
    let password = b"password";
    let meta = keystore.import_mnemonic(PHRASE, password, None).unwrap();
    let original = fs::read(v4_path(&dir, &meta.keystore_id)).unwrap();
    let tampered_path = dir.path().join("tampered.gemk");

    let mut magic = original.clone();
    magic[0] ^= 0xff;
    write_tampered(&tampered_path, &magic);
    assert_verify_path_error(&tampered_path, password, KeystoreError::corrupt_file("invalid magic"));

    let mut version = original.clone();
    version[4] ^= 0xff;
    write_tampered(&tampered_path, &version);
    assert_verify_path_error(&tampered_path, password, KeystoreError::unsupported("version"));

    let created_at_offset = PREFIX_LEN + 4 + meta.keystore_id.len() + 1;
    let mut header = original.clone();
    header[created_at_offset] ^= 0xff;
    write_tampered(&tampered_path, &header);
    assert_verify_path_error(&tampered_path, password, KeystoreError::AuthenticationFailed);

    let parsed = parse_v4(&original).unwrap();
    let mut ciphertext = original.clone();
    ciphertext[parsed.header_end] ^= 0xff;
    write_tampered(&tampered_path, &ciphertext);
    assert_verify_path_error(&tampered_path, password, KeystoreError::AuthenticationFailed);

    let mut tag = original;
    let last = tag.len() - 1;
    tag[last] ^= 0xff;
    write_tampered(&tampered_path, &tag);
    assert_verify_path_error(&tampered_path, password, KeystoreError::AuthenticationFailed);
}

#[test]
fn test_v4_rejects_malformed_files() {
    let mut bytes = Vec::new();
    assert_eq!(parse_v4(&bytes).unwrap_err(), KeystoreError::corrupt_file("file too short"));
    bytes.extend_from_slice(b"NOPE");
    bytes.push(VERSION_V4);
    bytes.extend_from_slice(&0u32.to_be_bytes());
    assert_eq!(parse_v4(&bytes).unwrap_err(), KeystoreError::corrupt_file("invalid magic"));

    let mut huge_header = Vec::new();
    huge_header.extend_from_slice(MAGIC);
    huge_header.push(VERSION_V4);
    huge_header.extend_from_slice(&((HEADER_LEN_CAP + 1) as u32).to_be_bytes());
    assert_eq!(parse_v4(&huge_header).unwrap_err(), KeystoreError::corrupt_file("header too large"));

    let mut malicious_borsh_header = Vec::new();
    malicious_borsh_header.extend_from_slice(MAGIC);
    malicious_borsh_header.push(VERSION_V4);
    malicious_borsh_header.extend_from_slice(&8u32.to_be_bytes());
    malicious_borsh_header.extend_from_slice(&u64::MAX.to_le_bytes());
    match parse_v4(&malicious_borsh_header).unwrap_err() {
        KeystoreError::CorruptFile(message) => assert!(!message.is_empty()),
        error => panic!("expected corrupt file, got {error:?}"),
    }
}

#[test]
fn test_v4_concurrent_import_same_wallet_is_idempotent() {
    let (dir, _keystore) = test_keystore();
    let keystore = Arc::new(FileKeystore::open_with_kdf(dir.path().to_path_buf(), KdfParams::mock()).unwrap());
    let barrier = Arc::new(Barrier::new(2));
    let id = new_keystore_id();
    let password = b"password".to_vec();

    let handles = (0..2)
        .map(|_| {
            let keystore = Arc::clone(&keystore);
            let barrier = Arc::clone(&barrier);
            let id = id.clone();
            let password = password.clone();
            thread::spawn(move || {
                barrier.wait();
                keystore.import_mnemonic(PHRASE, &password, Some(id))
            })
        })
        .collect::<Vec<_>>();

    let results = handles.into_iter().map(|handle| handle.join().unwrap()).collect::<Vec<_>>();
    assert!(results.iter().all(|result| result.is_ok()));
    assert_eq!(keystore.decrypt_mnemonic(&id, &password).unwrap().as_str(), PHRASE);

    // Re-importing under the same id with a different password must not clobber the existing keystore.
    assert_eq!(
        keystore.import_mnemonic(PHRASE, b"other-password", Some(id.clone())).unwrap_err(),
        KeystoreError::AuthenticationFailed
    );
    assert_eq!(keystore.decrypt_mnemonic(&id, &password).unwrap().as_str(), PHRASE);
}
