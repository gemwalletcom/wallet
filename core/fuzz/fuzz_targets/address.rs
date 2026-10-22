#![no_main]

use gemstone::address::validate_address;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|input: (u8, &str)| {
    let (chain, address) = input;
    validate_address(address, core_fuzz::chain(chain));
});
