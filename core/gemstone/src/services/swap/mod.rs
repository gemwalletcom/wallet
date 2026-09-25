pub mod model;
pub mod quote;
pub mod rules;
pub mod session;
pub mod slippage;
pub mod store;
#[cfg(test)]
pub(crate) mod testkit;

use crate::keystore::decode_password;
use crate::models::custom_types::GemBigUint;
use primitives::unix_seconds;
use std::sync::Arc;

use primitives::{Asset, Wallet};
use swapper::permit2_data::Permit2Data;
use swapper::{AssetList, FetchQuoteData, Quote, SwapperError};

use crate::config::swap_config::get_swap_config;
use crate::gem_swapper::{GemSwapper, permit2_data_to_eip712_json};
use crate::keystore::{GemKeystore, keystore_id_for_wallet};
use crate::message::sign_type::{SignDigestType, SignMessage};
use crate::message::signer::MessageSigner;
use crate::models::swap::GemSwapQuoteData;
use crate::services::error::GemServiceError;
use crate::services::wallet::GemKeystorePassword;
pub use model::{GemAssetRate, GemSwapButtonAction, GemSwapButtonInput, GemSwapPair, GemSwapPairSuggestion, GemSwapQuoteSummary, GemSwapTransfer, swap_quote_summary, swapper_quote_summary};
use primitives::AssetId;
pub use session::{GemSwapQuotePhase, GemSwapQuotesResult, GemSwapRequest, GemSwapSession, GemSwapSessionAction, GemSwapTransferPhase};
pub use store::GemSwapStore;

#[derive(uniffi::Object)]
pub struct GemSwapService {
    swapper: Arc<GemSwapper>,
    keystore: Arc<GemKeystore>,
    password: Arc<dyn GemKeystorePassword>,
    store: Arc<dyn GemSwapStore>,
}

#[uniffi::export]
impl GemSwapService {
    #[uniffi::constructor]
    pub fn new(swapper: Arc<GemSwapper>, keystore: Arc<GemKeystore>, password: Arc<dyn GemKeystorePassword>, store: Arc<dyn GemSwapStore>) -> Self {
        Self { swapper, keystore, password, store }
    }
}

impl GemSwapService {
    pub async fn get_quotes(&self, wallet: Wallet, from_asset: Asset, to_asset: Asset, value: GemBigUint, use_max_amount: bool, slippage_bps: Option<u32>) -> Result<Vec<Quote>, SwapperError> {
        let request = rules::quote_request(&wallet, &from_asset, &to_asset, value, use_max_amount, slippage_bps)?;
        self.swapper.preload_routes(from_asset.id, to_asset.id).await;
        self.swapper.get_quote(&request).await
    }

    pub async fn suggest_pair(&self, wallet: Wallet, pay_asset_id: Option<AssetId>) -> Result<Option<GemSwapPairSuggestion>, GemServiceError> {
        let pay_asset_id = match pay_asset_id {
            Some(asset_id) => asset_id,
            None => match self.store.get_pay_asset_ids(wallet.id.clone(), rules::CANDIDATES_LIMIT).await?.into_iter().next() {
                Some(asset_id) => asset_id,
                None => return Ok(None),
            },
        };
        Ok(Some(GemSwapPairSuggestion {
            receive_asset_id: self.suggest_receive_asset(&wallet, &pay_asset_id).await?,
            pay_asset_id,
        }))
    }

    pub async fn get_transfer(&self, wallet: Wallet, quote: Quote) -> Result<GemSwapTransfer, SwapperError> {
        rules::validate_quote_wallet(&wallet, &quote)?;
        let data = self.get_quote_data(&wallet, &quote).await?;
        rules::swap_transfer(&wallet, &quote, data)
    }

    pub fn pair_for_asset(&self, asset_id: AssetId, has_balance: bool) -> GemSwapPairSuggestion {
        rules::pair_for_asset(asset_id, has_balance)
    }

    pub fn supported_assets(&self, asset_id: AssetId) -> AssetList {
        self.swapper.supported_chains_for_from_asset(&asset_id)
    }
}

