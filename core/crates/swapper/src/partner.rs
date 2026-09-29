use async_trait::async_trait;
use primitives::{SwapProvider, swap::SwapPartnerTransaction};

use crate::SwapperError;

#[derive(Debug, Clone, PartialEq)]
pub enum SwapPartnerCursor {
    Next(String),
    Latest(String),
}

impl SwapPartnerCursor {
    pub fn value(&self) -> &str {
        match self {
            Self::Next(value) | Self::Latest(value) => value,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SwapPartnerTransactionsPage {
    pub transactions: Vec<SwapPartnerTransaction>,
    pub cursor: SwapPartnerCursor,
}

#[async_trait]
pub trait SwapPartnerProvider: Send + Sync {
    fn provider(&self) -> SwapProvider;
    async fn get_transactions(&self, cursor: Option<String>) -> Result<SwapPartnerTransactionsPage, SwapperError>;
}
