use gem_encoding::borsh::{BorshDeserialize, BorshSerialize};
use gem_hash::sha2::sha256;

use crate::{AccountMeta, Instruction, Pubkey, Result as SolanaResult, SolanaError};

const DISCRIMINATOR_LEN: usize = 8;

fn namespaced_discriminator(namespace: &str, name: &str) -> [u8; 8] {
    let hash = sha256(format!("{namespace}:{name}").as_bytes());
    let mut data = [0u8; 8];
    data.copy_from_slice(&hash[..8]);
    data
}

pub fn global_discriminator(name: &str) -> [u8; 8] {
    namespaced_discriminator("global", name)
}

pub fn instruction_data(name: &str, args: &impl BorshSerialize) -> Vec<u8> {
    [global_discriminator(name).to_vec(), args.to_vec()].concat()
}

pub fn instruction(program_id: Pubkey, name: &str, args: &impl BorshSerialize, accounts: Vec<AccountMeta>) -> Instruction {
    Instruction {
        program_id,
        accounts,
        data: instruction_data(name, args),
    }
}

pub trait AnchorAccount: BorshDeserialize {
    const NAME: &'static str;

    fn decode(data: &[u8]) -> SolanaResult<Self> {
        Self::deserialize(&mut account_body(data, Self::NAME)?).map_err(|error| SolanaError::invalid_input(format!("Invalid {} account: {error}", Self::NAME)))
    }
}

pub fn instruction_arguments<T: BorshDeserialize>(data: &[u8], name: &str) -> SolanaResult<T> {
    let mut arguments = discriminated_body(data, global_discriminator(name)).ok_or_else(|| SolanaError::invalid_input(format!("Invalid {name} instruction")))?;
    T::deserialize(&mut arguments).map_err(|error| SolanaError::invalid_input(format!("Invalid {name} instruction: {error}")))
}

fn account_body<'a>(data: &'a [u8], name: &str) -> SolanaResult<&'a [u8]> {
    discriminated_body(data, namespaced_discriminator("account", name)).ok_or_else(|| SolanaError::invalid_input(format!("Invalid {name} account")))
}

fn discriminated_body(data: &[u8], discriminator: [u8; DISCRIMINATOR_LEN]) -> Option<&[u8]> {
    data.split_at_checked(DISCRIMINATOR_LEN).filter(|(prefix, _)| *prefix == discriminator).map(|(_, body)| body)
}

#[cfg(test)]
mod tests {
    use primitives::hex;

    use super::*;
    use gem_encoding::borsh::{DecodeError, Reader};
    use primitives::decode_hex_literal;

    #[test]
    fn test_global_discriminator() {
        assert_eq!(global_discriminator("init_order"), decode_hex_literal("204c290c27a284db"));
    }

    #[derive(Debug, PartialEq)]
    struct Whirlpool {
        value: u16,
    }

    impl BorshDeserialize for Whirlpool {
        fn deserialize_reader(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
            Ok(Self { value: reader.read()? })
        }
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
    fn test_instruction_arguments() {
        let data = [global_discriminator("swap").as_slice(), &7u64.to_le_bytes(), &[1]].concat();

        assert_eq!(instruction_arguments::<u64>(&data, "swap"), Ok(7));
        assert_eq!(instruction_arguments::<u64>(&data, "route"), Err(SolanaError::invalid_input("Invalid route instruction")));
        assert_eq!(instruction_arguments::<u64>(&data[..12], "swap"), Err(SolanaError::invalid_input("Invalid swap instruction: Unexpected length of input")));
    }

    #[test]
    fn test_account_body() {
        assert_eq!(account_body(&hex::decode("3f95d10ce18063090102").unwrap(), "Whirlpool"), Ok([1, 2].as_slice()));
        assert_eq!(account_body(&hex::decode("3f95d10ce18063090102").unwrap(), "TickArray"), Err(SolanaError::invalid_input("Invalid TickArray account")));
        assert_eq!(account_body(&hex::decode("3f95d10c").unwrap(), "Whirlpool"), Err(SolanaError::invalid_input("Invalid Whirlpool account")));
    }
}
