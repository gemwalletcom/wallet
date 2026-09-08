use gem_client::Target;

pub enum DexScreenerTarget {
    TokenPairs { chain: String, address: String },
}

impl Target for DexScreenerTarget {
    fn path(&self) -> String {
        match self {
            Self::TokenPairs { chain, address } => format!("/token-pairs/v1/{chain}/{address}"),
        }
    }
}
