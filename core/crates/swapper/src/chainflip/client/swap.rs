use super::SwapTxResponse;
use super::target::ChainflipTarget;
use crate::SwapperError;
use gem_client::{Client, ClientExt};
use std::fmt::Debug;

#[derive(Clone, Debug)]
pub struct ChainflipClient<C>
where
    C: Client + Clone + Debug,
{
    client: C,
}

impl<C> ChainflipClient<C>
where
    C: Client + Clone + Debug,
{
    pub fn new(client: C) -> Self {
        Self { client }
    }

    pub async fn get_tx_status(&self, tx_hash: &str) -> Result<SwapTxResponse, SwapperError> {
        self.client
            .get(ChainflipTarget::SwapStatus { tx_hash: tx_hash.to_string() })
            .await
            .map_err(SwapperError::from)
    }
}
