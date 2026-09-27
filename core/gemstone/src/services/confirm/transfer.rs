use crate::services::simulation::warning_rows;
use std::sync::Arc;
use std::time::Instant;

use primitives::currency::Currency;
use primitives::{Asset, Chain, PerpetualModifyConfirmData, SimulationResult, Wallet, WalletId};

use crate::GemstoneError;
use crate::keystore::{GemKeystore, decode_password, keystore_id_for_wallet};
use crate::message::signer::MessageSigner;
use crate::models::custom_types::GemBigInt;
use crate::models::list::GemListRow;
use crate::models::transaction::GemSignedTransaction;
use crate::payment::{GemPaymentError, GemPaymentService};
use crate::services::confirm::error::sign_error;
use crate::services::confirm::rules::{confirm_row_contents, is_broadcast, is_insufficient_network_fee, max_swap_fit, submit_message};
use crate::services::confirm::{
    ConfirmSwapQuote, GemConfirmData, GemConfirmError, GemConfirmFeeLoad, GemConfirmInput, GemConfirmLoad, GemConfirmRowContent, GemConfirmService, GemConfirmSimulationState, GemConfirmation, GemSubmitResult, SendInput,
};
use crate::services::error_text::{GemErrorText, payment_error_text};
use crate::services::explorer::GemExplorerService;
use crate::services::name::GemNameService;
use crate::services::perpetual::rules::autoclose_row;
use crate::services::preferences::GemPreferencesService;
use crate::services::swap::{GemSwapRequote, GemSwapService};
use crate::services::transfer::rules::TransferInput;
use crate::services::transfer::{GemRecentActivityService, GemTransferData};
use crate::services::wallet::{GemKeystoreAuthentication, GemKeystorePassword};
use primitives::AddressName;
use primitives::BlockExplorerLink;
use primitives::TransactionInputType;
use swapper::Quote;
use zeroize::Zeroizing;

#[derive(uniffi::Object)]
pub struct GemConfirmTransferService {
    confirm: Arc<GemConfirmService>,
    explorer: Arc<GemExplorerService>,
    names: Arc<GemNameService>,
    keystore: Arc<GemKeystore>,
    password: Arc<dyn GemKeystorePassword>,
    recent_activity: Arc<GemRecentActivityService>,
    preferences: Arc<GemPreferencesService>,
    payment: Arc<GemPaymentService>,
    swap: Arc<GemSwapService>,
}

#[uniffi::export]
impl GemConfirmTransferService {
    #[uniffi::constructor]
    pub fn new(
        confirm: Arc<GemConfirmService>,
        explorer: Arc<GemExplorerService>,
        names: Arc<GemNameService>,
        keystore: Arc<GemKeystore>,
        password: Arc<dyn GemKeystorePassword>,
        recent_activity: Arc<GemRecentActivityService>,
        preferences: Arc<GemPreferencesService>,
        payment: Arc<GemPaymentService>,
        swap: Arc<GemSwapService>,
    ) -> Self {
        Self {
            confirm,
            explorer,
            names,
            keystore,
            password,
            recent_activity,
            preferences,
            payment,
            swap,
        }
    }

    pub fn confirmation(self: Arc<Self>, wallet: Wallet, transfer: GemTransferData, simulation: Option<SimulationResult>) -> Arc<GemConfirmation> {
        Arc::new(GemConfirmation::new(self, wallet, transfer, simulation))
    }
}

fn simulation_seed(chain: Chain, simulation: Option<SimulationResult>) -> GemConfirmSimulationState {
    GemConfirmSimulationState {
        chain,
        warnings: simulation.as_ref().map(|result| warning_rows(&result.warnings)).unwrap_or_default(),
        simulation: None,
    }
}

impl GemConfirmTransferService {
    pub(super) fn address_url(&self, chain: Chain, address: String) -> BlockExplorerLink {
        self.explorer.get_address_url(chain, address)
    }

