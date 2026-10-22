#![no_main]

use std::{fs, path::PathBuf, sync::LazyLock};

use gem_keystore::{FileKeystore, Keystore, KeystoreId};
use libfuzzer_sys::fuzz_target;

const PASSWORD: &[u8] = b"password";

static DIRECTORY: LazyLock<PathBuf> = LazyLock::new(|| std::env::temp_dir().join(format!("gem-keystore-fuzz-{}", std::process::id())));
static KEYSTORE: LazyLock<FileKeystore> = LazyLock::new(|| FileKeystore::open(DIRECTORY.clone()).expect("opens the keystore directory"));
static KEYSTORE_ID: LazyLock<String> = LazyLock::new(|| KeystoreId::from_wallet_id("fuzz").into_string());

fuzz_target!(|input: (bool, &[u8])| {
    let (is_v3, file) = input;
    let keystore = &*KEYSTORE;
    if is_v3 {
        let path = DIRECTORY.join("v3.input");
        fs::write(&path, file).expect("writes the v3 file");
        let _ = keystore.import_v3(&path, PASSWORD, PASSWORD, None);
    } else {
        fs::write(DIRECTORY.join(format!("{}.json", *KEYSTORE_ID)), file).expect("writes the v4 file");
        let _ = keystore.list();
        let _ = keystore.verify(&KEYSTORE_ID, PASSWORD);
    }
});
