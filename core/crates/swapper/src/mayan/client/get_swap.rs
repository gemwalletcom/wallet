use super::MayanClient;
use crate::{
    SwapperError,
    mayan::{
        model::{GetSwapEvmParams, GetSwapSolanaParams, GetSwapSuiParams},
        target::MayanTarget,
    },
};
use gem_client::{Client, ClientExt};
use serde::de::DeserializeOwned;
use std::fmt::Debug;

impl<C> MayanClient<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    pub(in crate::mayan) async fn get_swap_evm<U: DeserializeOwned + Send>(&self, params: GetSwapEvmParams) -> Result<U, SwapperError> {
        self.client.get(MayanTarget::SwapEvm { params }).await.map_err(SwapperError::from)
    }

    pub(in crate::mayan) async fn get_swap_solana<U: DeserializeOwned + Send>(&self, params: GetSwapSolanaParams) -> Result<U, SwapperError> {
        self.client.get(MayanTarget::SwapSolana { params }).await.map_err(SwapperError::from)
    }

    pub(in crate::mayan) async fn get_swap_sui<U: DeserializeOwned + Send>(&self, params: GetSwapSuiParams) -> Result<U, SwapperError> {
        self.client.post(MayanTarget::SwapSui, &params).await.map_err(SwapperError::from)
    }
}