impl GemSwapService {
    async fn suggest_receive_asset(&self, wallet: &Wallet, pay_asset_id: &AssetId) -> Result<Option<AssetId>, GemServiceError> {
        let supported = rules::assets_in_wallet(self.supported_assets(pay_asset_id.clone()), wallet);
        let pairs = self.store.get_swap_pairs(wallet.id.clone()).await?;
        if let Some(asset_id) = rules::most_swapped_receive_asset(&pairs, pay_asset_id, &supported) {
            return Ok(Some(asset_id));
        }
        let recents = self.store.get_recent_asset_ids(wallet.id.clone(), rules::RECENTS_LIMIT).await?;
        if let Some(asset_id) = rules::first_supported_receive_asset(recents, pay_asset_id, &supported) {
            return Ok(Some(asset_id));
        }
        let candidates = self.store.get_receive_asset_ids(wallet.id.clone(), supported.chains.clone(), supported.asset_ids.clone(), rules::CANDIDATES_LIMIT).await?;
        Ok(rules::first_supported_receive_asset(candidates, pay_asset_id, &supported))
    }

    async fn get_quote_data(&self, wallet: &Wallet, quote: &Quote) -> Result<GemSwapQuoteData, SwapperError> {
        let data = match self.swapper.get_permit2_for_quote(quote).await? {
            Some(approval) => FetchQuoteData::Permit2(self.permit2_data(wallet, quote, &approval)?),
            None => FetchQuoteData::None,
        };
        self.swapper.get_quote_data(quote, data).await
    }

    fn permit2_data(&self, wallet: &Wallet, quote: &Quote, approval: &swapper::Permit2ApprovalData) -> Result<Permit2Data, SwapperError> {
        let chain = AssetId::new(&quote.request.from_asset.id).ok_or(SwapperError::NotSupportedAsset)?.chain;
        let now = unix_seconds().map_err(|error| SwapperError::TransactionError(error.to_string()))?;
        let permit_single = rules::permit_single(approval, now, &get_swap_config());
        let json = permit2_data_to_eip712_json(chain, permit_single.clone(), &approval.permit2_contract)?;
        let signer = MessageSigner::new(SignMessage {
            chain,
            sign_type: SignDigestType::Eip712,
            data: json.into_bytes(),
        });
        let password = self
            .password
            .get_password(false)
            .map(|password| decode_password(&password))
            .map_err(|error| SwapperError::TransactionError(error.to_string()))?;
        let signature = signer
            .sign_with_keystore(self.keystore.clone(), keystore_id_for_wallet(wallet.id.id()), &quote.request.wallet_address, password)
            .map_err(|error| SwapperError::TransactionError(error.to_string()))?;
        let signature = primitives::hex::decode_hex(&signature).map_err(|error| SwapperError::TransactionError(error.to_string()))?;
        Ok(Permit2Data { permit_single, signature })
    }
}

#[cfg(test)]
mod tests {
    use gem_evm::uniswap::deployment::get_uniswap_permit2_by_chain;
    use primitives::testkit::signer_mock::{TEST_EVM_RECIPIENT, TEST_EVM_SENDER, TEST_PRIVATE_KEY_SOLANA_ADDRESS, TEST_SOLANA_SENDER};
    use primitives::{Account, Chain, SignerError, WalletId, asset_constants::SMARTCHAIN_USDT_TOKEN_ID};
    use swapper::{Permit2ApprovalData, SwapperQuoteAsset};
    use tempfile::TempDir;

    use super::testkit::MemorySwapStore;
    use super::*;
    use crate::GemstoneError;
    use crate::keystore::GemImportType;
    use crate::services::wallet::testkit::TEST_PASSWORD;
    use futures::executor::block_on;

    #[test]
    fn test_transfer_rejects_a_quote_from_another_wallet() {
        block_on(async {
            let service = GemSwapService::mock(Arc::new(MemorySwapStore::default()));
            let wallet = Wallet::mock_with_accounts(vec![Account::mock(Chain::Ethereum, TEST_EVM_RECIPIENT)]);
            let quote = Quote::mock(Chain::Ethereum, None);
            assert_eq!(
                service.get_transfer(wallet, quote).await.unwrap_err(),
                SwapperError::TransactionError("quote sender does not match the selected wallet".to_string())
            );
        });
    }

