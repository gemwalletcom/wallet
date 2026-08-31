use std::fs;

use super::super::types::SecretKind;
use super::testkit::{PHRASE, test_keystore, v4_path};
use crate::v3_testkit::{V3_MNEMONIC_FIXTURE, V3_MNEMONIC_PHRASE, V3_PASSWORD, V3_PRIVATE_KEY_FIXTURE};

#[test]
fn test_import_v3_mnemonic_fixture() {
    let (dir, keystore) = test_keystore();
    let v3_path = dir.path().join("v3.json");
    fs::write(&v3_path, V3_MNEMONIC_FIXTURE).unwrap();
    let meta = keystore.import_v3(&v3_path, V3_PASSWORD, b"new-password", None).unwrap();

    assert_eq!(meta.kind, SecretKind::Mnemonic);
    assert_eq!(keystore.decrypt_mnemonic(&meta.keystore_id, b"new-password").unwrap().as_str(), V3_MNEMONIC_PHRASE);
}

#[test]
fn test_delete_v3() {
    let (dir, keystore) = test_keystore();
    let legacy_id = "d6604f82-9e31-47b3-81db-bab91ab9d72d";
    let named = dir.path().join(legacy_id);
    let suffixed = dir.path().join(format!("UTC--2019-01-01T00-00-00Z--{legacy_id}"));
    let by_content = dir.path().join("walletcore-export");
    let other_wallet = dir.path().join("d4b27ee8-c826-4c5d-9d00-ad82f8269938");
    let unrelated = dir.path().join("shared_prefs.xml");
    let same_id_not_a_keystore = dir.path().join("app_state.json");
    for path in [&named, &suffixed, &by_content] {
        fs::write(path, V3_PRIVATE_KEY_FIXTURE).unwrap();
    }
    fs::write(&other_wallet, V3_MNEMONIC_FIXTURE).unwrap();
    fs::write(&unrelated, V3_PRIVATE_KEY_FIXTURE.replace(legacy_id, "not-a-keystore")).unwrap();
    fs::write(&same_id_not_a_keystore, format!("{{\"id\": \"{legacy_id}\"}}")).unwrap();
    let v4 = keystore.import_mnemonic(PHRASE, b"password", None).unwrap();

    assert!(keystore.delete_v3(legacy_id).unwrap());

    assert!(!named.exists());
    assert!(!suffixed.exists());
    assert!(!by_content.exists());
    assert!(other_wallet.exists());
    assert!(unrelated.exists());
    assert!(same_id_not_a_keystore.exists());
    assert!(v4_path(&dir, &v4.keystore_id).exists());
    assert!(!keystore.delete_v3(legacy_id).unwrap());
    assert!(keystore.delete_v3("").is_err());
}
