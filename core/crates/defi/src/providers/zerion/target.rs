use gem_client::{Target, build_path_with_query};

use super::model::PositionsQuery;

#[derive(Clone, Debug)]
pub enum ZerionTarget {
    WalletPositions { address: String, query: PositionsQuery },
}

impl Target for ZerionTarget {
    fn path(&self) -> String {
        match self {
            Self::WalletPositions { address, query } => build_path_with_query(&format!("/v1/wallets/{address}/positions/"), query),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path() {
        assert_eq!(
            ZerionTarget::WalletPositions {
                address: "0x1".into(),
                query: PositionsQuery::complex("xdai")
            }
            .path(),
            "/v1/wallets/0x1/positions/?filter%5Bpositions%5D=only_complex&filter%5Bchain_ids%5D=xdai&filter%5Btrash%5D=only_non_trash&currency=usd&sort=-value"
        );
    }
}
