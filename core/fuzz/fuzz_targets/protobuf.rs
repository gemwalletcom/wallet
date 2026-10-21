#![no_main]

use gem_encoding::protobuf::{decode_grpc_frame, visit_fields};
use libfuzzer_sys::fuzz_target;

const MAX_DEPTH: usize = 16;

fn visit(data: &[u8], depth: usize) {
    let _ = visit_fields(data, |field| {
        let _ = field.varint();
        let _ = field.fixed64();
        let _ = field.string();
        if let Ok(bytes) = field.bytes()
            && depth < MAX_DEPTH
        {
            visit(bytes, depth + 1);
        }
        Ok(())
    });
}

fuzz_target!(|data: &[u8]| {
    visit(data, 0);
    if let Ok(payload) = decode_grpc_frame(data) {
        visit(payload, 0);
    }
});
