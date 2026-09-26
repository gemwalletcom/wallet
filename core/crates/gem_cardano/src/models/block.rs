use serde::{Deserialize, Serialize};
use serde_serializers::deserialize_u64_from_str;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    pub number: u64,
    #[serde(deserialize_with = "deserialize_u64_from_str")]
    pub slot_no: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockData {
    pub cardano: BlockTip,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockTip {
    pub tip: Block,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenesisData {
    pub genesis: Genesis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Genesis {
    pub shelley: GenesisShelley,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenesisShelley {
    pub network_magic: i32,
}
