#![no_main]

use gem_sui::types::{Digest, Transaction, TypeTag};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(transaction) = bcs::from_bytes::<Transaction>(data) {
        let bytes = bcs::to_bytes(&transaction).expect("a decoded transaction serializes");
        assert_eq!(bytes, data, "a decoded transaction serializes to different bytes");
    }
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };
    if let Ok(type_tag) = text.parse::<TypeTag>() {
        assert_eq!(type_tag.to_string().parse::<TypeTag>().ok(), Some(type_tag), "a parsed type tag prints to a different type");
    }
    if let Ok(digest) = text.parse::<Digest>() {
        assert_eq!(digest.to_string(), text, "a parsed digest prints to a different string");
    }
});
