pub mod config;
pub mod model;
pub mod rules;
pub mod store;

use crate::clock::unix_seconds;
use crate::keystore::decode_password;
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
pub use model::{GemSwapPair, GemSwapPairSuggestion, GemSwapTransfer};
use primitives::{AssetId, WalletId};
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
        Self {
            swapper,
            keystore,
            password,
            store,
        }
    }

    pub fn supported_assets(&self, asset_id: AssetId) -> AssetList {
        self.swapper.supported_chains_for_from_asset(&asset_id)
    }

    pub async fn get_quotes(
        &self,
        wallet: Wallet,
        from_asset: Asset,
        to_asset: Asset,
        value: String,
        use_max_amount: bool,
        slippage_bps: Option<u32>,
    ) -> Result<Vec<Quote>, SwapperError> {
        let request = rules::quote_request(&wallet, &from_asset, &to_asset, value, use_max_amount, slippage_bps)?;
        self.swapper.preload_routes(from_asset.id, to_asset.id).await;
        Ok(rules::sort_quotes(self.swapper.get_quote(&request).await?))
    }

    pub fn pair_for_asset(&self, asset_id: AssetId, has_balance: bool) -> GemSwapPairSuggestion {
        rules::pair_for_asset(asset_id, has_balance)
    }

    pub async fn suggest_pair(&self, wallet_id: WalletId, pay_asset_id: Option<AssetId>) -> Result<Option<GemSwapPairSuggestion>, GemServiceError> {
        let pay_asset_id = match pay_asset_id {
            Some(asset_id) => asset_id,
            None => match self.store.get_pay_asset_ids(wallet_id.clone()).await?.into_iter().next() {
                Some(asset_id) => asset_id,
                None => return Ok(None),
            },
        };
        Ok(Some(GemSwapPairSuggestion {
            receive_asset_id: self.suggest_receive_asset(&wallet_id, &pay_asset_id).await?,
            pay_asset_id,
        }))
    }

    pub async fn get_transfer(&self, wallet: Wallet, quote: Quote) -> Result<GemSwapTransfer, SwapperError> {
        let data = self.get_quote_data(&wallet, &quote).await?;
        rules::swap_transfer(&wallet, &quote, data)
    }
}

impl GemSwapService {
    async fn suggest_receive_asset(&self, wallet_id: &WalletId, pay_asset_id: &AssetId) -> Result<Option<AssetId>, GemServiceError> {
        let pairs = self.store.get_swap_pairs(wallet_id.clone()).await?;
        if let Some(asset_id) = rules::most_swapped_receive_asset(&pairs, pay_asset_id) {
            return Ok(Some(asset_id));
        }
        let recents = self.store.get_recent_asset_ids(wallet_id.clone()).await?;
        if let Some(asset_id) = rules::first_other_asset(recents, pay_asset_id) {
            return Ok(Some(asset_id));
        }
        let supported = self.supported_assets(pay_asset_id.clone());
        let candidates = self.store.get_receive_asset_ids(wallet_id.clone(), supported.chains, supported.asset_ids).await?;
        Ok(rules::first_other_asset(candidates, pay_asset_id))
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
            .get_password(wallet.id.clone(), false)
            .map(|password| decode_password(&password))
            .map_err(|error| SwapperError::TransactionError(error.to_string()))?;
        let signature = signer
            .sign_with_keystore(self.keystore.clone(), keystore_id_for_wallet(wallet.id.id()), password)
            .map_err(|error| SwapperError::TransactionError(error.to_string()))?;
        let signature = primitives::hex::decode_hex(&signature).map_err(|error| SwapperError::TransactionError(error.to_string()))?;
        Ok(Permit2Data { permit_single, signature })
    }
}
