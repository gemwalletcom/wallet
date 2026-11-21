mod collection;
mod data;
mod uses;

pub mod metadata;
use crate::metaplex::metadata::Metadata;
use gem_encoding::{
    borsh::{BorshDeserialize, DecodeError, Reader},
    decode_base64,
};

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum Key {
    Uninitialized,
    EditionV1,
    MasterEditionV1,
    ReservationListV1,
    MetadataV1,
    ReservationListV2,
    MasterEditionV2,
    EditionMarker,
    UseAuthorityRecord,
    CollectionAuthorityRecord,
    TokenOwnedEscrow,
    TokenRecord,
    MetadataDelegate,
    EditionMarkerV2,
    HolderDelegate,
}

impl BorshDeserialize for Key {
    fn deserialize_reader(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        match reader.read::<u8>()? {
            0 => Ok(Self::Uninitialized),
            1 => Ok(Self::EditionV1),
            2 => Ok(Self::MasterEditionV1),
            3 => Ok(Self::ReservationListV1),
            4 => Ok(Self::MetadataV1),
            5 => Ok(Self::ReservationListV2),
            6 => Ok(Self::MasterEditionV2),
            7 => Ok(Self::EditionMarker),
            8 => Ok(Self::UseAuthorityRecord),
            9 => Ok(Self::CollectionAuthorityRecord),
            10 => Ok(Self::TokenOwnedEscrow),
            11 => Ok(Self::TokenRecord),
            12 => Ok(Self::MetadataDelegate),
            13 => Ok(Self::EditionMarkerV2),
            14 => Ok(Self::HolderDelegate),
            tag => Err(DecodeError::UnexpectedTag(tag)),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum TokenStandard {
    NonFungible,
    FungibleAsset,
    Fungible,
    NonFungibleEdition,
    ProgrammableNonFungible,
    ProgrammableNonFungibleEdition,
}

impl BorshDeserialize for TokenStandard {
    fn deserialize_reader(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        match reader.read::<u8>()? {
            0 => Ok(Self::NonFungible),
            1 => Ok(Self::FungibleAsset),
            2 => Ok(Self::Fungible),
            3 => Ok(Self::NonFungibleEdition),
            4 => Ok(Self::ProgrammableNonFungible),
            5 => Ok(Self::ProgrammableNonFungibleEdition),
            tag => Err(DecodeError::UnexpectedTag(tag)),
        }
    }
}

pub fn decode_metadata(base64_str: &str) -> Result<Metadata, Box<dyn std::error::Error>> {
    let data = decode_base64(base64_str)?;
    let metadata = Metadata::deserialize(&mut data.as_slice())?;
    Ok(metadata)
}

#[cfg(test)]
mod tests {
    use crate::{
        Pubkey, USDC_TOKEN_MINT,
        metaplex::{Key, decode_metadata, metadata::Metadata},
    };
    use std::str::FromStr;

    #[test]
    fn test_metadata_data() {
        let string = "BBzjWe1aAS4E+hQrnHUaHF6Hz9CgFhuchf/TG3jN/Nj2xvp6877brTo9ZfNqq8l0MbG75MLS9uDkfKYCA0UvXWEgAAAAVVNEIENvaW4AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAKAAAAVVNEQwAAAAAAAMgAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAfwAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==";
        let metadata = decode_metadata(string).unwrap();

        assert_eq!(metadata.key, Key::MetadataV1);
        assert_eq!(metadata.mint.to_string(), USDC_TOKEN_MINT);

        assert_eq!(metadata.data.uri.trim_matches(char::from(0)), "");
        assert_eq!(metadata.data.symbol.trim_matches(char::from(0)), "USDC");
        assert_eq!(metadata.data.name.trim_matches(char::from(0)), "USD Coin");
    }

    #[test]
    fn test_find_pda() {
        let mut mint = Pubkey::from_str(USDC_TOKEN_MINT).unwrap();
        let (pda, _) = Metadata::find_pda(mint).unwrap();

        assert_eq!(pda.to_string(), "5x38Kp4hvdomTCnCrAny4UtMUt5rQBdB6px2K1Ui45Wq");

        mint = Pubkey::from_str("MEW1gQWJ3nEXg2qgERiKu7FAFj79PHvQVREQUzScPP5").unwrap();
        let (pda, _) = Metadata::find_pda(mint).unwrap();

        assert_eq!(pda.to_string(), "5G95zJ9w6ESv7AFWqLKNfbZAoKBADjpVm9MT1cQm8Dpw");
    }
}
