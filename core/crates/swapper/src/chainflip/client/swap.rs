use super::SwapTxResponse;
use crate::SwapperError;
use gem_client::{Client, ClientExt};
use std::fmt::Debug;

const SWAP_PATH: &str = "/v2/swaps";

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
        let path = format!("{SWAP_PATH}/{tx_hash}");
        self.client.get(&path).await.map_err(SwapperError::from)
    }
}
