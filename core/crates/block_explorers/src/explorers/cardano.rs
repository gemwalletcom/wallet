use crate::BlockExplorer;
use crate::explorers::metadata::{Explorer, Metadata};

pub struct CardanoScan;

impl CardanoScan {
    pub fn boxed() -> Box<dyn BlockExplorer> {
        Explorer::boxed(Metadata::blockchair("CardanoScan", "https://cardanoscan.io"))
    }
}
