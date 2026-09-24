use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use primitives::{Asset, AssetBasic, AssetFull, AssetId, Chain, Transaction, Wallet, WalletId};

use super::{
    GemConfirmData, GemConfirmFee, GemConfirmFeeSelection, GemConfirmInput, GemConfirmLoad, GemConfirmMetadata, GemConfirmService, GemConfirmSimulationState, GemConfirmTransferService, GemTransactionSigner, GemTransferAmountResult,
    SendInput,
};
use crate::GemstoneError;
use crate::api::{GemApiClient, GemDeviceApiClient, GemStaticApiClient};
use crate::gateway::GemGateway;
use crate::models::transaction::{GemSignedTransaction, GemSignerInput, GemTransactionLoadFee, GemTransactionLoadMetadata};
use crate::payment::GemPaymentService;
use crate::services::assets::{GemAssetStore, GemAssetsService};
use crate::services::balance::testkit::MemoryBalanceStore;
use crate::services::balance::{GemAssetBalance, GemBalanceService};
use crate::services::device::GemDeviceKeyService;
use crate::services::error::GemServiceError;
use crate::services::explorer::GemExplorerService;
use crate::services::name::GemNameService;
use crate::services::nft::{GemNftService, testkit::MemoryNftStore};
use crate::services::node::GemNodeService;
use crate::services::preferences::{GemPreferencesService, testkit::MemoryPreferencesStore};
use crate::services::price::{GemPriceService, testkit::MemoryPriceStore};
use crate::services::stake::GemStakeService;
use crate::services::stake::testkit::UnusedStakeStore;
use crate::services::stream::testkit::SubscriptionTestkit;
use crate::services::transaction_state::{GemTransactionStateService, GemTransactionStatusService, testkit::MemoryTransactionStateStore};
use crate::services::transfer::GemTransferData;
use crate::services::transfer::{GemRecentActivityService, testkit::MemoryRecentActivityStore};
use crate::services::wallet::testkit::{MemoryAddressStore, MemoryKeystorePassword, MemoryWalletStore};
use crate::services::wallet_session::{GemWalletSessionService, testkit::MemoryWalletSessionStore};
use crate::services::{GemScanService, GemSimulationService};
use crate::testkit::{EmptyPreferences, TestAlienProvider};
use crate::transfer_amount::GemTransferAmount;
use num_bigint::BigInt;
use primitives::{Account, FeePriority, GasPriceType, TransactionInputType};

pub struct ConfirmTestkit {
    pub service: Arc<GemConfirmTransferService>,
    pub confirm: Arc<GemConfirmService>,
    pub balances: Arc<MemoryBalanceStore>,
}

impl ConfirmTestkit {
    pub fn new(wallet: Wallet, selected_wallet: Wallet) -> Self {
        Self::with_provider(wallet, selected_wallet, Arc::new(TestAlienProvider::with_status(503)))
    }

    pub fn with_provider(wallet: Wallet, selected_wallet: Wallet, provider: Arc<TestAlienProvider>) -> Self {
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
        let gateway = Arc::new(GemGateway::new(provider.clone(), Arc::new(GemNodeService::mock()), preferences_store, Arc::new(EmptyPreferences)));
        let api = Arc::new(GemApiClient::new(provider.clone()));
        let device_api = Arc::new(GemDeviceApiClient::new(provider.clone(), Arc::new(GemDeviceKeyService::new(Arc::new(EmptyPreferences)))));
        let price = Arc::new(GemPriceService::mock(Arc::new(MemoryPriceStore::default())));
        let asset_store = Arc::new(MemoryAssetStore);
        let assets = Arc::new(GemAssetsService::new(api, gateway.clone(), asset_store.clone(), price.clone(), preferences.clone(), session.clone()));
        let balances = Arc::new(MemoryBalanceStore::with_balances(
            wallet.id.clone(),
            wallet
                .accounts
                .iter()
                .map(|account| GemAssetBalance {
                    asset_id: AssetId::from_chain(account.chain),
                    ..GemAssetBalance::mock_with_available(100)
                })
                .collect(),
        ));
        let balance = Arc::new(GemBalanceService::new(gateway.clone(), balances.clone(), assets.clone(), session.clone(), Arc::new(SubscriptionTestkit::new(&[], &[]).service)));
        let explorer = Arc::new(GemExplorerService::new(preferences.clone()));
        let names = Arc::new(GemNameService::new(device_api.clone(), Arc::new(MemoryAddressStore::default())));
        let stake = Arc::new(GemStakeService::new(
            gateway.clone(),
            Arc::new(GemStaticApiClient::new(provider.clone())),
            Arc::new(UnusedStakeStore),
            names.clone(),
            explorer.clone(),
            preferences.clone(),
            session.clone(),
        ));
        let nft = Arc::new(GemNftService::new(device_api.clone(), Arc::new(MemoryNftStore::default()), session.clone()));
        let payment = Arc::new(GemPaymentService::new(provider.clone(), assets.clone()));
        let transactions = Arc::new(GemTransactionStateService::new(
            gateway.clone(),
            Arc::new(MemoryTransactionStateStore::default()),
            assets.clone(),
            balance.clone(),
            stake,
            nft,
            payment.clone(),
        ));
        let confirm = Arc::new(GemConfirmService::new(
            gateway,
            Arc::new(GemSimulationService::new(provider, Arc::new(GemNodeService::mock()))),
            Arc::new(GemScanService::new(device_api.clone())),
            transactions,
            balance,
            price,
            assets,
            Arc::new(UnusedTransactionStatus),
        ));
        let service = Arc::new(GemConfirmTransferService::new(
            confirm.clone(),
            explorer,
            names,
            Arc::new(UnusedSigner),
            Arc::new(MemoryKeystorePassword::default()),
            Arc::new(GemRecentActivityService::new(Arc::new(MemoryRecentActivityStore::default()), session)),
            preferences,
            payment,
        ));
        Self { service, confirm, balances }
    }
}