    pub(super) fn row_contents(&self, transfer: GemTransferData, wallet: Wallet, address_name: Option<AddressName>) -> Vec<GemConfirmRowContent> {
        confirm_row_contents(&transfer, wallet, address_name, |chain, address| self.address_url(chain, address))
    }

    pub(super) fn get_currency(&self) -> Currency {
        self.preferences.get_currency()
    }
    pub(super) fn authentication(&self) -> GemKeystoreAuthentication {
        self.password.authentication().unwrap_or(GemKeystoreAuthentication::None)
    }
    pub(super) fn autoclose_row(&self, data: PerpetualModifyConfirmData) -> Option<GemListRow> {
        autoclose_row(&data)
    }
    pub(super) fn payment(&self) -> &GemPaymentService {
        &self.payment
    }
    pub(super) fn confirm(&self) -> &GemConfirmService {
        &self.confirm
    }

    pub(super) async fn refreshed_swap(&self, wallet: &Wallet, transfer: GemTransferData, swap: ConfirmSwapQuote, now: Instant) -> Result<(GemTransferData, ConfirmSwapQuote), GemConfirmError> {
        if !swap.needs_requote(&transfer, now) {
            return Ok((transfer, swap));
        }
        let GemSwapRequote { quote, transfer } = self.swap.requote(wallet, &transfer).await?;
        Ok((transfer, ConfirmSwapQuote::requoted(quote, now)))
    }

    pub(super) async fn fitted_max_swap(&self, wallet: &Wallet, fee: GemConfirmFeeLoad) -> Result<(GemConfirmFeeLoad, Option<Quote>), GemConfirmError> {
        let Some(value) = max_swap_fit(&fee.confirm_data.input.transfer, &fee.metadata, &fee.fee.value) else {
            return Ok((fee, None));
        };
        let GemSwapRequote { quote, transfer } = self.swap.requote_at(wallet, &fee.confirm_data.input.transfer, &value).await?;
        let confirm_data = GemConfirmData {
            input: GemConfirmInput { transfer, ..fee.confirm_data.input },
            ..fee.confirm_data
        };
        let fitted = confirm_data.fee_load(fee.metadata, fee.fee_asset, self.get_currency())?;
        Ok((GemConfirmFeeLoad { simulation: fee.simulation, ..fitted }, Some(quote)))
    }

    pub(super) async fn submit(&self, input: SendInput) -> Result<GemSubmitResult, GemConfirmError> {
        let password = Zeroizing::new(decode_password(&self.password.get_password(false)?));
        let keystore_id = keystore_id_for_wallet(input.wallet.id.id());
        let input = self.with_signed_permit(input, &keystore_id, &password).await?;
        let input_type = &input.confirm.input.transfer.input_type;
        let signed = self.confirm.sign(&input, |chain, signer_input| self.keystore.sign(keystore_id, chain, signer_input, password.to_vec()))?;
        let (transactions, signatures): (Vec<GemSignedTransaction>, Vec<GemSignedTransaction>) = signed.into_iter().partition(|transaction| is_broadcast(input_type, transaction));
        let data: Vec<String> = signatures.iter().map(|transaction| transaction.data.clone()).collect();
        if transactions.is_empty() {
            let warning = self.report_payment(&input, data.clone(), &signatures).await;
            return Ok(GemSubmitResult::Signed {
                data,
                message: submit_message(input_type, warning),
            });
        }
        let hashes = self.confirm.send(&input, transactions).await?;
        let _ = self.recent_activity.add(input_type.clone(), input.wallet.id.clone()).await;
        let warning = self.report_payment(&input, [hashes.clone(), data].concat(), &signatures).await;
        Ok(GemSubmitResult::Sent {
            hashes,
            message: submit_message(input_type, warning),
        })
    }

