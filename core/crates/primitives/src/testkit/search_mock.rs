use crate::AssetList;

impl AssetList {
    pub fn mock(count: u32) -> Self {
        Self {
            id: "stablecoins".to_string(),
            name: "Stablecoins".to_string(),
            count,
        }
    }
}
