pub mod model;
pub mod rules;
pub mod store;
#[cfg(test)]
pub(crate) mod testkit;

use crate::models::list::GemListRow;
use crate::models::state::GemLoadState;
use crate::services::error::{GemServiceError, required_account};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;

use primitives::{Asset, AssetId, Chain, Currency, Delegation, DelegationBase, DelegationValidator, Resource, StakeProviderType, StakeType, WalletId, WalletType};

use crate::api::GemStaticApiClient;
use crate::gateway::GemGateway;
use crate::models::custom_types::GemBigInt;
use crate::models::{GemContractCallData, GemEarnType};

pub use model::{
    GemClaimRewards, GemClaimRewardsDestination, GemDelegationAction, GemDelegationAmountInput, GemDelegationDestination, GemDelegationDetails, GemDelegationStatus, GemEarnActions, GemStakeAction, GemStakeActionItem, GemStakeAmountInput,
    GemStakeDelegationItem, GemStakeDestination, GemStakeInput, GemStakeSection, GemStakeValidatorSelection, GemStakeViewState, GemValidatorRow,
};
pub use store::GemStakeStore;

use crate::services::explorer::GemExplorerService;
use crate::services::name::GemNameService;
use crate::services::preferences::GemPreferencesService;
use crate::services::transfer::GemTransferData;
use crate::services::transfer::rules as transfer_rules;
use crate::services::wallet_session::GemWalletSessionService;

#[derive(uniffi::Object)]
pub struct GemStakeService {
    gateway: Arc<GemGateway>,
    static_api: Arc<GemStaticApiClient>,
    store: Arc<dyn GemStakeStore>,
    names: Arc<GemNameService>,
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
        names: Arc<GemNameService>,
        explorer: Arc<GemExplorerService>,
        preferences: Arc<GemPreferencesService>,
        session: Arc<GemWalletSessionService>,
    ) -> Self {
        Self {
            gateway,
            static_api,
            store,
            names,
            explorer,
            preferences,
            session,
        }
    }

    pub fn get_currency(&self) -> Currency {
        self.preferences.get_currency()
    }

    pub fn stake_transfer_data(&self, asset: Asset, stake_type: StakeType, value: GemBigInt, use_max_amount: bool) -> GemTransferData {
        transfer_rules::stake_transfer_data(asset, stake_type, value, use_max_amount)
    }

    pub fn stake_validator_selection(&self, chain: Chain, input: GemStakeAmountInput) -> GemStakeValidatorSelection {
        rules::validator_selection(chain, &input)
    }

    pub fn earn_apr_row(&self, providers: Vec<DelegationValidator>, asset_apr: Option<f64>) -> GemListRow {
        rules::earn_apr_row(&providers, asset_apr)
    }

    pub fn validator_rows(&self, validators: Vec<DelegationValidator>) -> Vec<GemValidatorRow> {
        validators
            .iter()
            .map(|validator| GemValidatorRow {
                explorer: rules::validator_explorer_address(validator).and_then(|address| self.explorer.get_validator_url(validator.chain, address)),
                ..rules::validator_row(validator)
            })
            .collect()
    }

    pub async fn refresh(&self, chain: Chain, delegations: Vec<Delegation>) -> GemLoadState {
        GemLoadState::refreshed(self.sync(chain).await, !delegations.is_empty())
    }

    pub async fn refresh_earn(&self, asset_id: AssetId, has_rows: bool) -> GemLoadState {
        GemLoadState::refreshed(self.sync_earn(asset_id).await, has_rows)
    }

    pub fn earn_actions(&self, wallet_type: WalletType, providers: Vec<DelegationValidator>) -> GemEarnActions {
        rules::earn_actions(wallet_type, providers)
    }

    pub fn delegation_destination(&self, wallet_type: WalletType, asset: Asset, delegation: Delegation) -> GemDelegationDestination {
        rules::delegation_destination(wallet_type, asset, delegation)
    }

    pub fn delegation_action_destination(&self, asset: Asset, delegation: Delegation, action: GemDelegationAction, validators: Vec<DelegationValidator>) -> GemDelegationDestination {
        rules::delegation_action_destination(asset, delegation, action, validators)
    }

    pub fn positions(&self, delegations: Vec<Delegation>) -> Vec<Delegation> {
        rules::positions(delegations)
    }

    pub fn resource_options(&self, chain: Chain) -> Vec<Resource> {
        rules::resource_options(chain)
    }

    pub fn stake_view_state(&self, input: GemStakeInput) -> GemStakeViewState {
        rules::stake_view_state(input)
    }

    pub fn sorted_delegations(&self, delegations: Vec<Delegation>) -> Vec<Delegation> {
        rules::sorted_delegations(delegations)
    }

    pub fn delegation_details(&self, wallet_type: WalletType, delegation: Delegation, asset: Asset, price: Option<f64>, currency: Currency) -> GemDelegationDetails {
        let rows = self.delegation_rows(delegation.clone());
        rules::delegation_details(wallet_type, &delegation, &asset, price, currency, rows)
    }

    pub fn selectable_validators(&self, validators: Vec<DelegationValidator>) -> Vec<DelegationValidator> {
        rules::selectable_validators(validators)
    }
}

