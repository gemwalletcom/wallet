use chrono::{DateTime, Utc};
use primitives::{Chain, TransactionChange, TransactionMetadata, TransactionState, TransactionUpdate, chain_transaction_timeout, swap_transaction_timeout};
use std::sync::Arc;
use swapper::{SwapResult, SwapperProvider, swapper::GemSwapper};

use crate::gateway::ChainClientFactory;
use crate::models::{GemTransactionStateRequest, GemTransactionSwapStateRequest};

use super::TransactionStatusError;

pub struct StatusProvider {
    chain_factory: Arc<ChainClientFactory>,
    swapper: GemSwapper,
}

impl StatusProvider {
    pub fn new(chain_factory: Arc<ChainClientFactory>, swapper: GemSwapper) -> Self {
        Self { chain_factory, swapper }
    }

    pub async fn get(&self, chain: Chain, request: GemTransactionStateRequest) -> Result<TransactionUpdate, TransactionStatusError> {
        let created_at = request.created_at;
        let result = self.chain_status(chain, request).await;
        get_transaction_update(chain, None, created_at, result)
    }

    pub async fn get_swap_status(&self, chain: Chain, request: GemTransactionSwapStateRequest) -> Result<TransactionUpdate, TransactionStatusError> {
        let created_at = request.transaction.created_at;
        let destination_chain = request.destination_chain;
        let result = self.swap_transaction_status(chain, request).await;
        get_transaction_update(chain, Some(destination_chain), created_at, result)
    }

    async fn swap_transaction_status(&self, chain: Chain, request: GemTransactionSwapStateRequest) -> Result<TransactionUpdate, TransactionStatusError> {
        if !request.swap_provider.is_cross_chain() {
            return self.chain_status(chain, request.transaction).await;
        }
        self.cross_chain_swap_status(chain, request).await
    }

    async fn cross_chain_swap_status(&self, chain: Chain, request: GemTransactionSwapStateRequest) -> Result<TransactionUpdate, TransactionStatusError> {
        match request.state {
            TransactionState::Pending => {
                let source_chain_update = self.chain_status(chain, request.transaction).await?;
                Ok(pending_cross_chain_swap_update(source_chain_update))
            }
            TransactionState::InTransit => self.swap_provider_status(chain, request.swap_provider, &request.transaction.id).await,
            state @ (TransactionState::Confirmed | TransactionState::Failed | TransactionState::Reverted) => Ok(TransactionUpdate::new_state(state)),
        }
    }

    async fn chain_status(&self, chain: Chain, request: GemTransactionStateRequest) -> Result<TransactionUpdate, TransactionStatusError> {
        let provider = self.chain_factory.create(chain).await?;
        provider
            .get_transaction_status(request.into())
            .await
            .map_err(|e| TransactionStatusError::NetworkError(e.to_string()))
    }

    async fn swap_provider_status(&self, chain: Chain, provider: SwapperProvider, transaction_hash: &str) -> Result<TransactionUpdate, TransactionStatusError> {
        let result = self
            .swapper
            .get_swap_result(chain, provider, transaction_hash)
            .await
            .map_err(|e| TransactionStatusError::NetworkError(e.to_string()))?;
        Ok(in_transit_swap_update(result))
    }
}

fn in_transit_swap_update(result: SwapResult) -> TransactionUpdate {
    let state = result.status.transaction_state().unwrap_or(TransactionState::InTransit);
    let eta_in_seconds = if state.is_completed() { 0 } else { result.eta_in_seconds.unwrap_or_default() };
    let changes = result
        .metadata
        .map(|metadata| TransactionChange::Metadata(TransactionMetadata::Swap(metadata)))
        .into_iter()
        .chain([TransactionChange::ConfirmationEtaSeconds(eta_in_seconds)])
        .collect();
    TransactionUpdate::new(state, changes)
}

fn pending_cross_chain_swap_update(source_update: TransactionUpdate) -> TransactionUpdate {
    match source_update.state {
        TransactionState::Confirmed => TransactionUpdate::new(
            TransactionState::InTransit,
            source_update
                .changes
                .into_iter()
                .filter(|change| !matches!(change, TransactionChange::ConfirmationEtaSeconds(_)))
                .chain([TransactionChange::ConfirmationEtaSeconds(0)])
                .collect(),
        ),
        _ => source_update,
    }
}

fn transaction_timeout(chain: Chain, destination_chain: Option<Chain>, state: TransactionState) -> Option<i64> {
    match state {
        TransactionState::Pending => Some(i64::from(chain_transaction_timeout(chain))),
        TransactionState::InTransit => Some(swap_transaction_timeout(chain, destination_chain.unwrap_or(chain)) as i64),
        TransactionState::Confirmed | TransactionState::Failed | TransactionState::Reverted => None,
    }
}

