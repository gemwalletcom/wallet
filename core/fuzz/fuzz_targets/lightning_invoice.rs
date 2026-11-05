#![no_main]

use gem_spark::decode_invoice;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|invoice: &str| {
    let _ = decode_invoice(invoice);
});
