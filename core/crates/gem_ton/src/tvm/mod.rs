mod bag;
mod builder;
mod cell;
mod error;
mod header;
mod indexed_cell;
mod raw_cell;
mod reader;
mod writer;

pub use bag::BagOfCells;
pub use builder::CellBuilder;
pub use cell::{Cell, CellArc};
pub use error::TvmError;
pub use reader::BitReader;
pub use writer::BitWriter;

fn invalid(msg: &'static str) -> TvmError {
    TvmError::new(msg)
}

pub(crate) fn crc32(bytes: &[u8], reflected_polynomial: u32) -> u32 {
    !bytes
        .iter()
        .fold(!0, |crc, &byte| (0..8).fold(crc ^ u32::from(byte), |crc, _| if crc & 1 == 0 { crc >> 1 } else { (crc >> 1) ^ reflected_polynomial }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crc32() {
        assert_eq!(crc32(b"123456789", 0xEDB8_8320), 0xCBF4_3926);
        assert_eq!(crc32(b"123456789", 0x82F6_3B78), 0xE306_9283);
        assert_eq!(crc32(&[], 0x82F6_3B78), 0);
    }
}
