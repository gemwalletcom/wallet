use std::time::Instant;

use primitives::swap::SwapQuoteDataType;
use primitives::{TransactionInputType, Wallet};
use swapper::fees::max_amount_spends_all_but_fee;
use swapper::{ProviderType, Quote};

use super::error::sign_error;
use super::{GemConfirmData, GemConfirmError, GemConfirmFeeLoad, GemConfirmInput, GemConfirmMetadata, GemConfirmTransferService, SendInput};
use crate::GemstoneError;
use crate::message::signer::MessageSigner;
use crate::models::custom_types::GemBigInt;
use crate::services::swap::GemSwapRequote;
use crate::services::transfer::GemTransferData;

#[derive(Debug, Clone)]
pub struct ConfirmSwapQuote {
    pub quote: Option<Quote>,
    pub quoted_at: Instant,
}

impl ConfirmSwapQuote {
    pub(super) fn initial(transfer: &GemTransferData, built_at: Instant) -> Option<Self> {
        match transfer.input_type {
            TransactionInputType::Swap { .. } => Some(Self { quote: None, quoted_at: built_at }),
            _ => None,
        }
    }

    pub(super) fn requoted(quote: Quote, now: Instant) -> Self {
        Self { quote: Some(quote), quoted_at: now }
    }
}

impl ConfirmSwapQuote {
    pub(super) fn needs_requote(&self, transfer: &GemTransferData, now: Instant) -> bool {
        let TransactionInputType::Swap { swap_data, .. } = &transfer.input_type else {
            return false;
        };
        if self.quote.is_none() && swap_data.data.permit2.is_some() {
            return true;
        }
        now.saturating_duration_since(self.quoted_at) >= ProviderType::mode(swap_data.quote.provider_data.provider).quote_lifetime()
    }
}

pub(super) fn max_swap_fit(transfer: &GemTransferData, metadata: &GemConfirmMetadata, fee: &GemBigInt) -> Option<GemBigInt> {
    let TransactionInputType::Swap { from_asset, swap_data, .. } = &transfer.input_type else {
        return None;
    };
    let exact_native = from_asset.id.is_native() && swap_data.data.data_type == SwapQuoteDataType::Contract;
    if !transfer.use_max_amount || !exact_native || !max_amount_spends_all_but_fee(from_asset.chain()) {
        return None;
    }
    let target = GemBigInt::from(metadata.asset_balance.available.clone()) - fee;
    (target > GemBigInt::ZERO && target != transfer.value).then_some(target)
}

impl GemConfirmTransferService {
    pub(super) async fn refreshed_swap(&self, wallet: &Wallet, transfer: GemTransferData, swap: Option<ConfirmSwapQuote>, now: Instant) -> Result<(GemTransferData, Option<ConfirmSwapQuote>), GemConfirmError> {
        let Some(swap) = swap else {
            return Ok((transfer, None));
        };
        if !swap.needs_requote(&transfer, now) {
            return Ok((transfer, Some(swap)));
        }
        let GemSwapRequote { quote, transfer } = self.swap.requote(wallet, &transfer).await?;
        Ok((transfer, Some(ConfirmSwapQuote::requoted(quote, now))))
    }

    pub(super) async fn fitted_max_swap(&self, wallet: &Wallet, fee: GemConfirmFeeLoad, swap: Option<ConfirmSwapQuote>, now: Instant) -> Result<(GemConfirmFeeLoad, Option<ConfirmSwapQuote>), GemConfirmError> {
        let Some(value) = max_swap_fit(&fee.confirm_data.input.transfer, &fee.metadata, &fee.fee.value) else {
            return Ok((fee, swap));
        };
        let GemSwapRequote { quote, transfer } = self.swap.requote_at(wallet, &fee.confirm_data.input.transfer, &value).await?;
        let confirm_data = GemConfirmData {
            input: GemConfirmInput { transfer, ..fee.confirm_data.input },
            ..fee.confirm_data
        };
        let fitted = confirm_data.fee_load(fee.metadata, fee.fee_asset, self.get_currency())?;
        Ok((GemConfirmFeeLoad { simulation: fee.simulation, ..fitted }, Some(ConfirmSwapQuote::requoted(quote, now))))
    }

