use curve25519_dalek::edwards::CompressedEdwardsY;
use sha2::{Digest, Sha256};

use crate::{
    error::{Result, SolanaError},
    types::Pubkey,
};

const MAX_SEEDS: usize = 16;
const MAX_SEED_LENGTH: usize = 32;
const PROGRAM_DERIVED_ADDRESS_MARKER: &[u8] = b"ProgramDerivedAddress";

pub fn find_program_address(program_id: &Pubkey, seeds: &[&[u8]]) -> Result<(Pubkey, u8)> {
    validate_seeds(seeds, true)?;

    for bump in (1..=u8::MAX).rev() {
        let bump_seed = [bump];
        let mut seeds_with_bump = Vec::with_capacity(seeds.len().saturating_add(1));
        seeds_with_bump.extend_from_slice(seeds);
        seeds_with_bump.push(&bump_seed);

        if let Ok(address) = create_program_address(program_id, &seeds_with_bump) {
            return Ok((address, bump));
        }
    }

    Err(SolanaError::invalid_input("Unable to find a valid Solana program address"))
}

fn create_program_address(program_id: &Pubkey, seeds: &[&[u8]]) -> Result<Pubkey> {
    validate_seeds(seeds, false)?;

    let mut hasher = Sha256::new();
    for seed in seeds {
        hasher.update(seed);
    }
    hasher.update(program_id.as_bytes());
    hasher.update(PROGRAM_DERIVED_ADDRESS_MARKER);

    let address = Pubkey::new(hasher.finalize().into());
    if is_on_curve(address.as_bytes()) {
        return Err(SolanaError::invalid_input("Solana program address must be off curve"));
    }

    Ok(address)
}

fn validate_seeds(seeds: &[&[u8]], reserve_bump: bool) -> Result<()> {
    let maximum_seed_count = if reserve_bump { MAX_SEEDS.saturating_sub(1) } else { MAX_SEEDS };
    if seeds.len() > maximum_seed_count {
        return Err(SolanaError::invalid_input(format!("Too many Solana program address seeds: {}, maximum: {maximum_seed_count}", seeds.len())));
    }
    if let Some(seed) = seeds.iter().find(|seed| seed.len() > MAX_SEED_LENGTH) {
        return Err(SolanaError::invalid_input(format!("Solana program address seed is too long: {}, maximum: {MAX_SEED_LENGTH}", seed.len())));
    }

    Ok(())
}

fn is_on_curve(bytes: &[u8; 32]) -> bool {
    CompressedEdwardsY(*bytes).decompress().is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_program_address_vectors() {
        let program_id = Pubkey::from_str("BPFLoaderUpgradeab1e11111111111111111111111").unwrap();
        let address = create_program_address(&program_id, &[b"", &[1]]).unwrap();
        assert_eq!(address.to_string(), "BwqrghZA2htAcqq8dzP1WDAhTXYTYWj7CHxF5j7TDBAe");

        let unicode_seed = "\u{2609}".as_bytes();
        let address = create_program_address(&program_id, &[unicode_seed, &[0]]).unwrap();
        assert_eq!(address.to_string(), "13yWmRpaTR4r5nAktwLqMpRNr28tnVUZw26rTvPSSB19");

        let system_program = crate::instructions::program_ids::system_program();
        let (address, bump) = find_program_address(&system_program, &[b"test_seed"]).unwrap();
        let bump_seed = [bump];
        assert_eq!(address, create_program_address(&system_program, &[b"test_seed", &bump_seed]).unwrap());
        assert!(!is_on_curve(address.as_bytes()));
    }

    #[test]
    fn test_invalid_seeds_are_rejected() {
        let program_id = crate::instructions::program_ids::system_program();
        let long_seed = [0_u8; MAX_SEED_LENGTH + 1];
        assert!(find_program_address(&program_id, &[&long_seed]).is_err());

        let seed = [0_u8; 1];
        let too_many_seeds = vec![seed.as_slice(); MAX_SEEDS];
        assert!(find_program_address(&program_id, &too_many_seeds).is_err());
    }
}
