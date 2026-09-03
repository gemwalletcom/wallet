use std::error::Error;
use std::str::FromStr;

use gem_hash::sha2::sha256;
use gem_solana::{Pubkey, find_program_address};
use primitives::contract_constants::{SOLANA_ALLDOMAINS_ANS_PROGRAM_ID, SOLANA_ALLDOMAINS_NAME_HOUSE_PROGRAM_ID, SOLANA_ALLDOMAINS_TLD_HOUSE_PROGRAM_ID};

const HASH_PREFIX: &str = "ALT Name Service";
const TLD_HOUSE_PREFIX: &str = "tld_house";
const NAME_HOUSE_PREFIX: &str = "name_house";
const NFT_RECORD_PREFIX: &str = "nft_record";

pub fn name_account_key(name: &str, parent: &Pubkey) -> Result<Pubkey, Box<dyn Error + Send + Sync>> {
    let hashed_name = sha256(&[HASH_PREFIX.as_bytes(), name.as_bytes()].concat());
    let name_class = Pubkey::new([0u8; 32]);
    let program_id = Pubkey::from_str(SOLANA_ALLDOMAINS_ANS_PROGRAM_ID)?;
    let (key, _) = find_program_address(&program_id, &[hashed_name.as_ref(), name_class.as_bytes().as_ref(), parent.as_bytes().as_ref()])?;
    Ok(key)
}

pub fn tld_house_key(tld: &str) -> Result<Pubkey, Box<dyn Error + Send + Sync>> {
    let tld = tld.to_lowercase();
    let program_id = Pubkey::from_str(SOLANA_ALLDOMAINS_TLD_HOUSE_PROGRAM_ID)?;
    let (key, _) = find_program_address(&program_id, &[TLD_HOUSE_PREFIX.as_bytes(), tld.as_bytes()])?;
    Ok(key)
}

pub fn nft_record_key(name_account: &Pubkey, tld_house: &Pubkey) -> Result<Pubkey, Box<dyn Error + Send + Sync>> {
    let program_id = Pubkey::from_str(SOLANA_ALLDOMAINS_NAME_HOUSE_PROGRAM_ID)?;
    let (name_house, _) = find_program_address(&program_id, &[NAME_HOUSE_PREFIX.as_bytes(), tld_house.as_bytes().as_ref()])?;
    let (key, _) = find_program_address(
        &program_id,
        &[NFT_RECORD_PREFIX.as_bytes(), name_house.as_bytes().as_ref(), name_account.as_bytes().as_ref()],
    )?;
    Ok(key)
}