    #[test]
    fn test_transfer_rejects_a_quote_that_pays_out_to_another_wallet() {
        let wallet = Wallet::mock_with_accounts(vec![Account::mock(Chain::Ethereum, TEST_EVM_SENDER), Account::mock(Chain::Solana, TEST_SOLANA_SENDER)]);
        let mut quote = Quote::mock(Chain::Ethereum, None);
        quote.request.to_asset = SwapperQuoteAsset::from(Chain::Solana.as_asset_id());
        quote.request.wallet_address = TEST_EVM_SENDER.to_lowercase();
        quote.request.destination_address = TEST_SOLANA_SENDER.to_string();
        assert_eq!(rules::validate_quote_wallet(&wallet, &quote), Ok(()));

        quote.request.destination_address = TEST_PRIVATE_KEY_SOLANA_ADDRESS.to_string();
        assert_eq!(
            rules::validate_quote_wallet(&wallet, &quote),
            Err(SwapperError::TransactionError("quote destination does not match the selected wallet".to_string()))
        );
    }

    #[test]
    fn test_permit2_rejects_unrelated_quote_sender() {
        let dir = TempDir::new().unwrap();
        let keystore = GemKeystore::new(dir.path().to_string_lossy().to_string()).unwrap();
        let stored = keystore.create_store(GemImportType::mock_private_key(), decode_password(TEST_PASSWORD)).unwrap();
        let wallet = Wallet {
            id: WalletId::from_id(&stored.wallet_id).unwrap(),
            wallet_type: stored.wallet_type,
            ..Wallet::mock_with_accounts(vec![Account::mock(Chain::Ethereum, &stored.accounts[0].address)])
        };
        let service = GemSwapService {
            keystore,
            ..GemSwapService::mock(Arc::new(MemorySwapStore::default()))
        };
        let mut quote = Quote::mock(Chain::Ethereum, None);
        quote.request.wallet_address = stored.accounts[0].address.to_lowercase();
        let approval = Permit2ApprovalData {
            token: TEST_EVM_RECIPIENT.to_string(),
            spender: TEST_EVM_RECIPIENT.to_string(),
            value: 1u32.into(),
            permit2_contract: get_uniswap_permit2_by_chain(&Chain::Ethereum).unwrap().to_string(),
            permit2_nonce: 0,
        };
        assert_eq!(service.permit2_data(&wallet, &quote, &approval).unwrap().signature.len(), 65);

        quote.request.wallet_address = TEST_EVM_RECIPIENT.to_string();
        assert_eq!(
            service.permit2_data(&wallet, &quote, &approval).unwrap_err(),
            SwapperError::TransactionError(GemstoneError::from(SignerError::invalid_input("signing key does not match the approved account")).to_string())
        );
    }

    #[test]
    fn test_a_wallet_that_has_never_paid_with_anything_gets_no_suggestion() {
        block_on(async {
            let store = Arc::new(MemorySwapStore::default());
            let wallet = Wallet::mock_with_chains(&[Chain::Ethereum, Chain::Solana]);

            assert_eq!(GemSwapService::mock(store).suggest_pair(wallet, None).await.unwrap(), None);
        });
    }

    #[test]
    fn test_the_pay_asset_comes_from_the_store_only_when_the_caller_names_none() {
        block_on(async {
            let store = Arc::new(MemorySwapStore::default());
            *store.pay_asset_ids.lock().unwrap() = vec![Chain::Solana.as_asset_id()];
            let service = GemSwapService::mock(store);
            let wallet = Wallet::mock_with_chains(&[Chain::Ethereum, Chain::Solana]);

            let stored = service.suggest_pair(wallet.clone(), None).await.unwrap().unwrap();
            let named = service.suggest_pair(wallet, Some(Chain::Ethereum.as_asset_id())).await.unwrap().unwrap();

            assert_eq!(stored.pay_asset_id, Chain::Solana.as_asset_id());
            assert_eq!(named.pay_asset_id, Chain::Ethereum.as_asset_id());
        });
    }

    #[test]
    fn test_every_candidate_list_is_read_one_page_at_a_time() {
        block_on(async {
            let store = Arc::new(MemorySwapStore::default());
            *store.pay_asset_ids.lock().unwrap() = vec![Chain::Solana.as_asset_id()];
            let wallet = Wallet::mock_with_chains(&[Chain::Ethereum, Chain::Solana]);

            let _ = GemSwapService::mock(store.clone()).suggest_pair(wallet, None).await.unwrap();

            let limits = store.limits.lock().unwrap().clone();
            assert!(!limits.is_empty(), "a suggestion reads the store");
            assert!(limits.iter().all(|limit| *limit > 0), "a wallet is never read whole: {limits:?}");
            assert!(limits.contains(&rules::CANDIDATES_LIMIT));
            assert!(limits.contains(&rules::RECENTS_LIMIT));
        });
    }

