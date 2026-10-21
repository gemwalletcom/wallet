#![no_main]

use gem_solana::VersionedTransaction;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(transaction) = VersionedTransaction::deserialize_with_version(data) else {
        return;
    };
    let _ = transaction.serialize_message();
    let bytes = transaction.serialize().expect("a decoded transaction serializes");
    assert_eq!(bytes, data, "a decoded transaction serializes to different bytes");
});
