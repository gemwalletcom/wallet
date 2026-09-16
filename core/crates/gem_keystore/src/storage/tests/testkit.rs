use std::fs;
use std::path::{Path, PathBuf};

use tempfile::TempDir;

use crate::KeystoreError;

use super::super::types::FileKeystore;

pub(super) use crate::testkit::ABANDON_PHRASE as PHRASE;

pub(super) fn v4_path(dir: &TempDir, keystore_id: &str) -> PathBuf {
    dir.path().join(format!("{}.json", keystore_id))
}

pub(super) fn write_tampered(path: &Path, bytes: &[u8]) {
    fs::write(path, bytes).unwrap();
}

pub(super) fn assert_verify_path_error(path: &Path, password: &[u8], expected: KeystoreError) {
    assert_eq!(FileKeystore::verify_path(path, password).unwrap_err(), expected);
}
