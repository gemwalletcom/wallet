#[derive(Clone, Debug)]
pub enum TronGridTarget {
    GetTransactions(String, usize, Option<String>),
    GetTrc20Transactions(String, usize, Option<String>),
    GetAccount(String),
}

impl TronGridTarget {
    pub fn path(&self) -> String {
        match self {
            Self::GetTransactions(address, limit, fingerprint) => with_fingerprint(format!("/v1/accounts/{address}/transactions?limit={limit}"), fingerprint.as_deref()),
            Self::GetTrc20Transactions(address, limit, fingerprint) => with_fingerprint(format!("/v1/accounts/{address}/transactions/trc20?limit={limit}"), fingerprint.as_deref()),
            Self::GetAccount(address) => format!("/v1/accounts/{address}"),
        }
    }
}

fn with_fingerprint(path: String, fingerprint: Option<&str>) -> String {
    match fingerprint {
        Some(fingerprint) => format!("{path}&fingerprint={fingerprint}"),
        None => path,
    }
}
