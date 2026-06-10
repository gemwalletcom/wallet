use std::fs;

use super::super::types::SecretKind;
use super::testkit::test_keystore;
use crate::v3_testkit::{V3_MNEMONIC_FIXTURE, V3_MNEMONIC_PHRASE, V3_PASSWORD};

#[test]
fn test_import_v3_mnemonic_fixture() {
    let (dir, keystore) = test_keystore();
    let v3_path = dir.path().join("v3.json");
    fs::write(&v3_path, V3_MNEMONIC_FIXTURE).unwrap();
    let meta = keystore.import_v3(&v3_path, V3_PASSWORD, b"new-password", None).unwrap();

    assert_eq!(meta.kind, SecretKind::Mnemonic);
    assert_eq!(keystore.decrypt_mnemonic(&meta.keystore_id, b"new-password").unwrap().as_str(), V3_MNEMONIC_PHRASE);
}
