use gem_client::Target;

#[derive(Clone, Debug)]
pub enum ChainflipTarget {
    SwapStatus { tx_hash: String },
}

impl Target for ChainflipTarget {
    fn path(&self) -> String {
        match self {
            Self::SwapStatus { tx_hash } => format!("/v2/swaps/{tx_hash}"),
        }
    }
}
