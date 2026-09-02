pub mod model;
pub mod rules;
pub mod store;

use crate::services::error::GemServiceError;
use std::collections::HashMap;
use std::sync::Arc;

use primitives::{Asset, AssetId, Chain, Currency, Delegation, DelegationBase, DelegationValidator, StakeProviderType, StakeType, WalletId, WalletType};

use crate::api::GemStaticApiClient;
use crate::gateway::GemGateway;
use crate::models::custom_types::GemBigInt;
use crate::models::{GemContractCallData, GemEarnType};

pub use model::{GemClaimRewards, GemClaimRewardsDestination, GemDelegationAction, GemStakeAction, GemStakeActionItem};
pub use store::GemStakeStore;

use crate::block_explorer::GemBlockExplorerLink;
use crate::services::balance::GemAssetBalance;
use crate::services::explorer::GemExplorerService;
use crate::services::name::GemAddressStore;
use crate::services::preferences::GemPreferencesService;
use crate::services::transfer::GemTransferData;
use crate::services::transfer::rules as transfer_rules;
use crate::services::wallet_session::GemWalletSessionService;

#[derive(uniffi::Object)]
pub struct GemStakeService {
    gateway: Arc<GemGateway>,
    static_api: Arc<GemStaticApiClient>,
    store: Arc<dyn GemStakeStore>,
    address_store: Arc<dyn GemAddressStore>,
    explorer: Arc<GemExplorerService>,
    preferences: Arc<GemPreferencesService>,
    session: Arc<GemWalletSessionService>,
}

#[uniffi::export]
impl GemStakeService {
    #[uniffi::constructor]
    pub fn new(
        gateway: Arc<GemGateway>,
        static_api: Arc<GemStaticApiClient>,
        store: Arc<dyn GemStakeStore>,
        address_store: Arc<dyn GemAddressStore>,
        explorer: Arc<GemExplorerService>,
        preferences: Arc<GemPreferencesService>,
        session: Arc<GemWalletSessionService>,
    ) -> Self {
        Self {
            gateway,
            static_api,
            store,
            address_store,
            explorer,
            preferences,
            session,
        }
    }

    pub fn currency(&self) -> Currency {
        self.preferences.get_currency()
    }

    pub fn stake_transfer_data(&self, asset: Asset, stake_type: StakeType, value: GemBigInt, use_max_amount: bool) -> GemTransferData {
        transfer_rules::stake_transfer_data(asset, stake_type, value, use_max_amount)
    }

    pub fn validator_url(&self, validator: DelegationValidator) -> Option<GemBlockExplorerLink> {
        let address = rules::validator_explorer_address(&validator)?;
        self.explorer.get_validator_url(validator.chain, address)
    }

    pub async fn sync(&self, chain: Chain) -> Result<(), GemServiceError> {
        let (wallet_id, address) = self.current_account(chain)?;
        self.sync_wallet(wallet_id, chain, address).await
    }

    pub async fn sync_earn(&self, asset_id: AssetId) -> Result<(), GemServiceError> {
        let (wallet_id, address) = self.current_account(asset_id.chain)?;
        self.sync_earn_wallet(wallet_id, asset_id, address).await
    }

    pub fn delegation_actions(&self, wallet_type: WalletType, delegation: Delegation) -> Vec<GemDelegationAction> {
        rules::delegation_actions(wallet_type, &delegation)
    }

    pub fn can_claim_delegation_rewards(&self, wallet_type: WalletType, delegation: Delegation) -> bool {
        rules::can_claim_rewards(wallet_type, &delegation)
    }

    pub fn shows_completion_date(&self, delegation: DelegationBase) -> bool {
        rules::shows_completion_date(&delegation)
    }

    pub fn shows_rewards(&self, delegation: DelegationBase) -> bool {
        rules::shows_rewards(&delegation)
    }

    pub fn stake_actions(&self, wallet_type: WalletType, chain: Chain, has_validators: bool, balance: GemAssetBalance, delegations: Vec<Delegation>) -> Vec<GemStakeActionItem> {
        rules::stake_actions(wallet_type, chain, has_validators, &balance, &delegations)
    }

    pub fn claim_rewards(&self, chain: Chain, delegations: Vec<Delegation>) -> GemClaimRewards {
        rules::claim_rewards(chain, delegations)
    }

    pub fn recommended_validator_ids(&self, chain: Chain) -> Vec<String> {
        rules::recommended_validator_ids(chain)
    }

