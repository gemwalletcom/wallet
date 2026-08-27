pub mod model;
pub mod rules;

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use primitives::{Asset, Wallet};
use swapper::permit2_data::Permit2Data;
use swapper::{AssetList, FetchQuoteData, Quote, SwapperError};

use crate::config::swap_config::get_swap_config;
use crate::gem_swapper::{GemSwapper, permit2_data_to_eip712_json};
use crate::keystore::{GemKeystore, keystore_id_for_wallet};
use crate::message::sign_type::{SignDigestType, SignMessage};
use crate::message::signer::MessageSigner;
use crate::models::swap::{GemSwapQuote, GemSwapQuoteData};
use crate::services::wallet::GemKeystorePassword;
pub use model::GemSwapTransfer;
use primitives::AssetId;

#[derive(uniffi::Object)]
pub struct GemSwapService {
    swapper: Arc<GemSwapper>,
    keystore: Arc<GemKeystore>,
    password: Arc<dyn GemKeystorePassword>,
}

#[uniffi::export]
impl GemSwapService {
    #[uniffi::constructor]
    pub fn new(swapper: Arc<GemSwapper>, keystore: Arc<GemKeystore>, password: Arc<dyn GemKeystorePassword>) -> Self {
        Self { swapper, keystore, password }
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

    pub async fn get_transfer(&self, wallet: Wallet, quote: Quote) -> Result<GemSwapTransfer, SwapperError> {
        let data = self.get_quote_data(&wallet, &quote).await?;
        rules::swap_transfer(&wallet, &quote, data)
    }
}

#[uniffi::export]
pub fn swap_quote(quote: Quote) -> GemSwapQuote {
    rules::swap_quote(&quote)
}

impl GemSwapService {
    async fn get_quote_data(&self, wallet: &Wallet, quote: &Quote) -> Result<GemSwapQuoteData, SwapperError> {
        let data = match self.swapper.get_permit2_for_quote(quote).await? {
            Some(approval) => FetchQuoteData::Permit2(self.permit2_data(wallet, quote, &approval)?),
            None => FetchQuoteData::None,
        };
        self.swapper.get_quote_data(quote, data).await
    }

    fn permit2_data(&self, wallet: &Wallet, quote: &Quote, approval: &swapper::Permit2ApprovalData) -> Result<Permit2Data, SwapperError> {
        let chain = AssetId::new(&quote.request.from_asset.id).ok_or(SwapperError::NotSupportedAsset)?.chain;
        let now = SystemTime::now().duration_since(UNIX_EPOCH).map(|duration| duration.as_secs()).unwrap_or(0);
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
            .map_err(|error| SwapperError::TransactionError(error.to_string()))?;
        let signature = signer
            .sign_with_keystore(self.keystore.clone(), keystore_id_for_wallet(wallet.id.id()), password)
            .map_err(|error| SwapperError::TransactionError(error.to_string()))?;
        let signature = primitives::hex::decode_hex(&signature).map_err(|error| SwapperError::TransactionError(error.to_string()))?;
        Ok(Permit2Data { permit_single, signature })
    }
}
