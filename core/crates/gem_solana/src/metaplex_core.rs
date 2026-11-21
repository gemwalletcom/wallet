use crate::Pubkey;
use gem_encoding::{
    borsh::{BorshDeserialize, DecodeError, Reader},
    decode_base64,
};

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum Key {
    Uninitialized,
    AssetV1,
    HashedAssetV1,
    PluginHeaderV1,
    PluginRegistryV1,
    CollectionV1,
}

impl BorshDeserialize for Key {
    fn deserialize_reader(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        match reader.read::<u8>()? {
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

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum UpdateAuthority {
    None,
    Address(Pubkey),
    Collection(Pubkey),
}

impl BorshDeserialize for UpdateAuthority {
    fn deserialize_reader(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        match reader.read::<u8>()? {
            0 => Ok(Self::None),
            1 => Ok(Self::Address(reader.read()?)),
            2 => Ok(Self::Collection(reader.read()?)),
            tag => Err(DecodeError::UnexpectedTag(tag)),
        }
    }
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

impl BorshDeserialize for AssetV1 {
    fn deserialize_reader(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            key: reader.read()?,
            owner: reader.read()?,
            update_authority: reader.read()?,
            name: reader.read()?,
            uri: reader.read()?,
            seq: reader.read()?,
        })
    }
}

impl AssetV1 {
    pub fn collection(&self) -> Option<Pubkey> {
        match self.update_authority {
            UpdateAuthority::Collection(pubkey) => Some(pubkey),
            UpdateAuthority::Address(_) | UpdateAuthority::None => None,
        }
    }
}

pub fn decode_asset(base64_data: &str) -> Result<AssetV1, Box<dyn std::error::Error + Send + Sync>> {
    let data = decode_base64(base64_data).map_err(|error| format!("decode core asset base64: {error}"))?;
    let asset = AssetV1::deserialize(&mut data.as_slice()).map_err(|error| format!("decode core asset: {error}"))?;
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
