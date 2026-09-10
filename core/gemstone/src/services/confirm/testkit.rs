use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use primitives::{Asset, AssetBasic, AssetFull, AssetId, Chain, DelegationBase, DelegationValidator, StakeProviderType, Transaction, Wallet, WalletId};

use super::{GemConfirmData, GemConfirmInput, GemConfirmService, GemConfirmTransferService, GemTransactionSigner};
use crate::GemstoneError;
use crate::api::{GemApiClient, GemDeviceApiClient, GemStaticApiClient};
use crate::gateway::{EmptyPreferences, GemGateway};
use crate::models::transaction::{GemSignedTransaction, GemSignerInput, GemTransactionLoadFee, GemTransactionLoadMetadata};
use crate::services::assets::{GemAssetStore, GemAssetsService, config::GemAssetConfigService};
use crate::services::balance::{GemAssetBalance, GemBalanceRecord, GemBalanceService, GemBalanceStore};
use crate::services::device::GemDeviceKeyService;
use crate::services::error::GemServiceError;
use crate::services::explorer::GemExplorerService;
use crate::services::name::GemNameService;
use crate::services::nft::{GemNftService, testkit::MemoryNftStore};
use crate::services::preferences::{GemPreferencesService, testkit::MemoryPreferencesStore};
use crate::services::price::{GemPriceService, testkit::MemoryPriceStore};
use crate::services::stake::{GemStakeService, GemStakeStore};
use crate::services::stream::testkit::SubscriptionTestkit;
use crate::services::transaction_state::{GemTransactionStateService, GemTransactionStatusService, testkit::MemoryTransactionStateStore};
use crate::services::transfer::{GemRecentActivityService, testkit::MemoryRecentActivityStore};
use crate::services::transfer::{GemRecipient, GemTransferData};
use crate::services::wallet::testkit::{MemoryAddressStore, MemoryKeystorePassword, MemoryWalletStore};
use crate::services::wallet_session::{GemWalletSessionService, testkit::MemoryWalletSessionStore};
use crate::services::{GemScanService, GemSimulationService};
use crate::testkit::TestAlienProvider;
use num_bigint::BigInt;
use primitives::{Account, FeePriority, GasPriceType, TransactionInputType};

pub struct ConfirmTestkit {
    pub service: Arc<GemConfirmTransferService>,
    pub balances: Arc<MemoryBalanceStore>,
}

