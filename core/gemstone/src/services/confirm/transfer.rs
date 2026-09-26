use crate::services::simulation::warning_rows;
use std::sync::Arc;

use primitives::currency::Currency;
use primitives::{Asset, Chain, PerpetualModifyConfirmData, SimulationResult, Wallet, WalletId};

use crate::keystore::{GemKeystore, decode_password, keystore_id_for_wallet};
use crate::message::signer::MessageSigner;
use crate::models::custom_types::GemBigInt;
use crate::models::list::GemListRow;
use crate::models::transaction::GemSignedTransaction;
use crate::payment::{GemPaymentError, GemPaymentService};
use crate::services::confirm::error::{sign_error, swap_error};
use crate::services::confirm::rules::{confirm_row_contents, is_broadcast, is_insufficient_network_fee, submit_message};
use crate::services::confirm::{GemConfirmData, GemConfirmError, GemConfirmInput, GemConfirmLoad, GemConfirmRowContent, GemConfirmService, GemConfirmSimulationState, GemConfirmation, GemSubmitResult, SendInput};
use crate::services::error_text::{GemErrorText, payment_error_text};
use crate::services::explorer::GemExplorerService;
use crate::services::name::GemNameService;
use crate::services::perpetual::rules::autoclose_row;
use crate::services::preferences::GemPreferencesService;
use crate::services::swap::GemSwapService;
use crate::services::transfer::rules::TransferInput;
use crate::services::transfer::{GemRecentActivityService, GemTransferData};
use crate::services::wallet::{GemKeystoreAuthentication, GemKeystorePassword};
use primitives::AddressName;
use primitives::BlockExplorerLink;
use primitives::TransactionInputType;
use swapper::permit2_data::Permit2Data;
use swapper::{FetchQuoteData, Quote};
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

    pub(super) async fn fresh_transfer(&self, wallet: &Wallet, transfer: GemTransferData) -> Result<(GemTransferData, Option<Quote>), GemConfirmError> {
        match transfer.input_type {
            TransactionInputType::Swap { .. } => {
                let (quote, transfer) = self.swap.requote(wallet, &transfer).await.map_err(swap_error)?;
                Ok((transfer, Some(quote)))
            }
            _ => Ok((transfer, None)),
        }
    }

    pub(super) async fn submit(&self, input: SendInput) -> Result<GemSubmitResult, GemConfirmError> {
        let password = Zeroizing::new(decode_password(&self.password.get_password(false)?));
        let keystore_id = keystore_id_for_wallet(input.wallet.id.id());
        let input = self.signed_swap_input(input, &keystore_id, &password).await?;
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

    async fn signed_swap_input(&self, input: SendInput, keystore_id: &str, password: &[u8]) -> Result<SendInput, GemConfirmError> {
        let Some(quote) = &input.swap_quote else {
            return Ok(input);
        };
        let transfer = &input.confirm.input.transfer;
        let data = match transfer.input_type.get_swap_data().ok().and_then(|swap| swap.data.permit2.clone()) {
            Some(approval) => {
                let chain = transfer.input_type.get_asset().chain();
                let (permit_single, message) = self.swap.permit2_message(quote, &approval).map_err(swap_error)?;
                let signature = MessageSigner::new(message)
                    .sign_with_keystore(self.keystore.clone(), keystore_id.to_string(), password.to_vec())
                    .map_err(|error| sign_error(chain, error))?;
                let signature = primitives::hex::decode_hex(&signature).map_err(|error| GemConfirmError::Load { msg: error.to_string() })?;
                FetchQuoteData::Permit2(Permit2Data { permit_single, signature })
            }
            None => FetchQuoteData::None,
        };
        let transfer = self.swap.rebuild(&input.wallet, quote, transfer, data).await.map_err(swap_error)?;
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
    use futures::executor::block_on;
    use primitives::{Account, Asset, Chain, TransactionInputType, Wallet, WalletId};

    use super::super::testkit::ConfirmTestkit;
    use crate::keystore::{GemImportType, decode_password};
    use crate::models::transaction::GemTransactionLoadMetadata;
    use crate::services::confirm::{GemConfirmData, GemConfirmError, GemConfirmInput, SendInput};
    use crate::services::transfer::{GemRecipient, GemTransferData};
    use crate::services::wallet::testkit::TEST_PASSWORD;

    #[test]
    fn test_a_submit_signs_in_core_with_one_password_read() {
        block_on(async {
            let testkit = ConfirmTestkit::new(Wallet::mock_with_chains(&[Chain::Ethereum]), Wallet::mock_with_chains(&[Chain::Ethereum]));
            let stored = testkit.keystore.create_store(GemImportType::mock_private_key(), decode_password(TEST_PASSWORD)).unwrap();
            let address = stored.accounts[0].address.clone();
            let wallet = Wallet {
                id: WalletId::from_id(&stored.wallet_id).unwrap(),
                ..Wallet::mock_with_accounts(vec![Account::mock(Chain::Ethereum, &address)])
            };
            let confirm = GemConfirmData::mock(Chain::Ethereum, TransactionInputType::Transfer { asset: Asset::mock_eth() });
            let input = SendInput {
                wallet,
                confirm: GemConfirmData {
                    input: GemConfirmInput {
                        from: Account::mock(Chain::Ethereum, &address),
                        transfer: GemTransferData {
                            recipient: GemRecipient::address(address.clone()),
                            ..confirm.input.transfer
                        },
                    },
                    metadata: GemTransactionLoadMetadata::Evm { nonce: 0, chain_id: 1, contract_call: None },
                    ..confirm
                },
                ..SendInput::mock(Chain::Ethereum, TransactionInputType::Transfer { asset: Asset::mock_eth() })
            };

            let error = testkit.service.submit(input).await.unwrap_err();

            assert_eq!(testkit.passwords.create_requests.lock().unwrap().clone(), vec![false], "the password is read once for the whole submit");
            assert!(
                matches!(error, GemConfirmError::Offline | GemConfirmError::Network { .. } | GemConfirmError::Broadcast { .. }),
                "signing succeeded and only the broadcast failed: {error:?}"
            );
        });
    }
}