    #[test]
    fn test_a_past_swap_pair_outranks_a_recent_asset() {
        block_on(async {
            let store = Arc::new(MemorySwapStore::default());
            *store.pairs.lock().unwrap() = vec![GemSwapPair::mock(Chain::Ethereum, Chain::Solana)];
            *store.recent_asset_ids.lock().unwrap() = vec![Chain::Bitcoin.as_asset_id()];

            let wallet = Wallet::mock_with_chains(&[Chain::Ethereum, Chain::Solana]);
            let suggestion = GemSwapService::mock(store).suggest_pair(wallet, Some(Chain::Ethereum.as_asset_id())).await.unwrap().unwrap();

            assert_eq!(suggestion.receive_asset_id, Some(Chain::Solana.as_asset_id()));
        });
    }

    #[test]
    fn test_a_recent_asset_is_used_when_no_pair_was_ever_swapped() {
        block_on(async {
            let store = Arc::new(MemorySwapStore::default());
            *store.recent_asset_ids.lock().unwrap() = vec![Chain::Ethereum.as_asset_id(), Chain::Bitcoin.as_asset_id(), Chain::Solana.as_asset_id()];

            let wallet = Wallet::mock_with_chains(&[Chain::Ethereum, Chain::Solana]);
            let suggestion = GemSwapService::mock(store).suggest_pair(wallet, Some(Chain::Ethereum.as_asset_id())).await.unwrap().unwrap();

            assert_eq!(
                suggestion.receive_asset_id,
                Some(Chain::Solana.as_asset_id()),
                "the pay asset is never its own receive asset, and neither is a chain the wallet has no account on"
            );
        });
    }

    #[test]
    fn test_a_single_chain_wallet_is_never_suggested_an_asset_it_cannot_receive() {
        block_on(async {
            let usdt_smartchain = AssetId::from_token(Chain::SmartChain, SMARTCHAIN_USDT_TOKEN_ID);
            let store = Arc::new(MemorySwapStore::default());
            *store.pairs.lock().unwrap() = vec![GemSwapPair {
                from_asset_id: Chain::Bitcoin.as_asset_id(),
                to_asset_id: usdt_smartchain.clone(),
            }];
            *store.recent_asset_ids.lock().unwrap() = vec![usdt_smartchain.clone()];
            *store.receive_asset_ids.lock().unwrap() = vec![usdt_smartchain];

            let wallet = Wallet::mock_with_chains(&[Chain::Bitcoin]);
            let suggestion = GemSwapService::mock(store).suggest_pair(wallet, Some(Chain::Bitcoin.as_asset_id())).await.unwrap().unwrap();

            assert_eq!(suggestion.pay_asset_id, Chain::Bitcoin.as_asset_id());
            assert_eq!(suggestion.receive_asset_id, None);
        });
    }

    #[test]
    fn test_the_last_resort_only_asks_for_chains_the_wallet_has_an_account_on() {
        block_on(async {
            let store = Arc::new(MemorySwapStore::default());
            *store.receive_asset_ids.lock().unwrap() = vec![Chain::Solana.as_asset_id()];
            let store_ref = store.clone();

            let wallet = Wallet::mock_with_chains(&[Chain::Ethereum, Chain::Solana]);
            let suggestion = GemSwapService::mock(store).suggest_pair(wallet, Some(Chain::Ethereum.as_asset_id())).await.unwrap().unwrap();

            assert_eq!(suggestion.receive_asset_id, Some(Chain::Solana.as_asset_id()));
            let (chains, asset_ids) = store_ref.receive_requests.lock().unwrap().first().cloned().unwrap();
            assert!(chains.iter().all(|chain| [Chain::Ethereum, Chain::Solana].contains(chain)), "{chains:?}");
            assert!(asset_ids.iter().all(|asset_id| [Chain::Ethereum, Chain::Solana].contains(&asset_id.chain)), "{asset_ids:?}");
        });
    }
}
