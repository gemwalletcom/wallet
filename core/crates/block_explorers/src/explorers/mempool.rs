use crate::BlockExplorer;
use crate::explorers::metadata::{Explorer, Metadata};

pub fn new() -> Box<dyn BlockExplorer> {
    Explorer::boxed(Metadata::new("Mempool", "https://mempool.space"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mempool_urls() {
        assert_eq!(
            new().get_tx_url("813d80363c09b1c4d3f0c6ce3382a048b320edefb573a8aedbc7ddd4c65cf7e4"),
            "https://mempool.space/tx/813d80363c09b1c4d3f0c6ce3382a048b320edefb573a8aedbc7ddd4c65cf7e4"
        );
    }
}
