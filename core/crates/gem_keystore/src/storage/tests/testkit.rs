use std::fs;
use std::path::{Path, PathBuf};

use tempfile::TempDir;

use crate::{KeystoreError, KeystoreId};

use super::super::types::{FileKeystore, KdfParams};

pub(super) use crate::testkit::ABANDON_PHRASE as PHRASE;

pub(super) fn test_keystore() -> (TempDir, FileKeystore) {
    let dir = TempDir::new().unwrap();
    let keystore = FileKeystore::open_with_kdf(dir.path().to_path_buf(), KdfParams::mock()).unwrap();
    (dir, keystore)
}

pub(super) fn v4_path(dir: &TempDir, keystore_id: &str) -> PathBuf {
    dir.path().join("v4").join(format!("{}.gemk", keystore_id))
}

pub(super) fn write_tampered(path: &Path, bytes: &[u8]) {
    fs::write(path, bytes).unwrap();
}

pub(super) fn assert_verify_path_error(path: &Path, password: &[u8], expected: KeystoreError) {
    assert_eq!(FileKeystore::verify_path(path, password).unwrap_err(), expected);
}

pub(super) fn new_keystore_id() -> String {
    KeystoreId::new().to_string()
}
