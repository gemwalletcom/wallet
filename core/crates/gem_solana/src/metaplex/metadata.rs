use std::str::FromStr;

use crate::metaplex::{
    Key, TokenStandard,
    collection::{Collection, CollectionDetails},
    data::Data,
    uses::Uses,
};
use crate::{METAPLEX_PROGRAM, Pubkey, find_program_address};
use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Clone, BorshDeserialize, BorshSerialize, Debug, PartialEq, Eq)]
pub struct Metadata {
    pub key: Key,
    pub update_authority: Pubkey,
    pub mint: Pubkey,
    pub data: Data,
    pub primary_sale_happened: bool,
    pub is_mutable: bool,
    pub edition_nonce: Option<u8>,
    pub token_standard: Option<TokenStandard>,
    pub collection: Option<Collection>,
    pub uses: Option<Uses>,
    pub collection_details: Option<CollectionDetails>,
    pub programmable_config: Option<ProgrammableConfig>,
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub enum ProgrammableConfig {
    V1 { rule_set: Option<Pubkey> },
}

impl Metadata {
    pub fn find_pda(mint: Pubkey) -> Option<(Pubkey, u8)> {
        let mpl_id = Pubkey::from_str(METAPLEX_PROGRAM).ok()?;
        find_program_address(&mpl_id, &["metadata".as_bytes(), mpl_id.as_bytes().as_ref(), mint.as_bytes().as_ref()]).ok()
    }

    pub fn find_master_edition_pda(mint: Pubkey) -> Option<(Pubkey, u8)> {
        let mpl_id = Pubkey::from_str(METAPLEX_PROGRAM).ok()?;
        find_program_address(&mpl_id, &["metadata".as_bytes(), mpl_id.as_bytes().as_ref(), mint.as_bytes().as_ref(), "edition".as_bytes()]).ok()
    }

    pub fn find_token_record_pda(mint: Pubkey, token_account: Pubkey) -> Option<(Pubkey, u8)> {
        let mpl_id = Pubkey::from_str(METAPLEX_PROGRAM).ok()?;
        find_program_address(
            &mpl_id,
            &["metadata".as_bytes(), mpl_id.as_bytes().as_ref(), mint.as_bytes().as_ref(), "token_record".as_bytes(), token_account.as_bytes().as_ref()],
        )
        .ok()
    }

    pub fn is_programmable(&self) -> bool {
        #[allow(clippy::match_like_matches_macro)]
        match self.token_standard {
            Some(TokenStandard::ProgrammableNonFungible | TokenStandard::ProgrammableNonFungibleEdition) => true,
            _ => false,
        }
    }

    pub fn rule_set(&self) -> Option<Pubkey> {
        match self.programmable_config {
            Some(ProgrammableConfig::V1 { rule_set: Some(pubkey) }) => Some(pubkey),
            _ => None,
        }
    }
}