    pub fn recommended_validator(&self, chain: Chain, validators: Vec<DelegationValidator>) -> Option<DelegationValidator> {
        rules::recommended_validator(chain, validators)
    }

    pub fn selectable_validators(&self, validators: Vec<DelegationValidator>) -> Vec<DelegationValidator> {
        rules::selectable_validators(validators)
    }
}

impl GemStakeService {
    pub async fn get_earn_data(&self, asset_id: AssetId, address: String, value: String, earn_type: GemEarnType) -> Result<GemContractCallData, GemServiceError> {
        Ok(self.gateway.get_earn_data(asset_id, address, value, earn_type).await?)
    }

    pub async fn sync_wallet(&self, wallet_id: WalletId, chain: Chain, address: String) -> Result<(), GemServiceError> {
        let apr = self.store.get_apr(AssetId::from_chain(chain), StakeProviderType::Stake).await?.unwrap_or_default();
        let names = self.sync_validators(chain, &address, apr).await?;
        self.sync_delegations(wallet_id, chain, &address, &names).await
    }

    pub async fn sync_earn_wallet(&self, wallet_id: WalletId, asset_id: AssetId, address: String) -> Result<(), GemServiceError> {
        let apr = self.store.get_apr(asset_id.clone(), StakeProviderType::Earn).await?.unwrap_or_default();
        let providers = rules::earn_validators(self.gateway.get_earn_providers(asset_id.clone()), apr);
        self.store.save_validators(providers).await?;
        let positions = self.gateway.get_earn_positions(address, asset_id.clone()).await?;
        let existing_ids = self.store.get_delegation_ids(wallet_id.clone(), asset_id, StakeProviderType::Earn).await?;
        let delete_ids = rules::stale_delegation_ids(existing_ids, &positions);
        self.store.update_delegations(wallet_id, positions, delete_ids).await
    }

    fn current_account(&self, chain: Chain) -> Result<(WalletId, String), GemServiceError> {
        let wallet = self.session.current_wallet()?;
        let account = wallet.account(chain).ok_or_else(|| GemServiceError::NotFound {
            msg: format!("wallet {} has no {chain} account", wallet.id.id()),
        })?;
        Ok((wallet.id.clone(), account.address.clone()))
    }
    async fn sync_validators(&self, chain: Chain, address: &str, apr: f64) -> Result<HashMap<String, String>, GemServiceError> {
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
            let asset_id = AssetId::from_chain(chain);
            let stale_ids = rules::stale_validator_ids(self.store.get_validators(asset_id.clone(), StakeProviderType::Stake).await?, &validators);
            self.store.save_validators(validators.clone()).await?;
            if !stale_ids.is_empty() {
                self.store.deactivate_validators(asset_id, stale_ids).await?;
            }
            self.address_store.save_address_names(rules::validator_address_names(&validators)).await?;
        }
        Ok(names)
    }

    async fn sync_delegations(&self, wallet_id: WalletId, chain: Chain, address: &str, names: &HashMap<String, String>) -> Result<(), GemServiceError> {
        let asset_id = AssetId::from_chain(chain);
        let delegations = self.gateway.get_staking_delegations(chain, address.to_string()).await?;
        let mut validators: HashMap<String, DelegationValidator> = self
            .store
            .get_validators(asset_id.clone(), StakeProviderType::Stake)
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
        let existing_ids = self.store.get_delegation_ids(wallet_id.clone(), asset_id, StakeProviderType::Stake).await?;
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
    fn test_stale_validator_ids_returns_only_ids_missing_from_the_response() {
        let existing = vec![validator("kept", true), validator("gone", true)];
        let incoming = vec![validator("kept", true), validator("fresh", true)];

        assert_eq!(stale_validator_ids(existing, &incoming), vec!["gone".to_string()]);
        assert!(stale_validator_ids(vec![validator("kept", true)], &[validator("kept", true)]).is_empty());
        assert!(stale_validator_ids(vec![validator("gone", false)], &[validator("kept", true)]).is_empty());
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

    #[test]
    fn test_earn_validators_take_asset_apr() {
        let provider = DelegationValidator {
            chain: Chain::Ethereum,
            id: "provider".into(),
            name: "Provider".into(),
            is_active: true,
            commission: 0.0,
            apr: 0.0,
            provider_type: StakeProviderType::Earn,
        };

        let validators = earn_validators(vec![provider], 4.5);

        assert_eq!(validators[0].apr, 4.5);
        assert_eq!(validators[0].provider_type, StakeProviderType::Earn);
    }
}