struct MemoryAssetStore;

#[async_trait]
impl GemAssetStore for MemoryAssetStore {
    async fn get_asset_ids(&self, asset_ids: Vec<AssetId>) -> Result<Vec<AssetId>, GemServiceError> {
        Ok(asset_ids)
    }
    async fn get_asset_basics(&self, _asset_ids: Vec<AssetId>) -> Result<Vec<AssetBasic>, GemServiceError> {
        Ok(vec![])
    }
    async fn get_assets(&self, asset_ids: Vec<AssetId>) -> Result<Vec<Asset>, GemServiceError> {
        Ok(asset_ids.into_iter().map(|id| Asset::from_chain(id.chain)).collect())
    }
    async fn get_wallet_assets(&self, _: WalletId) -> Result<Vec<Asset>, GemServiceError> {
        Ok(vec![])
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

impl GemConfirmData {
    pub fn mock(chain: Chain, input_type: TransactionInputType) -> Self {
        GemConfirmData {
            input: GemConfirmInput {
                from: Account::mock(chain, "sender"),
                transfer: GemTransferData {
                    use_max_amount: true,
                    ..GemTransferData::mock(input_type)
                },
            },
            fee: GemTransactionLoadFee {
                gas_price_type: GasPriceType::regular(5),
                fee_asset: AssetId::from_chain(Chain::Solana),
                ..GemTransactionLoadFee::mock(0)
            },
            selected_priority: FeePriority::Normal,
            fee_selection: GemConfirmFeeSelection::Priority { priority: FeePriority::Normal },
            fee_rates: vec![],
            metadata: GemTransactionLoadMetadata::None,
            simulation: None,
        }
    }
}

impl SendInput {
    pub fn mock(chain: Chain, input_type: TransactionInputType) -> Self {
        SendInput {
            wallet: Wallet {
                id: WalletId::Multicoin("wallet".to_string()),
                ..Wallet::mock_with_accounts(vec![Account::mock(chain, "sender")])
            },
            confirm: GemConfirmData::mock(chain, input_type),
            value: BigInt::from(9),
            network_fee: BigInt::from(1),
            simulation: None,
        }
    }
}

impl GemConfirmMetadata {
    pub fn mock(asset_id: &AssetId, available: u64) -> Self {
        let balance = GemAssetBalance {
            asset_id: asset_id.clone(),
            ..GemAssetBalance::mock_with_available(available)
        };
        GemConfirmMetadata {
            asset_balance: balance.clone(),
            fee_asset_balance: balance,
            prices: vec![],
        }
    }
}

impl GemConfirmFee {
    pub fn mock(amount: GemTransferAmountResult) -> Self {
        GemConfirmFee {
            value: BigInt::from(1),
            additional_fees: vec![],
            selected_priority: FeePriority::Normal,
            amount,
        }
    }
}

impl GemTransferAmountResult {
    pub fn mock() -> Self {
        GemTransferAmountResult::Amount {
            amount: GemTransferAmount {
                value: BigInt::from(1),
                network_fee: BigInt::from(1),
                is_max_amount: false,
            },
        }
    }
}

impl GemConfirmSimulationState {
    pub fn mock() -> Self {
        GemConfirmSimulationState {
            chain: Chain::Ethereum,
            warnings: vec![],
            simulation: None,
        }
    }
}

impl GemConfirmLoad {
    pub fn mock() -> Self {
        let eth = Asset::mock_eth();
        GemConfirmLoad {
            transfer: GemTransferData::mock(TransactionInputType::Transfer { asset: eth.clone() }),
            sender: Account::mock(Chain::Ethereum, "sender"),
            metadata: GemConfirmMetadata::mock(&eth.id, 0),
            fee_asset: eth,
            fee_assets: vec![],
            simulation: GemConfirmSimulationState::mock(),
            address_name: None,
            fee: None,
        }
    }
}
