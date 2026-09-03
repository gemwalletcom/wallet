use gem_client::{Target, build_path_with_query};

use super::model::{PathsQuery, StatusQuery};

#[derive(Clone, Debug)]
pub(super) enum SwapsXyzTarget {
    Paths { query: PathsQuery },
    Action,
    Status { query: StatusQuery },
}

impl Target for SwapsXyzTarget {
    fn path(&self) -> String {
        match self {
            Self::Paths { query } => build_path_with_query("/getPaths", query),
            Self::Action => "/action".to_string(),
            Self::Status { query } => build_path_with_query("/getStatus", query),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path() {
        assert_eq!(
            SwapsXyzTarget::Paths {
                query: PathsQuery::native(1, 8453)
            }
            .path(),
            "/getPaths?srcChainId=1&srcToken=0x0000000000000000000000000000000000000000&dstChainId=8453"
        );
        assert_eq!(
            SwapsXyzTarget::Status {
                query: StatusQuery {
                    tx_hash: "0xabc".into(),
                    chain_id: 1
                }
            }
            .path(),
            "/getStatus?txHash=0xabc&chainId=1"
        );
    }
}
