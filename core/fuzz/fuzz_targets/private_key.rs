#![no_main]

use libfuzzer_sys::fuzz_target;
use signer::{decode_private_key, encode_private_key};

fuzz_target!(|input: (u8, &str)| {
    let (chain, value) = input;
    let chain = core_fuzz::chain(chain);
    let Ok(private_key) = decode_private_key(&chain, value) else {
        return;
    };
    let Ok(exported) = encode_private_key(&chain, &private_key) else {
        return;
    };
    assert_eq!(*decode_private_key(&chain, &exported).expect("an exported private key imports"), *private_key);
});
