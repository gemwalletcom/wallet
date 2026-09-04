use gem_client::Target;

#[derive(Clone, Debug)]
pub enum OpenSeaTarget {
    AccountNfts { chain: &'static str, address: String, limit: usize },
    Contract { chain: &'static str, address: String },
    Nft { chain: &'static str, address: String, token_id: String },
    Collection { slug: String },
}

impl Target for OpenSeaTarget {
    fn path(&self) -> String {
        match self {
            Self::AccountNfts { chain, address, limit } => format!("/api/v2/chain/{chain}/account/{address}/nfts?limit={limit}"),
            Self::Contract { chain, address } => format!("/api/v2/chain/{chain}/contract/{address}"),
            Self::Nft { chain, address, token_id } => format!("/api/v2/chain/{chain}/contract/{address}/nfts/{token_id}"),
            Self::Collection { slug } => format!("/api/v2/collections/{slug}"),
        }
    }
}
