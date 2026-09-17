use crate::{AssetId, ScanAddressTarget, ScanTransactionPayload, TransactionType};

impl ScanTransactionPayload {
    pub fn mock_with_assets(origin: AssetId, target: AssetId) -> Self {
        Self {
            origin: ScanAddressTarget {
                asset_id: origin,
                address: "origin".to_string(),
            },
            target: ScanAddressTarget {
                asset_id: target,
                address: "target".to_string(),
            },
            website: None,
            transaction_type: TransactionType::Transfer,
        }
    }
}
