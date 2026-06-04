use crate::block_explorer::BlockExplorer;

pub struct MayaScan;

impl MayaScan {
    pub fn boxed() -> Box<dyn BlockExplorer> {
        Box::new(MayaScanExplorer)
    }
}

struct MayaScanExplorer;

impl BlockExplorer for MayaScanExplorer {
    fn name(&self) -> String {
        "MayaScan".into()
    }
    fn get_tx_url(&self, hash: &str) -> String {
        format!("https://www.mayascan.org/tx/{}", hash.trim_start_matches("0x"))
    }
    fn get_address_url(&self, address: &str) -> String {
        format!("https://www.mayascan.org/address/{}", address)
    }
}
