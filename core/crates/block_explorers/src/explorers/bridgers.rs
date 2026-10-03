use crate::{BlockExplorer, ExplorerInput};

pub struct BridgersScan;

impl BridgersScan {
    pub fn boxed() -> Box<dyn BlockExplorer> {
        Box::new(Self)
    }
}

impl BlockExplorer for BridgersScan {
    fn name(&self) -> String {
        "Bridgers".into()
    }

    fn get_tx_url(&self, transaction_id: &str) -> String {
        format!("https://explorer.bridgers.xyz/#/search/{transaction_id}")
    }

    fn get_address_url(&self, address: &str) -> String {
        format!("https://explorer.bridgers.xyz/#/search/{address}")
    }

    fn get_swap_tx_url(&self, input: &ExplorerInput) -> String {
        self.get_tx_url(&input.hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swap_transaction_url() {
        let hash = "b50ddc01b1006266e4c8efe0a64a609201cebdf0fbcb5f6d59748c66f08a14b0";
        assert_eq!(BridgersScan.get_swap_tx_url(&ExplorerInput::from(hash)), format!("https://explorer.bridgers.xyz/#/search/{hash}"));
    }
}
