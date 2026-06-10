use std::fs;
use std::sync::{Arc, Barrier};
use std::thread;

use crate::{KeystoreError, KeystoreId};

use super::super::{
    constants::AES_GCM_TAG_LEN,
    format::{FileV4, parse_v4},
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
        "550e8400-e29b-41d4-a716-446655440000.json",
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
    fs::copy(v4_path(&dir, &meta.keystore_id), dir.path().join(format!("{}.json", id_b.as_str()))).unwrap();
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
fn test_v4_json_shape() {
    let (dir, keystore) = test_keystore();
    let meta = keystore.import_mnemonic(PHRASE, b"password", None).unwrap();
    let file = read_file_v4(&dir, &meta.keystore_id);

    assert_eq!(file.version, 4);
    assert_eq!(file.id, meta.keystore_id);
    assert_eq!(file.kind, "mnemonic");
    assert_eq!(file.crypto.kdf.algorithm, "argon2id");
    assert_eq!(file.crypto.kdf.salt.len(), 32);
    assert_eq!(file.crypto.cipher.algorithm, "aes-256-gcm");
    assert_eq!(file.crypto.cipher.nonce.len(), 24);
    assert_eq!(file.crypto.cipher.tag_len, 16);
    assert!(file.crypto.ciphertext.len() > 32);
}

#[test]
fn test_v4_rejects_roundtrip_tampering() {
    let (dir, keystore) = test_keystore();
    let password = b"password";
    let meta = keystore.import_mnemonic(PHRASE, password, None).unwrap();
    let original = read_file_v4(&dir, &meta.keystore_id);
    let tampered_path = dir.path().join("tampered.json");
    let tamper = |mutate: &dyn Fn(&mut FileV4)| {
        let mut file = original.clone();
        mutate(&mut file);
        write_tampered(&tampered_path, &serde_json::to_vec(&file).unwrap());
    };

    tamper(&|file| file.version = 5);
    assert_verify_path_error(&tampered_path, password, KeystoreError::unsupported("version"));

    tamper(&|file| file.kind = "private_key".to_string());
    assert_verify_path_error(&tampered_path, password, KeystoreError::AuthenticationFailed);

    tamper(&|file| file.crypto.kdf.memory_kib = 8);
    assert_verify_path_error(&tampered_path, password, KeystoreError::AuthenticationFailed);

    tamper(&|file| file.crypto.kdf.salt = flip_hex_at(&file.crypto.kdf.salt, 0));
    assert_verify_path_error(&tampered_path, password, KeystoreError::AuthenticationFailed);

    tamper(&|file| file.crypto.cipher.nonce = flip_hex_at(&file.crypto.cipher.nonce, 0));
    assert_verify_path_error(&tampered_path, password, KeystoreError::AuthenticationFailed);

    tamper(&|file| file.crypto.ciphertext = flip_hex_at(&file.crypto.ciphertext, 0));
    assert_verify_path_error(&tampered_path, password, KeystoreError::AuthenticationFailed);

    // The last ciphertext byte is part of the GCM tag.
    tamper(&|file| file.crypto.ciphertext = flip_hex_at(&file.crypto.ciphertext, file.crypto.ciphertext.len() - 1));
    assert_verify_path_error(&tampered_path, password, KeystoreError::AuthenticationFailed);
}

#[test]
fn test_v4_rejects_malformed_files() {
    for bytes in [b"".as_slice(), b"not json".as_slice(), br#"{"version":4}"#.as_slice()] {
        match parse_v4(bytes).unwrap_err() {
            KeystoreError::CorruptFile(message) => assert!(!message.is_empty()),
            error => panic!("expected corrupt file, got {error:?}"),
        }
    }

    let parse_error = |file: &FileV4| parse_v4(&serde_json::to_vec(file).unwrap()).unwrap_err();

    let mut file = FileV4::mock();
    file.version = 3;
    assert_eq!(parse_error(&file), KeystoreError::unsupported("version"));

    let mut file = FileV4::mock();
    file.kind = "seed".to_string();
    assert_eq!(parse_error(&file), KeystoreError::corrupt_file("unknown secret kind"));

    let mut file = FileV4::mock();
    file.crypto.kdf.salt = "zz".to_string();
    assert_eq!(parse_error(&file), KeystoreError::corrupt_file("invalid hex"));

    let mut file = FileV4::mock();
    file.crypto.kdf.salt = "0000".to_string();
    assert_eq!(parse_error(&file), KeystoreError::corrupt_file("invalid Argon2 salt length"));

    let mut json = serde_json::to_string(&FileV4::mock()).unwrap();
    json.insert_str(1, r#""unknown_field":1,"#);
    match parse_v4(json.as_bytes()).unwrap_err() {
        KeystoreError::CorruptFile(message) => assert!(message.contains("unknown field"), "{message}"),
        error => panic!("expected corrupt file, got {error:?}"),
    }
}

#[test]
fn test_v4_rejects_payload_that_would_exceed_the_read_cap() {
    let (_dir, keystore) = test_keystore();
    let oversized = vec![7u8; 65_500];
    assert_eq!(
        keystore.import_private_key(&oversized, b"password", None).unwrap_err(),
        KeystoreError::corrupt_file("payload too large")
    );
    assert!(keystore.list().unwrap().is_empty());
}

fn read_file_v4(dir: &tempfile::TempDir, keystore_id: &str) -> FileV4 {
    serde_json::from_slice(&fs::read(v4_path(dir, keystore_id)).unwrap()).unwrap()
}

fn flip_hex_at(text: &str, index: usize) -> String {
    let mut text = text.to_string();
    let replacement = if &text[index..index + 1] == "0" { "1" } else { "0" };
    text.replace_range(index..index + 1, replacement);
    text
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