impl ConfirmTestkit {
    pub fn new(wallet: Wallet, selected_wallet: Wallet) -> Self {
        let provider = Arc::new(TestAlienProvider::with_status(503));
        let preferences_store = Arc::new(MemoryPreferencesStore::default());
        let preferences = Arc::new(GemPreferencesService::new(preferences_store.clone()));
        let selected = Arc::new(MemoryWalletSessionStore {
            current: Mutex::new(Some(selected_wallet.id.clone())),
        });
        let wallets = Arc::new(MemoryWalletStore {
            wallets: Mutex::new(vec![wallet.clone(), selected_wallet]),
            ..Default::default()
        });
        let session = Arc::new(GemWalletSessionService::new(selected.clone(), wallets.clone()));
        let gateway = Arc::new(GemGateway::new(provider.clone(), preferences_store, Arc::new(EmptyPreferences)));
        let api = Arc::new(GemApiClient::new(provider.clone()));
        let device_api = Arc::new(GemDeviceApiClient::new(provider.clone(), Arc::new(GemDeviceKeyService::new(Arc::new(EmptyPreferences)))));
        let price = Arc::new(GemPriceService::new(Arc::new(MemoryPriceStore::default())));
        let asset_store = Arc::new(MemoryAssetStore);
        let assets = Arc::new(GemAssetsService::new(
            api,
            gateway.clone(),
            asset_store.clone(),
            price.clone(),
            preferences.clone(),
            session.clone(),
        ));
        let balances = Arc::new(MemoryBalanceStore {
            wallet_id: wallet.id,
            requests: Mutex::new(Vec::new()),
        });
        let balance = Arc::new(GemBalanceService::new(
            gateway.clone(),
            wallets,
            asset_store,
            balances.clone(),
            assets.clone(),
            Arc::new(SubscriptionTestkit::new(&[], &[]).service),
        ));
        let explorer = Arc::new(GemExplorerService::new(preferences.clone()));
        let addresses = Arc::new(MemoryAddressStore::default());
        let stake = Arc::new(GemStakeService::new(
            gateway.clone(),
            Arc::new(GemStaticApiClient::new(provider.clone())),
            Arc::new(UnusedStakeStore),
            addresses.clone(),
            explorer.clone(),
            preferences.clone(),
            session.clone(),
        ));
        let nft = Arc::new(GemNftService::new(device_api.clone(), Arc::new(MemoryNftStore::default()), session.clone()));
        let transactions = Arc::new(GemTransactionStateService::new(
            gateway.clone(),
            Arc::new(MemoryTransactionStateStore::default()),
            assets.clone(),
            balance.clone(),
            stake,
            nft,
        ));
        let confirm = Arc::new(GemConfirmService::new(
            gateway,
            Arc::new(GemSimulationService::new(provider, Arc::new(EmptyPreferences))),
            Arc::new(GemScanService::new(device_api.clone())),
            transactions,
            balance,
            price,
            assets,
            Arc::new(UnusedTransactionStatus),
        ));
        let service = Arc::new(GemConfirmTransferService::new(
            confirm,
            explorer,
            Arc::new(GemNameService::new(device_api, addresses)),
            Arc::new(GemAssetConfigService::new()),
            Arc::new(UnusedSigner),
            Arc::new(MemoryKeystorePassword::default()),
            Arc::new(GemRecentActivityService::new(Arc::new(MemoryRecentActivityStore::default()), session.clone())),
            preferences,
            session,
        ));
        Self { service, balances }
    }
}

pub struct MemoryBalanceStore {
    wallet_id: WalletId,
    pub requests: Mutex<Vec<WalletId>>,
}

#[async_trait]
impl GemBalanceStore for MemoryBalanceStore {
    async fn get_available_balances(&self, wallet_id: WalletId, asset_ids: Vec<AssetId>) -> Result<Vec<GemAssetBalance>, GemServiceError> {
        self.requests.lock().unwrap().push(wallet_id.clone());
        if wallet_id != self.wallet_id {
            return Ok(Vec::new());
        }
        Ok(asset_ids
            .into_iter()
            .map(|asset_id| GemAssetBalance {
                asset_id,
                available: 100u32.into(),
                ..GemAssetBalance::mock()
            })
            .collect())
    }
    async fn update_balances(&self, _: WalletId, _: Vec<GemBalanceRecord>) -> Result<(), GemServiceError> {
        panic!("unexpected balance write")
    }
    async fn get_enabled_asset_ids(&self, _: WalletId) -> Result<Vec<AssetId>, GemServiceError> {
        panic!("unexpected enabled assets read")
    }
    async fn set_assets_enabled(&self, _: WalletId, _: Vec<AssetId>, _: bool) -> Result<(), GemServiceError> {
        panic!("unexpected asset enable")
    }
    async fn set_asset_pinned(&self, _: WalletId, _: AssetId, _: bool) -> Result<(), GemServiceError> {
        panic!("unexpected asset pin")
    }
}

struct MemoryAssetStore;

