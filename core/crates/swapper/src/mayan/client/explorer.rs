use super::MayanClient;
use crate::{
    SwapperError,
    mayan::model::{MayanChain, MayanTransactionResult},
};
use gem_client::{Client, ClientExt};
use std::fmt::Debug;

impl<C> MayanClient<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    pub async fn get_chains(&self) -> Result<Vec<MayanChain>, SwapperError> {
        self.client.get("/chains").await.map_err(SwapperError::from)
    }

    pub async fn get_transaction_status(&self, hash: &str) -> Result<MayanTransactionResult, SwapperError> {
        self.client.get(&format!("/swap/trx/{hash}")).await.map_err(SwapperError::from)
    }
}

#[cfg(test)]
mod tests {
    use gem_client::testkit::MockClient;

    use super::*;

    #[tokio::test]
    async fn test_get_transaction_status() {
        const TRANSACTION_HASH: &str = "0x8867073b70abb2d5700e6ff4bea1e4e196786ca99f72737d080ae13f40bf59f1";

        let client = MockClient::new().with_get(|path| {
            assert_eq!(path, format!("/swap/trx/{TRANSACTION_HASH}"));
            Ok(include_bytes!("../test/bnb_to_mon_swift.json").to_vec())
        });

        MayanClient::new(client).get_transaction_status(TRANSACTION_HASH).await.unwrap();
    }
}
