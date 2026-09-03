use gem_client::{Target, build_path_with_query};

use super::model::OwnedNftsQuery;

#[derive(Clone, Debug)]
pub enum AlchemyNftTarget {
    OwnedNfts { query: OwnedNftsQuery },
    ContractMetadata { contract_address: String },
    NftMetadata { contract_address: String, token_id: String },
}

impl Target for AlchemyNftTarget {
    fn path(&self) -> String {
        match self {
            Self::OwnedNfts { query } => build_path_with_query("/getNFTsForOwner", query),
            Self::ContractMetadata { contract_address } => build_path_with_query("/getContractMetadata", &[("contractAddress", contract_address)]),
            Self::NftMetadata { contract_address, token_id } => build_path_with_query("/getNFTMetadata", &[("contractAddress", contract_address), ("tokenId", token_id)]),
        }
    }
}
