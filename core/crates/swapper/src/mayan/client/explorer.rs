use super::MayanClient;
use crate::{
    SwapperError,
    mayan::{
        model::{MayanChain, MayanTransactionResult},
        target::MayanTarget,
    },
};
use gem_client::{Client, ClientExt};
use std::fmt::Debug;

impl<C> MayanClient<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    pub async fn get_chains(&self) -> Result<Vec<MayanChain>, SwapperError> {
        self.client.get(MayanTarget::Chains).await.map_err(SwapperError::from)
    }

    pub async fn get_transaction_status(&self, hash: &str) -> Result<MayanTransactionResult, SwapperError> {
        let result: MayanTransactionResult = self.client.get(MayanTarget::TransactionStatus { hash: hash.to_string() }).await.map_err(SwapperError::from)?;
        if !result.source_tx_hash.as_deref().is_some_and(|source_hash| source_hash_matches(source_hash, hash)) {
            return Err(SwapperError::transaction_error("Mayan transaction source hash mismatch"));
        }
        Ok(result)
    }
}

fn source_hash_matches(source_hash: &str, requested_hash: &str) -> bool {
    source_hash == requested_hash || (source_hash.starts_with("0x") && requested_hash.starts_with("0x") && source_hash.eq_ignore_ascii_case(requested_hash))
}

#[cfg(test)]
mod tests {
    use gem_client::testkit::MockClient;

    use super::*;

    #[tokio::test]
    async fn test_get_transaction_status() {
        const TRANSACTION_HASH: &str = "0x8867073B70ABB2D5700E6FF4BEA1E4E196786CA99F72737D080AE13F40BF59F1";

        let client = MockClient::new().with_get(|path| {
            assert_eq!(path, format!("/swap/trx/{TRANSACTION_HASH}"));
            Ok(include_bytes!("../test/bnb_to_mon_swift.json").to_vec())
        });

        MayanClient::new(client).get_transaction_status(TRANSACTION_HASH).await.unwrap();
    }

    #[tokio::test]
    async fn test_rejects_shared_unlock_transaction_result() {
        const UNLOCK_HASH: &str = "shared-unlock-hash";

        let client = MockClient::new().with_get(|path| {
            assert_eq!(path, format!("/swap/trx/{UNLOCK_HASH}"));
            Ok(include_bytes!("../test/bnb_to_mon_swift.json").to_vec())
        });

        let error = MayanClient::new(client).get_transaction_status(UNLOCK_HASH).await.unwrap_err();
        assert_eq!(error, SwapperError::transaction_error("Mayan transaction source hash mismatch"));
    }
}
