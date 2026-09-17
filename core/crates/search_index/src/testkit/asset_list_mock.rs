use std::collections::HashMap;

use crate::AssetListDocument;

impl AssetListDocument {
    pub fn mock() -> Self {
        Self::new(
            "stablecoins".to_string(),
            "Stablecoins".to_string(),
            HashMap::from([("ethereum".to_string(), 4), ("solana".to_string(), 2)]),
        )
    }
}
