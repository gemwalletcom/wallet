pub mod error;
pub mod rules;
pub mod store;

use primitives::WalletId;
use std::collections::HashMap;
use std::sync::Arc;

use primitives::{AssetId, Chain, DelegationValidator};

use crate::api::GemStaticApiClient;
use crate::gateway::GemGateway;

pub use error::GemStakeError;
pub use store::GemStakeStore;

#[derive(uniffi::Object)]
pub struct GemStakeService {
    gateway: Arc<GemGateway>,
    static_api: Arc<GemStaticApiClient>,
    store: Arc<dyn GemStakeStore>,
}

#[uniffi::export]
impl GemStakeService {
    #[uniffi::constructor]
    pub fn new(gateway: Arc<GemGateway>, static_api: Arc<GemStaticApiClient>, store: Arc<dyn GemStakeStore>) -> Self {
        Self { gateway, static_api, store }
    }

    pub async fn sync(&self, wallet_id: WalletId, chain: Chain, address: String, apr: f64) -> Result<(), GemStakeError> {
        let names = self.sync_validators(chain, &address, apr).await?;
        self.sync_delegations(wallet_id, chain, &address, &names).await
    }
}

impl GemStakeService {
    async fn sync_validators(&self, chain: Chain, address: &str, apr: f64) -> Result<HashMap<String, String>, GemStakeError> {
        let names: HashMap<String, String> = self
            .static_api
            .client
            .get_validators(chain)
            .await
            .map(|validators| validators.into_iter().map(|validator| (validator.id, validator.name)).collect())
            .unwrap_or_default();

        let (validators, delegation_validators) = futures::join!(
            self.gateway.get_staking_validators(chain, Some(apr)),
            self.gateway.get_staking_delegation_validators(chain, address.to_string()),
        );
        let validators = rules::merge_validators(validators?, delegation_validators?, &names);
        if !validators.is_empty() {
            self.store.save_validators(validators.clone()).await?;
            self.store.save_address_names(rules::validator_address_names(&validators)).await?;
        }
        Ok(names)
    }

    async fn sync_delegations(&self, wallet_id: WalletId, chain: Chain, address: &str, names: &HashMap<String, String>) -> Result<(), GemStakeError> {
        let asset_id = AssetId::from_chain(chain);
        let delegations = self.gateway.get_staking_delegations(chain, address.to_string()).await?;
        let mut validators: HashMap<String, DelegationValidator> = self
            .store
            .get_validators(asset_id.clone())
            .await?
            .into_iter()
            .map(|validator| (validator.id.clone(), validator))
            .collect();

        let missing = rules::missing_validators(chain, &delegations, &validators, names);
        if !missing.is_empty() {
            self.store.save_validators(missing.clone()).await?;
            validators.extend(missing.into_iter().map(|validator| (validator.id.clone(), validator)));
        }

        let incoming = rules::apply_validator_state(delegations, &validators);
        let existing_ids = self.store.get_delegation_ids(wallet_id.clone(), asset_id).await?;
        let delete_ids = rules::stale_delegation_ids(existing_ids, &incoming);
        self.store.update_delegations(wallet_id, incoming, delete_ids).await
    }
}

#[cfg(test)]
mod tests {
    use super::rules::*;
    use num_bigint::BigUint;
    use primitives::{AssetId, Chain, DelegationBase, DelegationState, DelegationValidator, StakeProviderType};
    use std::collections::HashMap;

    fn validator(id: &str, is_active: bool) -> DelegationValidator {
        DelegationValidator {
            chain: Chain::Cosmos,
            id: id.to_string(),
            name: id.to_string(),
            is_active,
            commission: 0.0,
            apr: 1.0,
            provider_type: StakeProviderType::Stake,
        }
    }

    fn delegation(validator_id: &str, state: DelegationState) -> DelegationBase {
        DelegationBase {
            asset_id: AssetId::from_chain(Chain::Cosmos),
            state,
            balance: BigUint::from(1u32),
            shares: BigUint::from(0u32),
            rewards: BigUint::from(0u32),
            completion_date: None,
            delegation_id: "d".to_string(),
            validator_id: validator_id.to_string(),
        }
    }

    #[test]
    fn test_missing_validators_only_for_unknown_ids() {
        let existing: HashMap<_, _> = [("known".to_string(), validator("known", true))].into();
        let delegations = vec![
            delegation("known", DelegationState::Active),
            delegation("gone", DelegationState::Active),
            delegation("gone", DelegationState::Active),
        ];
        let names: HashMap<_, _> = [("gone".to_string(), "Gone".to_string())].into();

        let missing = missing_validators(Chain::Cosmos, &delegations, &existing, &names);

        assert_eq!(missing.len(), 1);
        assert_eq!(missing[0].id, "gone");
        assert_eq!(missing[0].name, "Gone");
        assert!(!missing[0].is_active);
    }

    #[test]
    fn test_delegations_on_inactive_validators_become_inactive() {
        let validators: HashMap<_, _> = [("v".to_string(), validator("v", false))].into();
        let delegations = apply_validator_state(vec![delegation("v", DelegationState::Active), delegation("other", DelegationState::Active)], &validators);
        assert_eq!(delegations[0].state, DelegationState::Inactive);
        assert_eq!(delegations[1].state, DelegationState::Active);
    }

    #[test]
    fn test_stale_delegation_ids() {
        let incoming = vec![delegation("v", DelegationState::Active)];
        let stale = stale_delegation_ids(vec![incoming[0].id(), "old".to_string()], &incoming);
        assert_eq!(stale, vec!["old".to_string()]);
        assert!(stale_delegation_ids(vec!["a".to_string()], &[]).contains(&"a".to_string()));
    }

    #[test]
    fn test_merge_validators_fills_names_and_dedupes() {
        let names: HashMap<_, _> = [("b".to_string(), "Bee".to_string())].into();
        let mut unnamed = validator("b", true);
        unnamed.name = String::new();
        let merged = merge_validators(vec![validator("a", true)], vec![validator("a", false), unnamed], &names);
        assert_eq!(merged.len(), 2);
        assert!(merged[0].is_active);
        assert_eq!(merged[1].name, "Bee");
    }
}
