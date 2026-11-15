#![no_main]

use std::sync::{Arc, LazyLock};

use gemstone::keystore::{GemImportType, GemKeystore};
use gemstone::services::wallet::{GemWalletImportKind, GemWalletImportType, rules::import_request};
use libfuzzer_sys::fuzz_target;

static KEYSTORE: LazyLock<Arc<GemKeystore>> = LazyLock::new(|| {
    let directory = std::env::temp_dir().join(format!("gem-wallet-import-fuzz-{}", std::process::id()));
    GemKeystore::new(directory.to_string_lossy().into_owned()).expect("opens the keystore directory")
});

fuzz_target!(|input: (u8, u8, &str)| {
    let (kind, chain, text) = input;
    let chain = core_fuzz::chain(chain);
    let (kind, chain) = match kind % 4 {
        0 => (GemWalletImportKind::Phrase, None),
        1 => (GemWalletImportKind::Phrase, Some(chain)),
        2 => (GemWalletImportKind::PrivateKey, Some(chain)),
        _ => (GemWalletImportKind::Address, Some(chain)),
    };
    let Ok(import) = import_request(kind, chain, text, None).and_then(GemWalletImportType::validated) else {
        return;
    };
    let import = match import {
        GemWalletImportType::MulticoinPhrase { words, chains } => GemImportType::MulticoinPhrase { words, chains },
        GemWalletImportType::SinglePhrase { words, chain } => GemImportType::SinglePhrase { words, chain },
        GemWalletImportType::PrivateKey { value, chain } => GemImportType::PrivateKey { value, chain },
        GemWalletImportType::Address { .. } => return,
    };
    KEYSTORE.preview_import(import).expect("a validated import derives its wallet");
});
