#![no_main]

use gem_auth::{DeviceSignature, DeviceSignatureHeaders};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|input: (&str, &str, &str, Option<&str>, &str, Option<&str>)| {
    let (signature_input, signature, content_digest, wallet_id, path, query) = input;
    let headers = DeviceSignatureHeaders {
        signature_input,
        signature,
        content_digest,
        wallet_id,
    };
    if let Some(signature) = DeviceSignature::parse(&headers) {
        let _ = signature.verify("GET", path, query);
    }
});
