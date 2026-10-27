use crate::BlockExplorer;
use crate::explorers::metadata::{Explorer, Metadata};

pub struct Viewblock;

impl Viewblock {
    pub fn boxed() -> Box<dyn BlockExplorer> {
        Explorer::boxed(Metadata::new("Viewblock", "https://viewblock.io/thorchain"))
    }
}

pub struct RuneScan;

impl RuneScan {
    pub fn boxed() -> Box<dyn BlockExplorer> {
        Box::new(RuneScanExplorer)
    }
}

struct RuneScanExplorer;

impl BlockExplorer for RuneScanExplorer {
    fn name(&self) -> String {
        "RuneScan".into()
    }
    fn get_tx_url(&self, hash: &str) -> String {
        format!("https://runescan.io/tx/{}", hash.trim_start_matches("0x"))
    }
    fn get_address_url(&self, address: &str) -> String {
        format!("https://runescan.io/address/{}", address)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runescan_urls() {
        let explorer = RuneScan::boxed();

        assert_eq!(
            explorer.get_address_url("thor166n4w5039meulfa3p6ydg60ve6ueac7tlt0jws"),
            "https://runescan.io/address/thor166n4w5039meulfa3p6ydg60ve6ueac7tlt0jws"
        );
        assert_eq!(
            explorer.get_tx_url("FF82C517ECFDCA71A6CD3501063D76995C67509B2AFC012D2BCE61C130C05E98"),
            "https://runescan.io/tx/FF82C517ECFDCA71A6CD3501063D76995C67509B2AFC012D2BCE61C130C05E98"
        );
    }
}
