#![no_main]

use gem_ton::tvm::BagOfCells;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(bag) = BagOfCells::parse(data) else {
        return;
    };
    let bytes = bag.serialize(false).expect("a parsed BoC serializes");
    let reparsed = BagOfCells::parse(&bytes).expect("a serialized BoC parses");
    assert_eq!(reparsed.serialize(false).expect("a reparsed BoC serializes"), bytes, "a BoC changes after one round trip");
});