fn get_transaction_update(
    chain: Chain,
    destination_chain: Option<Chain>,
    created_at: DateTime<Utc>,
    result: Result<TransactionUpdate, TransactionStatusError>,
) -> Result<TransactionUpdate, TransactionStatusError> {
    let elapsed = (Utc::now() - created_at).num_milliseconds();
    let pending_expired = elapsed > i64::from(chain_transaction_timeout(chain));

    match result {
        Ok(update) => Ok(if transaction_timeout(chain, destination_chain, update.state).is_some_and(|timeout| elapsed > timeout) {
            let changes = if update.state == TransactionState::InTransit {
                vec![TransactionChange::ConfirmationEtaSeconds(0)]
            } else {
                Vec::new()
            };
            TransactionUpdate::new(TransactionState::Failed, changes)
        } else {
            update
        }),
        err @ Err(TransactionStatusError::NetworkError(_)) => err,
        Err(_) if pending_expired => Ok(TransactionUpdate::new_state(TransactionState::Failed)),
        Err(err) => Err(err),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::BigInt;
    use primitives::{TransactionSwapMetadata, swap::SwapStatus};

    #[test]
    fn test_get_transaction_update() {
        let chain = Chain::Ethereum;
        let now = Utc::now;
        let pending = || Ok(TransactionUpdate::new_state(TransactionState::Pending));
        let in_transit = || Ok(TransactionUpdate::new_state(TransactionState::InTransit));
        let confirmed = || Ok(TransactionUpdate::new_state(TransactionState::Confirmed));

        assert_eq!(get_transaction_update(chain, None, now(), pending()).unwrap().state, TransactionState::Pending);
        assert_eq!(
            get_transaction_update(chain, None, DateTime::<Utc>::UNIX_EPOCH, pending()).unwrap().state,
            TransactionState::Failed
        );
        assert_eq!(
            get_transaction_update(chain, Some(Chain::Solana), now() - chrono::Duration::hours(3), in_transit())
                .unwrap()
                .state,
            TransactionState::InTransit
        );
        assert_eq!(
            get_transaction_update(chain, Some(chain), now() - chrono::Duration::hours(3), in_transit()).unwrap(),
            TransactionUpdate::new(TransactionState::Failed, vec![TransactionChange::ConfirmationEtaSeconds(0)])
        );
        assert_eq!(
            get_transaction_update(chain, Some(Chain::Solana), DateTime::<Utc>::UNIX_EPOCH, confirmed()).unwrap().state,
            TransactionState::Confirmed
        );
    }

    #[test]
    fn test_pending_cross_chain_swap_update() {
        let source_update = TransactionUpdate::new(
            TransactionState::Confirmed,
            vec![
                TransactionChange::HashChange {
                    old: "broadcast_hash".into(),
                    new: "source_hash".into(),
                },
                TransactionChange::NetworkFee(BigInt::from(123_u32)),
                TransactionChange::ConfirmationEtaSeconds(60),
            ],
        );

        let update = pending_cross_chain_swap_update(source_update);

        assert_eq!(update.state, TransactionState::InTransit);
        assert_eq!(
            update.changes,
            vec![
                TransactionChange::HashChange {
                    old: "broadcast_hash".into(),
                    new: "source_hash".into(),
                },
                TransactionChange::NetworkFee(BigInt::from(123_u32)),
                TransactionChange::ConfirmationEtaSeconds(0),
            ]
        );

        let pending = pending_cross_chain_swap_update(TransactionUpdate::new_state(TransactionState::Pending));

        assert_eq!(pending.state, TransactionState::Pending);
        assert!(pending.changes.is_empty());
    }

    #[test]
    fn test_in_transit_swap_update() {
        let metadata = TransactionSwapMetadata {
            from_asset: Chain::Ton.as_asset_id(),
            from_value: "1000000".into(),
            to_asset: Chain::Solana.as_asset_id(),
            to_value: "966847".into(),
            provider: Some("near_intents".into()),
        };
        let completed = in_transit_swap_update(SwapResult {
            status: SwapStatus::Completed,
            metadata: Some(metadata.clone()),
            eta_in_seconds: Some(120),
        });
        let pending = in_transit_swap_update(SwapResult {
            status: SwapStatus::Pending,
            metadata: None,
            eta_in_seconds: Some(120),
        });
        let missing = in_transit_swap_update(SwapResult {
            status: SwapStatus::Pending,
            metadata: None,
            eta_in_seconds: None,
        });
        assert_eq!(completed.state, TransactionState::Confirmed);
        assert!(completed.changes.contains(&TransactionChange::Metadata(TransactionMetadata::Swap(metadata))));
        assert_eq!(pending.state, TransactionState::InTransit);
        assert_eq!(pending.changes, vec![TransactionChange::ConfirmationEtaSeconds(120)]);
        assert_eq!(missing.changes, vec![TransactionChange::ConfirmationEtaSeconds(0)]);
        assert_eq!(completed.changes.last(), Some(&TransactionChange::ConfirmationEtaSeconds(0)));
    }
}
