use gem_client::{ContentType, Target};

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

    fn content_type(&self) -> ContentType {
        match self {
            Self::SendTransaction => ContentType::TextPlain,
            _ => ContentType::ApplicationJson,
        }
    }
}