    pub(super) async fn with_signed_permit(&self, input: SendInput, keystore_id: &str, password: &[u8]) -> Result<SendInput, GemConfirmError> {
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
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    use futures::executor::block_on;
    use primitives::swap::{Permit2ApprovalData, SwapData};
    use primitives::{Asset, Chain, Currency, SwapProvider, TransactionInputType, Wallet};
    use swapper::testkit::MockSwapper;
    use swapper::{FetchQuoteData, Quote, SwapperProvider};

    use super::*;
    use crate::models::transaction::GemTransactionLoadFee;
    use crate::services::confirm::GemTransferAmountResult;
    use crate::services::confirm::testkit::{ConfirmTestkit, broadcast_failed, mock_swap_transfer};
    use crate::services::swap::GemSwapService;

    #[test]
    fn test_a_max_swap_of_a_network_coin_is_fitted_to_the_fee_only_where_the_fee_is_the_whole_cost() {
        use primitives::SwapProvider;
        use primitives::swap::SwapData;
        let swap = |from_asset: Asset, swap_data: SwapData, use_max_amount: bool, value: u32| GemTransferData {
            value: value.into(),
            use_max_amount,
            ..GemTransferData::mock(TransactionInputType::Swap {
                from_asset,
                to_asset: Asset::mock_erc20(),
                swap_data,
            })
        };
        let metadata = GemConfirmMetadata::mock(&Asset::mock_eth().id, 1_000);
        let fee = GemBigInt::from(10);
        let contract = SwapData::mock_with_provider(SwapProvider::UniswapV3);
        let deposit = SwapData::mock_transfer(SwapProvider::NearIntents, "1000", "1", "0xdeposit");

        assert_eq!(
            max_swap_fit(&swap(Asset::mock_eth(), contract.clone(), true, 995), &metadata, &fee),
            Some(990.into()),
            "the reserve the quote kept back becomes everything but the fee"
        );
        assert_eq!(max_swap_fit(&swap(Asset::mock_eth(), contract.clone(), true, 990), &metadata, &fee), None, "an amount that already fits is left alone");
        assert_eq!(max_swap_fit(&swap(Asset::mock_eth(), contract.clone(), false, 500), &metadata, &fee), None, "only a max amount is fitted");
        assert_eq!(max_swap_fit(&swap(Asset::mock_eth(), deposit, true, 1_000), &metadata, &fee), None, "a deposit takes the fee off at signing instead");
        assert_eq!(max_swap_fit(&swap(Asset::mock_erc20(), contract.clone(), true, 995), &metadata, &fee), None, "a token pays its fee in the coin");
        assert_eq!(
            max_swap_fit(&swap(Asset::mock_sol(), contract.clone(), true, 995), &GemConfirmMetadata::mock(&Asset::mock_sol().id, 1_000), &fee),
            None,
            "a network that charges more than its fee keeps its reserve"
        );
        assert_eq!(
            max_swap_fit(&swap(Asset::mock_eth(), contract, true, 995), &metadata, &GemBigInt::from(1_000)),
            None,
            "a fee that eats the balance is left to the amount check"
        );
    }

    #[test]
    fn test_a_confirm_swap_quote_is_asked_again_past_its_provider_lifetime_or_once_for_a_pending_permit() {
        use primitives::swap::{Permit2ApprovalData, SwapData, SwapQuoteData};
        use std::time::Duration;
        use swapper::Quote;
        let swap = |provider: primitives::SwapProvider, permit2: Option<Permit2ApprovalData>| {
            GemTransferData::mock(TransactionInputType::Swap {
                from_asset: Asset::mock_eth(),
                to_asset: Asset::mock_erc20(),
                swap_data: SwapData {
                    data: SwapQuoteData { permit2, ..SwapQuoteData::mock() },
                    ..SwapData::mock_with_provider(provider)
                },
            })
        };
        let built_at = Instant::now();
        let unquoted = ConfirmSwapQuote::initial(&swap(primitives::SwapProvider::UniswapV3, None), built_at).unwrap();
        let quoted = ConfirmSwapQuote::requoted(Quote::mock_with_provider(primitives::SwapProvider::UniswapV3, "1"), built_at);
        let on_chain = swap(primitives::SwapProvider::UniswapV3, None);
        let cross_chain = swap(primitives::SwapProvider::NearIntents, None);
        let permit_pending = swap(primitives::SwapProvider::UniswapV3, Some(Permit2ApprovalData::mock()));

        assert!(!unquoted.needs_requote(&on_chain, built_at + Duration::from_secs(59)), "a fee change right after opening asks nothing");
        assert!(unquoted.needs_requote(&on_chain, built_at + Duration::from_secs(60)), "an on-chain quote lives one minute");
        assert!(!quoted.needs_requote(&cross_chain, built_at + Duration::from_secs(299)), "a cross-chain quote keeps its deposit address for five minutes");
        assert!(quoted.needs_requote(&cross_chain, built_at + Duration::from_secs(300)));
        assert!(unquoted.needs_requote(&permit_pending, built_at), "a pending permit needs a quote to rebuild with, once");
        assert!(!quoted.needs_requote(&permit_pending, built_at));
        assert!(
            ConfirmSwapQuote::initial(&GemTransferData::mock(TransactionInputType::Transfer { asset: Asset::mock_eth() }), built_at).is_none(),
            "only a swap holds a quote"
        );
    }

    #[test]
    fn test_a_swap_load_asks_its_provider_again_for_the_transfer_it_holds() {
        block_on(async {
            let swapper = MockSwapper::new(SwapperProvider::UniswapV3, |request| Ok(Quote::mock_with_request(request))).with_pending_permit(Permit2ApprovalData::mock());
            let builds = swapper.builds();
            let wallet = Wallet::mock_with_chains(&[Chain::Ethereum]);
            let testkit = ConfirmTestkit::with_swap(wallet.clone(), Arc::new(GemSwapService::mock_with_swappers(vec![Box::new(swapper)])));
            let transfer = mock_swap_transfer(&wallet.accounts[0].address, None);

            let built_at = Instant::now();
            let unquoted = ConfirmSwapQuote::initial(&transfer, built_at).unwrap();

            let (refreshed, swap) = testkit.service.refreshed_swap(&wallet, transfer.clone(), Some(unquoted.clone()), built_at + Duration::from_secs(60)).await.unwrap();

            let swap = swap.unwrap();
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

            let (kept, swap) = testkit.service.refreshed_swap(&wallet, transfer.clone(), Some(unquoted), built_at).await.unwrap();
            let swap = swap.unwrap();
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

            let now = Instant::now();
            let (fitted, swap) = testkit.service.fitted_max_swap(&wallet, reserved, None, now).await.unwrap();

            let swap = swap.unwrap();
            assert_eq!(swap.quote.unwrap().request.value, 990u32.into(), "the provider is asked again for everything but the fee");
            assert_eq!(swap.quoted_at, now, "the fitted quote is the one the screen holds from now on");
            assert_eq!(fitted.confirm_data.input.transfer.value, 990.into());
            assert_eq!(sent(&fitted), Some((990.into(), 10.into())), "what leaves the wallet plus the fee is the whole balance");
            assert_eq!(builds.lock().unwrap().clone(), vec![FetchQuoteData::None]);

            let deposit = max_swap(SwapData::mock_transfer(SwapProvider::NearIntents, "1000", "1", "0xdeposit"), 1_000);
            let held = ConfirmSwapQuote::initial(&deposit.confirm_data.input.transfer, now).unwrap();
            let (kept, swap) = testkit.service.fitted_max_swap(&wallet, deposit, Some(held), now).await.unwrap();

            assert!(swap.unwrap().quote.is_none(), "a deposit is not asked again; the fee comes off at signing, and the held quote stays as it was");
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
            let wallet = testkit.keystore_wallet();
            let transfer = mock_swap_transfer(&wallet.accounts[0].address, Some(Permit2ApprovalData::mock()));
            let quote = Quote::mock_with_request(&crate::services::swap::rules::requote_request(&wallet, &transfer, &transfer.value).unwrap().1);

            let error = testkit.service.submit(SendInput::mock_signed_by(wallet, transfer, Some(quote))).await.unwrap_err();

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
            let wallet = testkit.keystore_wallet();
            let transfer = mock_swap_transfer(&wallet.accounts[0].address, None);
            let quote = Quote::mock_with_request(&crate::services::swap::rules::requote_request(&wallet, &transfer, &transfer.value).unwrap().1);

            let error = testkit.service.submit(SendInput::mock_signed_by(wallet, transfer, Some(quote))).await.unwrap_err();

            assert!(builds.lock().unwrap().is_empty(), "the data the screen loaded is what gets signed");
            assert_eq!(testkit.passwords.create_requests.lock().unwrap().clone(), vec![false]);
            assert!(broadcast_failed(&error), "signing succeeded and only the broadcast failed: {error:?}");
        });
    }
}
