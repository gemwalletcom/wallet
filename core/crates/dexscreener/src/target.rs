use gem_client::Target;

pub enum DexScreenerTarget {
    TrendingMetas,
    Meta { slug: String },
    TokenPairs { chain: String, address: String },
}

impl Target for DexScreenerTarget {
    fn path(&self) -> String {
        match self {
            Self::TrendingMetas => "/metas/trending/v1".to_string(),
            Self::Meta { slug } => format!("/metas/meta/v1/{slug}"),
            Self::TokenPairs { chain, address } => format!("/token-pairs/v1/{chain}/{address}"),
        }
    }
}