#[async_trait]
impl GemAssetStore for MemoryAssetStore {
    async fn get_asset_ids(&self, asset_ids: Vec<AssetId>) -> Result<Vec<AssetId>, GemServiceError> {
        Ok(asset_ids)
    }
    async fn get_assets(&self, asset_ids: Vec<AssetId>) -> Result<Vec<Asset>, GemServiceError> {
        Ok(asset_ids.into_iter().map(|id| Asset::from_chain(id.chain)).collect())
    }
    async fn save_assets(&self, _: Vec<AssetBasic>) -> Result<(), GemServiceError> {
        panic!("unexpected asset write")
    }
    async fn save_asset(&self, _: AssetFull) -> Result<(), GemServiceError> {
        panic!("unexpected asset write")
    }
    async fn add_missing_balances(&self, _: WalletId, _: Vec<AssetId>) -> Result<(), GemServiceError> {
        panic!("unexpected balance write")
    }
    async fn add_balances(&self, _: WalletId, _: Vec<AssetId>, _: bool) -> Result<(), GemServiceError> {
        panic!("unexpected balance write")
    }
    async fn set_buyable_assets(&self, _: Vec<AssetId>) -> Result<(), GemServiceError> {
        panic!("unexpected asset write")
    }
    async fn set_sellable_assets(&self, _: Vec<AssetId>) -> Result<(), GemServiceError> {
        panic!("unexpected asset write")
    }
    async fn set_swappable_assets(&self, _: Vec<AssetId>) -> Result<(), GemServiceError> {
        panic!("unexpected asset write")
    }
    async fn set_stakeable_assets(&self, _: Vec<AssetId>) -> Result<(), GemServiceError> {
        panic!("unexpected asset write")
    }
}

struct UnusedStakeStore;

#[async_trait]
impl GemStakeStore for UnusedStakeStore {
    async fn get_apr(&self, _: AssetId, _: StakeProviderType) -> Result<Option<f64>, GemServiceError> {
        panic!("unexpected stake read")
    }
    async fn get_validators(&self, _: AssetId, _: StakeProviderType) -> Result<Vec<DelegationValidator>, GemServiceError> {
        panic!("unexpected stake read")
    }
    async fn save_validators(&self, _: Vec<DelegationValidator>) -> Result<(), GemServiceError> {
        panic!("unexpected stake write")
    }
    async fn deactivate_validators(&self, _: AssetId, _: Vec<String>) -> Result<(), GemServiceError> {
        panic!("unexpected stake write")
    }
    async fn get_delegation_ids(&self, _: WalletId, _: AssetId, _: StakeProviderType) -> Result<Vec<String>, GemServiceError> {
        panic!("unexpected stake read")
    }
    async fn update_delegations(&self, _: WalletId, _: Vec<DelegationBase>, _: Vec<String>) -> Result<(), GemServiceError> {
        panic!("unexpected stake write")
    }
}

struct UnusedSigner;

#[async_trait]
impl GemTransactionSigner for UnusedSigner {
    async fn sign(&self, _: Wallet, _: GemSignerInput) -> Result<Vec<GemSignedTransaction>, GemstoneError> {
        panic!("unexpected signing")
    }
}

struct UnusedTransactionStatus;

impl GemTransactionStatusService for UnusedTransactionStatus {
    fn track(&self, _: WalletId, _: Vec<Transaction>) {
        panic!("unexpected transaction tracking")
    }
}

pub(super) fn confirm_data(chain: Chain, input_type: TransactionInputType, from: &str) -> GemConfirmData {
    GemConfirmData {
        input: GemConfirmInput {
            from: Account::mock(chain, from),
            transfer: GemTransferData {
                input_type,
                recipient: GemRecipient {
                    address: "recipient".to_string(),
                    name: None,
                    memo: Some("memo".to_string()),
                    references: vec![],
                },
                value: BigInt::from(10),
                use_max_amount: true,
            },
        },
        fee: GemTransactionLoadFee {
            fee: BigInt::ZERO,
            gas_price_type: GasPriceType::Regular { gas_price: BigInt::from(5) },
            gas_limit: BigInt::from(21_000),
            options: Default::default(),
            fee_asset: AssetId::from_chain(Chain::Solana),
        },
        selected_priority: FeePriority::Normal,
        fee_rates: vec![],
        metadata: GemTransactionLoadMetadata::None,
        simulation: None,
    }
}
