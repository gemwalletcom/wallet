use gem_client::{Target, build_path_with_query};
use primitives::EVMChain;

const FIRST_PAGE: &str = "1";
const SORT_DESC: &str = "desc";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EtherscanApi {
    Etherscan,
    ZkSync,
}

impl EtherscanApi {
    fn path(&self) -> &'static str {
        match self {
            Self::Etherscan => "/v2/api",
            Self::ZkSync => "/api",
        }
    }
}

pub enum EtherscanTarget {
    GasOracle { api: EtherscanApi, chain: EVMChain },
    Transactions { api: EtherscanApi, chain: EVMChain, address: String, limit: usize },
    TokenTransfers { api: EtherscanApi, chain: EVMChain, address: String, limit: usize },
}

impl EtherscanTarget {
    fn account_path(api: &EtherscanApi, chain: &EVMChain, action: &str, address: &str, limit: usize) -> String {
        let limit = limit.to_string();
        build_path_with_query(
            api.path(),
            &[
                ("chainid", chain.to_chain().network_id()),
                ("module", "account"),
                ("action", action),
                ("address", address),
                ("page", FIRST_PAGE),
                ("offset", &limit),
                ("sort", SORT_DESC),
            ],
        )
    }
}

impl Target for EtherscanTarget {
    fn path(&self) -> String {
        match self {
            Self::GasOracle { api, chain } => build_path_with_query(api.path(), &[("chainid", chain.to_chain().network_id()), ("module", "gastracker"), ("action", "gasoracle")]),
            Self::Transactions { api, chain, address, limit } => Self::account_path(api, chain, "txlist", address, *limit),
            Self::TokenTransfers { api, chain, address, limit } => Self::account_path(api, chain, "tokentx", address, *limit),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path() {
        let target = |api| EtherscanTarget::Transactions {
            api,
            chain: EVMChain::Blast,
            address: "0x123".to_string(),
            limit: 3,
        };

        assert_eq!(target(EtherscanApi::Etherscan).path(), "/v2/api?chainid=81457&module=account&action=txlist&address=0x123&page=1&offset=3&sort=desc");
        assert_eq!(target(EtherscanApi::ZkSync).path(), "/api?chainid=81457&module=account&action=txlist&address=0x123&page=1&offset=3&sort=desc");
    }
}