#[uniffi::export]
pub fn validator_row(validator: DelegationValidator) -> GemValidatorRow {
    rules::validator_row(&validator)
}

impl GemStakeService {
    pub async fn sync_earn(&self, asset_id: AssetId) -> Result<(), GemServiceError> {
        let (wallet_id, address) = self.current_account(asset_id.chain).await?;
        self.sync_earn_wallet(wallet_id, asset_id, address).await
    }

    pub fn delegation_rows(&self, delegation: Delegation) -> Vec<GemListRow> {
        rules::delegation_rows(&delegation, Utc::now())
    }

    pub async fn sync(&self, chain: Chain) -> Result<(), GemServiceError> {
        let (wallet_id, address) = self.current_account(chain).await?;
        self.sync_wallet(wallet_id, chain, address).await
    }

    pub async fn get_earn_data(&self, asset_id: AssetId, address: String, value: String, earn_type: GemEarnType) -> Result<GemContractCallData, GemServiceError> {
        Ok(self.gateway.get_earn_data(asset_id, address, value, earn_type).await?)
    }

    pub async fn sync_wallet(&self, wallet_id: WalletId, chain: Chain, address: String) -> Result<(), GemServiceError> {
        let apr = self.store.get_apr(AssetId::from_chain(chain), StakeProviderType::Stake).await?.unwrap_or_default();
        let (names, validators, delegation_validators, delegations) = futures::join!(
            self.static_api.client.get_validators(chain),
            self.gateway.get_staking_validators(chain, Some(apr)),
            self.gateway.get_staking_delegation_validators(chain, address.clone()),
            self.gateway.get_staking_delegations(chain, address),
        );
        let names: HashMap<String, String> = match names {
            Ok(validators) => validators.into_iter().map(|validator| (validator.id, validator.name)).collect(),
            Err(_) => rules::validator_names(self.store.get_validators(AssetId::from_chain(chain), StakeProviderType::Stake).await?),
        };
        self.save_validators(chain, rules::merge_validators(validators?, delegation_validators?, &names)).await?;
        self.save_delegations(wallet_id, chain, delegations?, &names).await
    }

    pub async fn sync_earn_wallet(&self, wallet_id: WalletId, asset_id: AssetId, address: String) -> Result<(), GemServiceError> {
        let apr = self.store.get_apr(asset_id.clone(), StakeProviderType::Earn).await?.unwrap_or_default();
        let providers = rules::earn_validators(self.gateway.get_earn_providers(asset_id.clone()), apr);
        let changed = rules::changed_validators(providers, &self.store.get_validators(asset_id.clone(), StakeProviderType::Earn).await?);
        if !changed.is_empty() {
            self.store.save_validators(changed).await?;
        }
        let positions = self.gateway.get_earn_positions(address, asset_id.clone()).await?;
        let existing_ids = self.store.get_delegation_ids(wallet_id.clone(), asset_id, StakeProviderType::Earn).await?;
        let delete_ids = rules::stale_delegation_ids(existing_ids, &positions);
        self.store.update_delegations(wallet_id, positions, delete_ids).await
    }

    async fn current_account(&self, chain: Chain) -> Result<(WalletId, String), GemServiceError> {
        let wallet = self.session.require_current_wallet().await?;
        let account = required_account(&wallet, chain)?;
        Ok((wallet.id.clone(), account.address.clone()))
    }
    async fn save_validators(&self, chain: Chain, validators: Vec<DelegationValidator>) -> Result<(), GemServiceError> {
        if !validators.is_empty() {
            let asset_id = AssetId::from_chain(chain);
            let stored = self.store.get_validators(asset_id.clone(), StakeProviderType::Stake).await?;
            let changed = rules::changed_validators(validators.clone(), &stored);
            let stale_ids = rules::stale_validator_ids(stored, &validators);
            if !changed.is_empty() {
                self.store.save_validators(changed).await?;
            }
            if !stale_ids.is_empty() {
                self.store.deactivate_validators(asset_id, stale_ids).await?;
            }
            self.names.save_names(rules::validator_address_names(&validators)).await?;
        }
        Ok(())
    }

