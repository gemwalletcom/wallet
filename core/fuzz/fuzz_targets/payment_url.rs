#![no_main]

use libfuzzer_sys::fuzz_target;
use payment::{PaymentURLDecoder, classify_url};
use primitives::WalletConnectLink;

fuzz_target!(|url: &str| {
    let _ = PaymentURLDecoder::decode(url);
    let _ = classify_url(url);
    let _ = WalletConnectLink::from_url(url);
});
