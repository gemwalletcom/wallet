use std::collections::HashMap;

use gem_client::{CONTENT_TYPE, ContentType, Target};

#[derive(Clone, Debug)]
pub enum BlockbookTarget {
    GetNodeInfo,
    GetBlock { height: u64, page: usize },
    GetAddress { address: String },
    GetAddressTransactions { address: String, page_size: usize },
    GetTransaction { hash: String },
    GetUtxos { address: String },
    EstimateFee { blocks: i32 },
    SendTransaction,
}

impl Target for BlockbookTarget {
    fn path(&self) -> String {
        match self {
            Self::GetNodeInfo => "/api/v2/".to_string(),
            Self::GetBlock { height, page } => format!("/api/v2/block/{height}?page={page}"),
            Self::GetAddress { address } => format!("/api/v2/address/{address}"),
            Self::GetAddressTransactions { address, page_size } => format!("/api/v2/address/{address}?pageSize={page_size}&details=txs"),
            Self::GetTransaction { hash } => format!("/api/v2/tx/{hash}"),
            Self::GetUtxos { address } => format!("/api/v2/utxo/{address}"),
            Self::EstimateFee { blocks } => format!("/api/v2/estimatefee/{blocks}"),
            Self::SendTransaction => "/api/v2/sendtx/".to_string(),
        }
    }

    fn headers(&self) -> HashMap<String, String> {
        match self {
            Self::SendTransaction => HashMap::from([(CONTENT_TYPE.to_string(), ContentType::TextPlain.as_str().to_string())]),
            _ => HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path() {
        assert_eq!(BlockbookTarget::GetBlock { height: 800000, page: 2 }.path(), "/api/v2/block/800000?page=2");
        assert_eq!(BlockbookTarget::GetAddress { address: "bc1q".into() }.path(), "/api/v2/address/bc1q");
        assert_eq!(
            BlockbookTarget::GetAddressTransactions {
                address: "bc1q".into(),
                page_size: 25
            }
            .path(),
            "/api/v2/address/bc1q?pageSize=25&details=txs"
        );
        assert_eq!(BlockbookTarget::GetTransaction { hash: "abc".into() }.path(), "/api/v2/tx/abc");
        assert_eq!(BlockbookTarget::GetUtxos { address: "bc1q".into() }.path(), "/api/v2/utxo/bc1q");
        assert_eq!(BlockbookTarget::EstimateFee { blocks: 6 }.path(), "/api/v2/estimatefee/6");
    }
}