    async fn report_payment(&self, input: &SendInput, action_results: Vec<String>, signatures: &[GemSignedTransaction]) -> Option<GemErrorText> {
        let input_type = &input.confirm.input.transfer.input_type;
        let TransactionInputType::Payment { .. } = input_type else {
            return None;
        };
        let warning = match self.payment.confirm(input_type, action_results).await {
            Ok(()) => None,
            Err(error @ GemPaymentError::Network { .. }) => Some(payment_error_text(error)),
            Err(error) => return Some(payment_error_text(error)),
        };
        if let Some(hash) = self.payment.record_hash(input_type)
            && !signatures.is_empty()
        {
            let record = SendInput {
                network_fee: GemBigInt::from(0),
                ..input.clone()
            };
            self.confirm.store_pending(&record, &[hash], signatures).await;
        }
        warning
    }

    async fn with_signed_permit(&self, input: SendInput, keystore_id: &str, password: &[u8]) -> Result<SendInput, GemConfirmError> {
        let transfer = &input.confirm.input.transfer;
        let (Some(quote), Some(approval)) = (&input.swap_quote, transfer.input_type.get_swap_data().ok().and_then(|swap| swap.data.permit2.clone())) else {
            return Ok(input);
        };
        let chain = transfer.input_type.get_asset().chain();
        let transfer = self
            .swap
            .transfer_with_permit(&input.wallet, quote, transfer, &approval, |message| {
                let signature = MessageSigner::new(message).sign_with_keystore(self.keystore.clone(), keystore_id.to_string(), password.to_vec());
                primitives::hex::decode_hex(&signature.map_err(|error| sign_error(chain, error))?).map_err(|error| sign_error(chain, GemstoneError::AnyError { msg: error.to_string() }))
            })
            .await?;
        Ok(SendInput {
            confirm: GemConfirmData {
                input: GemConfirmInput { transfer, ..input.confirm.input },
                ..input.confirm
            },
            ..input
        })
    }

    pub(super) fn confirm_input(&self, wallet: Wallet, transfer: GemTransferData) -> Result<GemConfirmInput, GemConfirmError> {
        let chain = transfer.input_type.get_asset().chain();
        let from = wallet.account(chain).cloned().ok_or(GemConfirmError::AccountMissing { chain })?;
        Ok(GemConfirmInput { from, transfer })
    }

    pub(super) async fn state(&self, wallet_id: WalletId, input: &GemConfirmInput, simulation: Option<SimulationResult>) -> Result<GemConfirmLoad, GemConfirmError> {
        let input_type = input.transfer.input_type.clone();
        let chain = input_type.transaction_asset().chain();
        let (metadata, fee_assets, address_name) = futures::join!(
            self.confirm.input_metadata(wallet_id.clone(), &input_type, input_type.fee_asset().id),
            self.confirm.fee_assets(wallet_id, chain, self.get_currency()),
            self.names.address_name(chain, input.transfer.recipient.address.clone()),
        );
        Ok(GemConfirmLoad {
            transfer: input.transfer.clone(),
            sender: input.from.clone(),
            fee_asset: input_type.fee_asset(),
            metadata: metadata?,
            fee_assets: fee_assets?,
            simulation: simulation_seed(chain, simulation),
            address_name: address_name.unwrap_or_default(),
            fee: None,
        })
    }

    pub(super) async fn simulation_state(&self, input_type: TransactionInputType, simulation: SimulationResult) -> Result<GemConfirmSimulationState, GemConfirmError> {
        let chain = input_type.transaction_asset().chain();
        let assets = self.confirm.ensure_simulation_assets(simulation.asset_ids()).await?;
        let Ok(details) = self.confirm.simulation(input_type, Some(simulation.clone()), assets, |chain, address| self.address_url(chain, address)) else {
            return Ok(simulation_seed(chain, Some(simulation)));
        };
        let requests = details.address_requests(chain);
        let address_names = self.names.get_address_names(requests).await.unwrap_or_default();
        Ok(GemConfirmSimulationState {
            chain,
            warnings: warning_rows(&simulation.warnings),
            simulation: Some(details.with_address_names(&address_names)),
        })
    }

