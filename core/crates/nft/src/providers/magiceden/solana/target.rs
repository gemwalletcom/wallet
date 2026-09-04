use gem_client::Target;

#[derive(Clone, Debug)]
pub enum MagicEdenTarget {
    WalletTokens { address: String },
    Collection { id: String },
    Token { mint: String },
}

impl Target for MagicEdenTarget {
    fn path(&self) -> String {
        match self {
            Self::WalletTokens { address } => format!("/v2/wallets/{address}/tokens"),
            Self::Collection { id } => format!("/collections/{id}"),
            Self::Token { mint } => format!("/v2/tokens/{mint}"),
        }
    }
}
