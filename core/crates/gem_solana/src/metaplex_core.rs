use gem_encoding::{
    borsh::{DecodeError, Reader},
    decode_base64,
};

use crate::Pubkey;

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum Key {
    Uninitialized,
    AssetV1,
    HashedAssetV1,
    PluginHeaderV1,
    PluginRegistryV1,
    CollectionV1,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum UpdateAuthority {
    None,
    Address(Pubkey),
    Collection(Pubkey),
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct AssetV1 {
    pub key: Key,
    pub owner: Pubkey,
    pub update_authority: UpdateAuthority,
    pub name: String,
    pub uri: String,
    pub seq: Option<u64>,
}

impl Key {
    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        match reader.u8()? {
            0 => Ok(Self::Uninitialized),
            1 => Ok(Self::AssetV1),
            2 => Ok(Self::HashedAssetV1),
            3 => Ok(Self::PluginHeaderV1),
            4 => Ok(Self::PluginRegistryV1),
            5 => Ok(Self::CollectionV1),
            tag => Err(DecodeError::UnexpectedTag(tag)),
        }
    }
}

impl UpdateAuthority {
    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        match reader.u8()? {
            0 => Ok(Self::None),
            1 => Ok(Self::Address(Pubkey::read(reader)?)),
            2 => Ok(Self::Collection(Pubkey::read(reader)?)),
            tag => Err(DecodeError::UnexpectedTag(tag)),
        }
    }
}

impl AssetV1 {
    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            key: Key::read(reader)?,
            owner: Pubkey::read(reader)?,
            update_authority: UpdateAuthority::read(reader)?,
            name: reader.string()?,
            uri: reader.string()?,
            seq: reader.option(Reader::u64)?,
        })
    }

    pub fn collection(&self) -> Option<Pubkey> {
        match self.update_authority {
            UpdateAuthority::Collection(pubkey) => Some(pubkey),
            UpdateAuthority::Address(_) | UpdateAuthority::None => None,
        }
    }
}

pub fn decode_asset(base64_data: &str) -> Result<AssetV1, Box<dyn std::error::Error + Send + Sync>> {
    let data = decode_base64(base64_data).map_err(|error| format!("decode core asset base64: {error}"))?;
    let asset = Reader::decode_prefix(&data, AssetV1::read).map_err(|error| format!("decode core asset: {error}"))?;
    if asset.key != Key::AssetV1 {
        return Err(format!("unexpected Metaplex Core asset key: {:?}", asset.key).into());
    }
    Ok(asset)
}

#[cfg(test)]
mod tests {
    use super::*;

    const BWED_1545_ASSET: &str = "AdQCoT0whdkMOHS+JayL1er4PFXv7D/dY+IJ+r2UN6wYAkeTzcRWuwoVceHlkHr4rEWE2hX58AjYjrPZJGJKtkYICgAAAEJXRUQgIzE1NDVbAAAAaHR0cHM6Ly9iYWZ5YmVpZm91dzRjdnF6a21pNzN3ZWQzM2lsM3l5cTdnam9xbW91emFvZGI3Z2J4YmNiZm94cXd1cS5pcGZzLnczcy5saW5rLzE1NDUuanNvbgA=";

    #[test]
    fn test_decode_asset() {
        let asset = decode_asset(BWED_1545_ASSET).unwrap();
        assert_eq!(asset.key, Key::AssetV1);
        assert_eq!(asset.owner.to_base58(), "FGbkx8rYTPJubjyScReeps6GA83D1nSmFr3BrN7buokb");
        assert_eq!(asset.collection().unwrap().to_base58(), "5pQfZttNUtaj8sySRY9RsdtB81aEAQDh2vnacpxiwTpT");
        assert_eq!(asset.name, "BWED #1545");
        assert!(asset.uri.contains("1545.json"));
        assert_eq!(asset.seq, None);

        assert!(decode_asset("Y29ycnVwdA==").is_err());
    }
}
