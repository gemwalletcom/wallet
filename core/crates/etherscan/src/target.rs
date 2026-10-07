use gem_client::{Target, build_path_with_query};
use primitives::EVMChain;

pub enum EtherscanTarget {
    GasOracle { chain: EVMChain },
}

impl Target for EtherscanTarget {
    fn path(&self) -> String {
        match self {
            Self::GasOracle { chain } => build_path_with_query("/v2/api", &[("chainid", chain.to_chain().network_id()), ("module", "gastracker"), ("action", "gasoracle")]),
        }
    }
}
