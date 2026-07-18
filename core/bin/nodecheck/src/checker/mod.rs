mod evm;

use std::error::Error;

use async_trait::async_trait;
use evm::EvmChecker;
use primitives::{Chain, EVMChain};

use crate::fixtures::fixture;

pub(crate) enum Checker {
    Evm(EvmChecker),
}

pub(crate) enum NodeCheckResult {
    Evm {
        chain: EVMChain,
        addresses: usize,
        transactions: usize,
        archival: bool,
    },
}

#[async_trait]
pub(crate) trait NodeCheck {
    async fn check(&self) -> Result<NodeCheckResult, Box<dyn Error + Send + Sync>>;
}

impl Checker {
    pub(crate) fn new(chain: Chain, url: String, archival: bool) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let evm_chain = EVMChain::from_chain(chain).ok_or_else(|| format!("node checking is not supported for {chain}"))?;
        let fixture = fixture(chain).ok_or_else(|| format!("node fixtures are not configured for {chain}"))?;
        Ok(Self::Evm(EvmChecker::new(evm_chain, url, fixture, archival)))
    }
}

#[async_trait]
impl NodeCheck for Checker {
    async fn check(&self) -> Result<NodeCheckResult, Box<dyn Error + Send + Sync>> {
        match self {
            Self::Evm(checker) => checker.check().await,
        }
    }
}
