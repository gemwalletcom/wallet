use crate::block_explorer::{BlockExplorer, ExplorerInput};

pub struct SwapsXyzScan;

impl SwapsXyzScan {
    pub fn boxed() -> Box<dyn BlockExplorer> {
        Box::new(Self)
    }
}

impl BlockExplorer for SwapsXyzScan {
    fn name(&self) -> String {
        "Swaps.xyz".into()
    }

    fn get_tx_url(&self, transaction_id: &str) -> String {
        format!("https://scan.swaps.xyz/transactions/{transaction_id}")
    }

    fn get_address_url(&self, _address: &str) -> String {
        "https://scan.swaps.xyz/transactions".into()
    }

    fn get_swap_tx_url(&self, input: &ExplorerInput) -> String {
        let hash = url::form_urlencoded::byte_serialize(input.hash.as_bytes()).collect::<String>();
        format!("https://scan.swaps.xyz/transactions?search={hash}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_url() {
        let transaction_id = "0x6331c6eded7cfe4ed578e41a57855102b3fd60b3daa2c4bef992f4f5869856b4";
        assert_eq!(SwapsXyzScan.get_tx_url(transaction_id), format!("https://scan.swaps.xyz/transactions/{transaction_id}"));
    }

    #[test]
    fn test_swap_transaction_url() {
        let hash = "9f79797739b2951cf0b7f549af6b2184101027cb0823f084e39c7ad97388da3c";
        assert_eq!(
            SwapsXyzScan.get_swap_tx_url(&ExplorerInput::from(hash)),
            format!("https://scan.swaps.xyz/transactions?search={hash}")
        );
        assert_eq!(
            SwapsXyzScan.get_swap_tx_url(&ExplorerInput::from("hash+/=")),
            "https://scan.swaps.xyz/transactions?search=hash%2B%2F%3D"
        );
    }
}
