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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path() {
        assert_eq!(
            OpenSeaTarget::AccountNfts {
                chain: "ethereum",
                address: "0x1".into(),
                limit: 100
            }
            .path(),
            "/api/v2/chain/ethereum/account/0x1/nfts?limit=100"
        );
        assert_eq!(
            OpenSeaTarget::Nft {
                chain: "polygon",
                address: "0x2".into(),
                token_id: "7".into()
            }
            .path(),
            "/api/v2/chain/polygon/contract/0x2/nfts/7"
        );
        assert_eq!(OpenSeaTarget::Collection { slug: "punks".into() }.path(), "/api/v2/collections/punks");
    }
}