    pub(super) async fn missing_network_fee(&self, wallet_id: WalletId, input_type: TransactionInputType) -> Option<GemConfirmError> {
        let balance = self.confirm.input_metadata(wallet_id, &input_type, input_type.fee_asset().id).await.ok()?.fee_asset_balance;
        is_insufficient_network_fee(&balance.asset_id, &balance.available).then(|| GemConfirmError::InsufficientNetworkFee {
            asset: Asset::from_chain(balance.asset_id.chain),
            requirement: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    use futures::executor::block_on;
    use primitives::swap::{Permit2ApprovalData, SwapData, SwapQuoteData};
    use primitives::{Account, Asset, Chain, Currency, SwapProvider, TransactionInputType, Wallet, WalletId};
    use swapper::testkit::MockSwapper;
    use swapper::{FetchQuoteData, Quote, SwapperProvider};

    use super::super::testkit::ConfirmTestkit;
    use crate::keystore::{GemImportType, decode_password};
    use crate::models::transaction::{GemTransactionLoadFee, GemTransactionLoadMetadata};
    use crate::services::confirm::{ConfirmSwapQuote, GemConfirmData, GemConfirmError, GemConfirmFeeLoad, GemConfirmInput, GemConfirmMetadata, GemTransferAmountResult, SendInput};
    use crate::services::swap::GemSwapService;
    use crate::services::transfer::{GemRecipient, GemTransferData};
    use crate::services::wallet::testkit::TEST_PASSWORD;

    fn swap_transfer(address: &str, permit2: Option<Permit2ApprovalData>) -> GemTransferData {
        GemTransferData {
            recipient: GemRecipient::address(address.to_string()),
            value: 5.into(),
            ..GemTransferData::mock(TransactionInputType::Swap {
                from_asset: Asset::mock_eth(),
                to_asset: Asset::mock_erc20(),
                swap_data: SwapData {
                    data: SwapQuoteData { permit2, ..SwapQuoteData::mock() },
                    ..SwapData::mock_with_provider(SwapProvider::UniswapV3)
                },
            })
        }
    }

    fn keystore_wallet(testkit: &ConfirmTestkit) -> Wallet {
        let stored = testkit.keystore.create_store(GemImportType::mock_private_key(), decode_password(TEST_PASSWORD)).unwrap();
        Wallet {
            id: WalletId::from_id(&stored.wallet_id).unwrap(),
            ..Wallet::mock_with_accounts(vec![Account::mock(Chain::Ethereum, &stored.accounts[0].address)])
        }
    }

    fn send_input(wallet: Wallet, transfer: GemTransferData, swap_quote: Option<Quote>) -> SendInput {
        let address = wallet.accounts[0].address.clone();
        let confirm = GemConfirmData::mock(Chain::Ethereum, transfer.input_type.clone());
        SendInput {
            wallet,
            confirm: GemConfirmData {
                input: GemConfirmInput {
                    from: Account::mock(Chain::Ethereum, &address),
                    transfer,
                },
                metadata: GemTransactionLoadMetadata::Evm { nonce: 0, chain_id: 1, contract_call: None },
                ..confirm
            },
            swap_quote,
            ..SendInput::mock(Chain::Ethereum, TransactionInputType::Transfer { asset: Asset::mock_eth() })
        }
    }

    fn broadcast_failed(error: &GemConfirmError) -> bool {
        matches!(error, GemConfirmError::Offline | GemConfirmError::Network { .. } | GemConfirmError::Broadcast { .. })
    }

    #[test]
    fn test_a_submit_signs_in_core_with_one_password_read() {
        block_on(async {
            let testkit = ConfirmTestkit::new(Wallet::mock_with_chains(&[Chain::Ethereum]), Wallet::mock_with_chains(&[Chain::Ethereum]));
            let wallet = keystore_wallet(&testkit);
            let address = wallet.accounts[0].address.clone();
            let transfer = GemTransferData {
                recipient: GemRecipient::address(address),
                ..GemTransferData::mock(TransactionInputType::Transfer { asset: Asset::mock_eth() })
            };

            let error = testkit.service.submit(send_input(wallet, transfer, None)).await.unwrap_err();

            assert_eq!(testkit.passwords.create_requests.lock().unwrap().clone(), vec![false], "the password is read once for the whole submit");
            assert!(broadcast_failed(&error), "signing succeeded and only the broadcast failed: {error:?}");
        });
    }

    #[test]
    fn test_a_swap_load_asks_its_provider_again_for_the_transfer_it_holds() {
        block_on(async {
            let swapper = MockSwapper::new(SwapperProvider::UniswapV3, |request| Ok(Quote::mock_with_request(request))).with_pending_permit(Permit2ApprovalData::mock());
            let builds = swapper.builds();
            let wallet = Wallet::mock_with_chains(&[Chain::Ethereum]);
            let testkit = ConfirmTestkit::with_swap(wallet.clone(), Arc::new(GemSwapService::mock_with_swappers(vec![Box::new(swapper)])));
            let transfer = swap_transfer(&wallet.accounts[0].address, None);

            let built_at = Instant::now();
            let unquoted = ConfirmSwapQuote::initial(&transfer, built_at).unwrap();

            let (refreshed, swap) = testkit.service.refreshed_swap(&wallet, transfer.clone(), unquoted.clone(), built_at + Duration::from_secs(60)).await.unwrap();

            let quote = swap.quote.unwrap();
            assert_eq!(swap.quoted_at, built_at + Duration::from_secs(60), "the quote's age restarts at the re-quote");
            assert_eq!(quote.request.value, 5u32.into(), "the provider is asked for the amount the transfer carries");
            assert_eq!(quote.data.provider.id, SwapperProvider::UniswapV3);
            assert_eq!(builds.lock().unwrap().clone(), vec![FetchQuoteData::None], "a load builds without a signature");
            assert_eq!(
                refreshed.input_type.get_swap_data().unwrap().data.permit2,
                Some(Permit2ApprovalData::mock()),
                "the pending permit the provider reports rides in the refreshed data"
            );
            assert_eq!(refreshed.input_type.get_swap_data().unwrap().quote.from_value, 5u32.into());

            let (kept, swap) = testkit.service.refreshed_swap(&wallet, transfer.clone(), unquoted, built_at).await.unwrap();
            assert!(swap.quote.is_none() && swap.quoted_at == built_at, "a quote younger than its lifetime is kept as it is");
            assert_eq!(kept.input_type.get_swap_data().unwrap().data, transfer.input_type.get_swap_data().unwrap().data);
            assert_eq!(builds.lock().unwrap().len(), 1, "nothing was built for the kept quote");
        });
    }

    #[test]
    fn test_a_max_swap_that_needs_the_exact_amount_is_asked_again_for_everything_but_the_fee() {
        block_on(async {
            let swapper = MockSwapper::new(SwapperProvider::UniswapV3, |request| Ok(Quote::mock_with_request(request)));
            let builds = swapper.builds();
            let wallet = Wallet::mock_with_chains(&[Chain::Ethereum]);
            let testkit = ConfirmTestkit::with_swap(wallet.clone(), Arc::new(GemSwapService::mock_with_swappers(vec![Box::new(swapper)])));
            let max_swap = |swap_data: SwapData, value: u32| {
                let mut confirm = GemConfirmData::mock(
                    Chain::Ethereum,
                    TransactionInputType::Swap {
                        from_asset: Asset::mock_eth(),
                        to_asset: Asset::mock_erc20(),
                        swap_data,
                    },
                );
                confirm.input.transfer.value = value.into();
                confirm.fee = GemTransactionLoadFee {
                    fee_asset: Asset::mock_eth().id,
                    ..GemTransactionLoadFee::mock(10)
                };
                confirm.fee_load(GemConfirmMetadata::mock(&Asset::mock_eth().id, 1_000), Asset::mock_eth(), Currency::USD).unwrap()
            };
            let sent = |fee: &GemConfirmFeeLoad| match &fee.fee.amount {
                GemTransferAmountResult::Amount { amount } => Some((amount.value.clone(), amount.network_fee.clone())),
                GemTransferAmountResult::Error { .. } => None,
            };

            let reserved = max_swap(SwapData::mock_with_provider(SwapProvider::UniswapV3), 995);
            assert_eq!(sent(&reserved), None, "the reserve the swap screen kept back is less than the fee, so the amount does not fit yet");

            let (fitted, quote) = testkit.service.fitted_max_swap(&wallet, reserved).await.unwrap();

            assert_eq!(quote.unwrap().request.value, 990u32.into(), "the provider is asked again for everything but the fee");
            assert_eq!(fitted.confirm_data.input.transfer.value, 990.into());
            assert_eq!(sent(&fitted), Some((990.into(), 10.into())), "what leaves the wallet plus the fee is the whole balance");
            assert_eq!(builds.lock().unwrap().clone(), vec![FetchQuoteData::None]);

            let deposit = max_swap(SwapData::mock_transfer(SwapProvider::NearIntents, "1000", "1", "0xdeposit"), 1_000);
            let (kept, quote) = testkit.service.fitted_max_swap(&wallet, deposit).await.unwrap();

            assert!(quote.is_none(), "a deposit is not asked again; the fee comes off at signing");
            assert_eq!(kept.confirm_data.input.transfer.value, 1_000.into());
            assert_eq!(sent(&kept), Some((990.into(), 10.into())));
            assert_eq!(builds.lock().unwrap().len(), 1);
        });
    }

    #[test]
    fn test_a_submit_signs_a_pending_permit_and_rebuilds_with_it_under_the_same_password_read() {
        block_on(async {
            let swapper = MockSwapper::new(SwapperProvider::UniswapV3, |request| Ok(Quote::mock_with_request(request)));
            let builds = swapper.builds();
            let testkit = ConfirmTestkit::with_swap(Wallet::mock_with_chains(&[Chain::Ethereum]), Arc::new(GemSwapService::mock_with_swappers(vec![Box::new(swapper)])));
            let wallet = keystore_wallet(&testkit);
            let transfer = swap_transfer(&wallet.accounts[0].address, Some(Permit2ApprovalData::mock()));
            let quote = Quote::mock_with_request(&crate::services::swap::rules::requote_request(&wallet, &transfer, &transfer.value).unwrap().1);

            let error = testkit.service.submit(send_input(wallet, transfer, Some(quote))).await.unwrap_err();

            assert_eq!(testkit.passwords.create_requests.lock().unwrap().clone(), vec![false], "the permit and the transaction share one password read");
            let builds = builds.lock().unwrap().clone();
            let [FetchQuoteData::Permit2(permit)] = builds.as_slice() else {
                panic!("the swap is rebuilt once, with the signed permit: {builds:?}");
            };
            assert_eq!(permit.permit_single.details.token, Permit2ApprovalData::mock().token);
            assert_eq!(permit.permit_single.details.nonce, Permit2ApprovalData::mock().permit2_nonce);
            assert_eq!(permit.signature.len(), 65);
            assert!(broadcast_failed(&error), "signing succeeded and only the broadcast failed: {error:?}");
        });
    }

    #[test]
    fn test_a_submit_signs_the_loaded_swap_data_when_no_permit_is_pending() {
        block_on(async {
            let swapper = MockSwapper::new(SwapperProvider::UniswapV3, |request| Ok(Quote::mock_with_request(request)));
            let builds = swapper.builds();
            let testkit = ConfirmTestkit::with_swap(Wallet::mock_with_chains(&[Chain::Ethereum]), Arc::new(GemSwapService::mock_with_swappers(vec![Box::new(swapper)])));
            let wallet = keystore_wallet(&testkit);
            let transfer = swap_transfer(&wallet.accounts[0].address, None);
            let quote = Quote::mock_with_request(&crate::services::swap::rules::requote_request(&wallet, &transfer, &transfer.value).unwrap().1);

            let error = testkit.service.submit(send_input(wallet, transfer, Some(quote))).await.unwrap_err();

            assert!(builds.lock().unwrap().is_empty(), "the data the screen loaded is what gets signed");
            assert_eq!(testkit.passwords.create_requests.lock().unwrap().clone(), vec![false]);
            assert!(broadcast_failed(&error), "signing succeeded and only the broadcast failed: {error:?}");
        });
    }
}
