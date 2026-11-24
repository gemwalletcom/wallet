use gem_encoding::borsh::{BorshDeserialize, DecodeError, Reader};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Clock {
    pub slot: u64,
    pub epoch_start_timestamp: i64,
    pub epoch: u64,
    pub leader_schedule_epoch: u64,
    pub unix_timestamp: i64,
}

impl BorshDeserialize for Clock {
    fn deserialize_reader(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            slot: reader.read()?,
            epoch_start_timestamp: reader.read()?,
            epoch: reader.read()?,
            leader_schedule_epoch: reader.read()?,
            unix_timestamp: reader.read()?,
        })
    }
}

impl Clock {
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
            Clock::try_from_slice(&data).unwrap(),
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
