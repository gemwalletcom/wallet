use crate::block_explorer::BlockExplorer;

#[derive(Default)]
pub struct AcrossScan;

impl AcrossScan {
    pub fn boxed() -> Box<dyn BlockExplorer> {
        Box::<Self>::default()
    }
}

impl BlockExplorer for AcrossScan {
    fn name(&self) -> String {
        "Across".into()
    }

    fn get_tx_url(&self, hash: &str) -> String {
        format!("https://across.to/transfer/{hash}")
    }

    fn get_address_url(&self, address: &str) -> String {
        format!("https://across.to/transfer/{address}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_across_scan() {
        let explorer = AcrossScan::boxed();
        let transaction_hash = "0xec0e05178bb2e7a13131c86ef5e4891e116cfe8757d63d7337998f848c63d9af";

        assert_eq!(explorer.name(), "Across");
        assert_eq!(
            explorer.get_tx_url(transaction_hash),
            "https://across.to/transfer/0xec0e05178bb2e7a13131c86ef5e4891e116cfe8757d63d7337998f848c63d9af"
        );
    }
}
