#![no_main]

use gem_zcash::ValueBalanceReader;
use libfuzzer_sys::fuzz_target;
use primitives::hex;

fuzz_target!(|data: &[u8]| {
    let _ = ValueBalanceReader::read(&hex::encode(data));
});
