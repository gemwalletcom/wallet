#![no_main]

use gemstone::address::{parse_address, validate_address};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|input: (u8, &str)| {
    let (chain, address) = input;
    let chain = core_fuzz::chain(chain);
    if let Some(parsed) = parse_address(address, chain) {
        assert!(validate_address(&parsed, chain));
        assert_eq!(parse_address(&parsed, chain), Some(parsed));
    }
});
