#![no_main]

use gem_spark::provider::LightningTransfer;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|value: &str| {
    if let Ok((transfer, authorization)) = LightningTransfer::decode(value) {
        let (decoded, decoded_authorization) = LightningTransfer::decode(&transfer.encode(&authorization.signature)).unwrap();
        assert_eq!(decoded, transfer);
        assert_eq!(decoded_authorization, authorization);
    }
});