    async fn save_delegations(&self, wallet_id: WalletId, chain: Chain, delegations: Vec<DelegationBase>, names: &HashMap<String, String>) -> Result<(), GemServiceError> {
        let asset_id = AssetId::from_chain(chain);
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

        let incoming = rules::delegations_with_state(delegations, &validators);
        let existing_ids = self.store.get_delegation_ids(wallet_id.clone(), asset_id, StakeProviderType::Stake).await?;
        let delete_ids = rules::stale_delegation_ids(existing_ids, &incoming);
        self.store.update_delegations(wallet_id, incoming, delete_ids).await
    }
}

#[cfg(test)]
mod tests {
    use super::rules::*;
    use primitives::{Chain, DelegationBase, DelegationState, DelegationValidator, StakeProviderType};
    use std::collections::HashMap;

    #[test]
    fn test_missing_validators_only_for_unknown_ids() {
        let existing: HashMap<_, _> = [("known".to_string(), DelegationValidator::mock_cosmos("known"))].into();
        let delegations = vec![DelegationBase::mock_with_validator("known"), DelegationBase::mock_with_validator("gone"), DelegationBase::mock_with_validator("gone")];
        let names: HashMap<_, _> = [("gone".to_string(), "Gone".to_string())].into();

        let missing = missing_validators(Chain::Cosmos, &delegations, &existing, &names);

        assert_eq!(missing.len(), 1);
        assert_eq!(missing[0].id, "gone");
        assert_eq!(missing[0].name, "Gone");
        assert!(!missing[0].is_active);
    }

    #[test]
    fn test_stale_validator_ids_returns_only_ids_missing_from_the_response() {
        let existing = vec![DelegationValidator::mock_cosmos("kept"), DelegationValidator::mock_cosmos("gone")];
        let incoming = vec![DelegationValidator::mock_cosmos("kept"), DelegationValidator::mock_cosmos("fresh")];

        assert_eq!(stale_validator_ids(existing, &incoming), vec!["gone".to_string()]);
        assert!(stale_validator_ids(vec![DelegationValidator::mock_cosmos("kept")], &[DelegationValidator::mock_cosmos("kept")]).is_empty());
        assert!(
            stale_validator_ids(
                vec![DelegationValidator {
                    is_active: false,
                    ..DelegationValidator::mock_cosmos("gone")
                }],
                &[DelegationValidator::mock_cosmos("kept")]
            )
            .is_empty()
        );
    }

    #[test]
    fn test_delegations_on_inactive_validators_become_inactive() {
        let validators: HashMap<_, _> = [(
            "v".to_string(),
            DelegationValidator {
                is_active: false,
                ..DelegationValidator::mock_cosmos("v")
            },
        )]
        .into();
        let delegations = delegations_with_state(vec![DelegationBase::mock_with_validator("v"), DelegationBase::mock_with_validator("other")], &validators);
        assert_eq!(delegations[0].state, DelegationState::Inactive);
        assert_eq!(delegations[1].state, DelegationState::Active);
    }

    #[test]
    fn test_stale_delegation_ids() {
        let incoming = vec![DelegationBase::mock_with_validator("v")];
        let stale = stale_delegation_ids(vec![incoming[0].id(), "old".to_string()], &incoming);
        assert_eq!(stale, vec!["old".to_string()]);
        assert!(stale_delegation_ids(vec!["a".to_string()], &[]).contains(&"a".to_string()));
    }

    #[test]
    fn test_merge_validators_fills_names_and_dedupes() {
        let names: HashMap<_, _> = [("b".to_string(), "Bee".to_string())].into();
        let mut unnamed = DelegationValidator::mock_cosmos("b");
        unnamed.name = String::new();
        let merged = merge_validators(
            vec![DelegationValidator::mock_cosmos("a")],
            vec![
                DelegationValidator {
                    is_active: false,
                    ..DelegationValidator::mock_cosmos("a")
                },
                unnamed,
            ],
            &names,
        );
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
