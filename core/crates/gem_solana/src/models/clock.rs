use gem_encoding::borsh::{DecodeError, Reader};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Clock {
    pub slot: u64,
    pub epoch_start_timestamp: i64,
    pub epoch: u64,
    pub leader_schedule_epoch: u64,
    pub unix_timestamp: i64,
}

impl Clock {
    pub fn decode(data: &[u8]) -> Result<Self, DecodeError> {
        Reader::decode(data, |reader| {
            Ok(Self {
                slot: reader.u64()?,
                epoch_start_timestamp: reader.i64()?,
                epoch: reader.u64()?,
                leader_schedule_epoch: reader.u64()?,
                unix_timestamp: reader.i64()?,
            })
        })
    }

    pub fn timestamp(&self) -> u64 {
        u64::try_from(self.unix_timestamp).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gem_encoding::decode_base64;

    #[test]
    fn test_decode_clock() {
        let data = decode_base64("+ZJIGwAAAAB7mtFqAAAAACMEAAAAAAAAJAQAAAAAAAD0b9JqAAAAAA==").unwrap();

        assert_eq!(
            Clock::decode(&data).unwrap(),
            Clock {
                slot: 457_741_049,
                epoch_start_timestamp: 1_792_121_467,
                epoch: 1_059,
                leader_schedule_epoch: 1_060,
                unix_timestamp: 1_792_176_116,
            }
        );
    }
}
