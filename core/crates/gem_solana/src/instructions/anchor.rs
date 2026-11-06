use borsh::{BorshDeserialize, BorshSerialize};
use sha2::{Digest, Sha256};

use crate::{Result, SolanaError};

const DISCRIMINATOR_LEN: usize = 8;

fn namespaced_discriminator(namespace: &str, name: &str) -> [u8; 8] {
    let mut hasher = Sha256::new();
    hasher.update(namespace.as_bytes());
    hasher.update(b":");
    hasher.update(name.as_bytes());
    let hash = hasher.finalize();
    let mut data = [0u8; 8];
    data.copy_from_slice(&hash[..8]);
    data
}

pub fn global_discriminator(name: &str) -> [u8; 8] {
    namespaced_discriminator("global", name)
}

pub fn instruction_data(name: &str, args: &impl BorshSerialize) -> Vec<u8> {
    [global_discriminator(name).to_vec(), borsh::to_vec(args).unwrap_or_default()].concat()
}

pub trait AnchorAccount: BorshDeserialize {
    const NAME: &'static str;

    fn decode(data: &[u8]) -> Result<Self> {
        Self::deserialize(&mut account_body(data, Self::NAME)?).map_err(|error| SolanaError::invalid_input(format!("Invalid {} account: {error}", Self::NAME)))
    }
}

fn account_body<'a>(data: &'a [u8], name: &str) -> Result<&'a [u8]> {
    data.split_at_checked(DISCRIMINATOR_LEN)
        .filter(|(discriminator, _)| *discriminator == namespaced_discriminator("account", name))
        .map(|(_, body)| body)
        .ok_or_else(|| SolanaError::invalid_input(format!("Invalid {name} account")))
}

#[cfg(test)]
mod tests {
    use primitives::hex;

    use super::*;
    use primitives::decode_hex_literal;

    #[test]
    fn test_global_discriminator() {
        assert_eq!(global_discriminator("init_order"), decode_hex_literal("204c290c27a284db"));
    }

    #[derive(Debug, PartialEq, BorshDeserialize)]
    struct Whirlpool {
        value: u16,
    }

    impl AnchorAccount for Whirlpool {
        const NAME: &'static str = "Whirlpool";
    }

    #[test]
    fn test_decode() {
        assert_eq!(Whirlpool::decode(&hex::decode("3f95d10ce18063090102").unwrap()), Ok(Whirlpool { value: 0x0201 }));
        assert_eq!(
            Whirlpool::decode(&hex::decode("3f95d10ce180630901").unwrap()),
            Err(SolanaError::invalid_input("Invalid Whirlpool account: Unexpected length of input"))
        );
    }

    #[test]
    fn test_account_body() {
        assert_eq!(account_body(&hex::decode("3f95d10ce18063090102").unwrap(), "Whirlpool"), Ok([1, 2].as_slice()));
        assert_eq!(account_body(&hex::decode("3f95d10ce18063090102").unwrap(), "TickArray"), Err(SolanaError::invalid_input("Invalid TickArray account")));
        assert_eq!(account_body(&hex::decode("3f95d10c").unwrap(), "Whirlpool"), Err(SolanaError::invalid_input("Invalid Whirlpool account")));
    }
}
