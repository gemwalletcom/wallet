use std::error::Error;
use std::ops::Range;
use std::time::{SystemTime, UNIX_EPOCH};

use gem_solana::Pubkey;

const DISCRIMINATOR: [u8; 8] = [68, 72, 88, 44, 15, 167, 103, 243];
const DISCRIMINATOR_RANGE: Range<usize> = 0..8;
const OWNER_RANGE: Range<usize> = 40..72;
const EXPIRES_AT_RANGE: Range<usize> = 104..112;
const HEADER_SIZE: usize = 200;
const GRACE_PERIOD_SECONDS: u64 = 45 * 24 * 60 * 60;

#[derive(Debug)]
pub struct NameRecord {
    pub owner: Pubkey,
    pub expires_at: u64,
}

impl NameRecord {
    pub fn from_account_data(data: &[u8]) -> Result<Self, Box<dyn Error + Send + Sync>> {
        if data.len() < HEADER_SIZE {
            return Err(format!("invalid name record length: {}", data.len()).into());
        }
        if data[DISCRIMINATOR_RANGE] != DISCRIMINATOR {
            return Err("invalid name record discriminator".into());
        }
        Ok(Self {
            owner: Pubkey::new(data[OWNER_RANGE].try_into()?),
            expires_at: u64::from_le_bytes(data[EXPIRES_AT_RANGE].try_into()?),
        })
    }

    pub fn is_active(&self) -> bool {
        if self.expires_at == 0 {
            return true;
        }
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        self.expires_at + GRACE_PERIOD_SECONDS > now
    }
}

#[cfg(test)]
mod tests {
    use gem_solana::Pubkey;

    use super::{DISCRIMINATOR, HEADER_SIZE, NameRecord};

    fn account_data(discriminator: [u8; 8], owner: [u8; 32], expires_at: u64) -> Vec<u8> {
        let mut data = vec![0u8; HEADER_SIZE];
        data[0..8].copy_from_slice(&discriminator);
        data[40..72].copy_from_slice(&owner);
        data[104..112].copy_from_slice(&expires_at.to_le_bytes());
        data
    }

    #[test]
    fn test_from_account_data() {
        let owner = [7u8; 32];
        let record = NameRecord::from_account_data(&account_data(DISCRIMINATOR, owner, 42)).unwrap();

        assert_eq!(record.owner, Pubkey::new(owner));
        assert_eq!(record.expires_at, 42);
        assert_eq!(
            NameRecord::from_account_data(&account_data([0u8; 8], owner, 42)).unwrap_err().to_string(),
            "invalid name record discriminator"
        );
        assert_eq!(NameRecord::from_account_data(&[0u8; 10]).unwrap_err().to_string(), "invalid name record length: 10");
    }
}
