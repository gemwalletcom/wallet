use std::sync::Arc;

use crate::GemstoneError;
use crate::address::validate_address;
use crate::alien::{AlienProvider, AlienProviderWrapper};
use crate::config::chain::is_memo_supported;
use crate::config::wallet_connect::get_wallet_connect_config;
use crate::models::payment::{GemPayment, GemPaymentAmount, GemPaymentInvoice, GemPaymentLink, GemPaymentRequest};
use crate::services::assets::{GemAssetAction, GemAssetFilter, GemAssetsService};
use crate::services::error::GemServiceError;
use crate::services::transfer::model::{GemRecipient, GemTransferData};
use chain_primitives::checksum_address;
use num_bigint::{BigInt, BigUint};
use number_formatter::BigNumberFormatter;
use payment::{PaymentLoad, PaymentService, PaymentTransaction, PaymentURLDecoder, PaymentUpdate, VerificationOutcome, WalletConnectPayAuth, verification_outcome};
use primitives::TransactionInputType;
use primitives::{
    Asset, AssetId, Chain, ChainAddress, ChainType, PaymentInvoice, PaymentLink, PaymentQuote, PaymentStatus, PaymentVerification, TransactionChange, TransactionState, TransactionType, TransactionUpdate, TransferDataExtra,
    TransferDataOutputAction, TransferDataOutputType, Wallet, hex,
};
use uuid::Uuid;

pub type GemPaymentError = payment::PaymentError;

#[uniffi::remote(Enum)]
pub enum GemPaymentError {
    NoPaymentOptions,
    Status { status: PaymentStatus },
    InvalidRequest { reason: String },
    Network { reason: String },
}

impl From<GemServiceError> for GemPaymentError {
    fn from(error: GemServiceError) -> Self {
        match error {
            GemServiceError::InvalidInput { msg } | GemServiceError::NotFound { msg } | GemServiceError::Unsupported { msg } => Self::invalid_request(msg),
            error => Self::Network { reason: error.to_string() },
        }
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, uniffi::Enum)]
pub enum GemPaymentLoad {
    Sign { transfer: GemTransferData },
    Verify { invoice: GemPaymentInvoice, asset_id: AssetId, url: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemPaymentVerificationOutcome {
    Complete,
    Error,
    Ignored,
}

#[uniffi::export]
pub fn payment_verification_outcome(message_type: String) -> GemPaymentVerificationOutcome {
    match verification_outcome(&message_type) {
        VerificationOutcome::Complete => GemPaymentVerificationOutcome::Complete,
        VerificationOutcome::Error => GemPaymentVerificationOutcome::Error,
        VerificationOutcome::Ignored => GemPaymentVerificationOutcome::Ignored,
    }
}

#[derive(uniffi::Object)]
pub struct GemPaymentService {
    payments: PaymentService,
    assets: Arc<GemAssetsService>,
}

#[uniffi::export]
impl GemPaymentService {
    #[uniffi::constructor]
    pub fn new(provider: Arc<dyn AlienProvider>, assets: Arc<GemAssetsService>) -> Self {
        let auth = WalletConnectPayAuth {
            app_id: get_wallet_connect_config().project_id,
            client_id: Uuid::new_v4().to_string(),
        };
        Self {
            payments: PaymentService::new(Arc::new(AlienProviderWrapper::new(provider)), auth),
            assets,
        }
    }

    pub async fn prepare(&self, payment: GemPayment, wallet: Wallet) -> Result<GemPaymentTarget, GemPaymentError> {
        match payment {
            GemPayment::Request { request } => Ok(self.prepare_request(request, wallet).await?),
            GemPayment::Link { link } => self.prepare_link(link, wallet).await,
        }
    }

    pub async fn prepare_asset(&self, payment: GemPaymentRecipient, asset_id: AssetId) -> Result<GemPaymentStep, GemPaymentError> {
        let asset = self.assets.ensure_asset(asset_id).await?;
        Ok(payment_step(&recipient_request(payment), &asset))
    }
}

impl GemPaymentService {
    pub async fn load(&self, link: GemPaymentLink, addresses: Vec<ChainAddress>) -> Result<GemPaymentLoad, GemPaymentError> {
        self.payment_load(self.payments.load(&link, &addresses).await?).await
    }

    pub(crate) fn decode_url(&self, string: String) -> Result<GemPayment, GemstoneError> {
        Ok(PaymentURLDecoder::decode(&string)?)
    }

    async fn prepare_request(&self, request: GemPaymentRequest, wallet: Wallet) -> Result<GemPaymentTarget, GemServiceError> {
        let destination = match self.request_destination(&request, &wallet, GemAssetAction::Send.filters()).await? {
            PaymentDestination::Unsupported => self.request_destination(&request, &wallet, Vec::new()).await?,
            sendable => sendable,
        };
        Ok(match destination {
            PaymentDestination::Transfer { asset, step } => step.target(asset),
            PaymentDestination::SelectAsset { payment, chains } => GemPaymentTarget::SelectAsset { payment, chains },
            PaymentDestination::Unsupported => GemPaymentTarget::Unsupported,
        })
    }

    async fn request_destination(&self, request: &GemPaymentRequest, wallet: &Wallet, filters: Vec<GemAssetFilter>) -> Result<PaymentDestination, GemServiceError> {
        let assets = self.assets.wallet_assets(wallet.id.clone(), filters).await?;
        Ok(payment_destination(request, &assets))
    }

    async fn prepare_link(&self, link: GemPaymentLink, wallet: Wallet) -> Result<GemPaymentTarget, GemPaymentError> {
        let addresses = wallet.accounts.iter().map(|account| ChainAddress::new(account.chain, account.address.clone())).collect();
        Ok(match self.load(link, addresses).await? {
            GemPaymentLoad::Sign { transfer } => GemPaymentTarget::Confirm { transfer },
            GemPaymentLoad::Verify { invoice, url, .. } => GemPaymentTarget::Verify { url, link: invoice.link },
        })
    }

    pub(crate) async fn select_asset(&self, link: &PaymentLink, addresses: Vec<ChainAddress>, asset_id: AssetId) -> Result<GemPaymentLoad, GemPaymentError> {
        self.payment_load(self.payments.select_asset(link, &addresses, asset_id).await?).await
    }

    pub(crate) async fn transaction_update(&self, hash: &str, link: &PaymentLink) -> Result<TransactionUpdate, GemPaymentError> {
        Ok(payment_transaction_update(hash, self.payments.status(link).await?))
    }

    pub(crate) fn record_hash(&self, input_type: &TransactionInputType) -> Option<String> {
        payment_quote(input_type).and_then(|(invoice, _)| payment_record_hash(&invoice.link))
    }

    pub(crate) async fn confirm(&self, input_type: &TransactionInputType, action_results: Vec<String>) -> Result<(), GemPaymentError> {
        let Some((invoice, quote)) = payment_quote(input_type) else {
            return Ok(());
        };
        self.payments.confirm(&invoice.link, &quote.id, action_results).await
    }

    pub(crate) async fn quote_transfer_data(&self, invoice: GemPaymentInvoice, asset_id: AssetId, verification: PaymentVerification) -> Result<GemTransferData, GemPaymentError> {
        let asset = self.assets.ensure_token_asset(asset_id).await?;
        quote_transfer_data(invoice, asset, verification)
    }

    async fn payment_load(&self, load: PaymentLoad) -> Result<GemPaymentLoad, GemPaymentError> {
        match load {
            PaymentLoad::Sign { transaction } => {
                let asset = self.assets.ensure_token_asset(payment_asset_id(&transaction)).await?;
                Ok(GemPaymentLoad::Sign {
                    transfer: transaction_transfer_data(transaction, asset),
                })
            }
            PaymentLoad::Verify { invoice, asset_id, url } => Ok(GemPaymentLoad::Verify { invoice, asset_id, url }),
        }
    }
}

fn quote_transfer_data(invoice: GemPaymentInvoice, asset: Asset, verification: PaymentVerification) -> Result<GemTransferData, GemPaymentError> {
    let value = invoice
        .quotes
        .iter()
        .find(|quote| quote.asset_id == asset.id)
        .map(|quote| quote.value.clone())
        .ok_or(GemPaymentError::invalid_request("Payment has no quote for the asset"))?;
    Ok(GemTransferData {
        input_type: TransactionInputType::Payment {
            asset,
            invoice: GemPaymentInvoice { verification: Some(verification), ..invoice },
            extra: TransferDataExtra {
                transaction_type: TransactionType::Transfer,
                ..Default::default()
            },
        },
        recipient: GemRecipient::address(String::new()),
        value: BigInt::from(value),
        use_max_amount: false,
    })
}

fn transaction_transfer_data(transaction: PaymentTransaction, asset: Asset) -> GemTransferData {
    let transfer = transaction.request.as_ref().filter(|request| is_request_for(request, &asset)).and_then(|request| request_transfer(request, &asset));
    let recipient = match &transfer {
        Some(transfer) => transfer.recipient.clone(),
        None => GemRecipient {
            address: String::new(),
            name: None,
            memo: transaction.memo.clone(),
            references: Vec::new(),
        },
    };
    GemTransferData {
        input_type: TransactionInputType::Payment {
            asset,
            invoice: transaction.invoice,
            extra: TransferDataExtra {
                to: recipient.address.clone(),
                gas_limit: None,
                gas_price: None,
                data: Some(transaction_data(&transaction.transaction)),
                output_action: match transaction.output_type {
                    TransferDataOutputType::EncodedTransaction => TransferDataOutputAction::Send,
                    TransferDataOutputType::Signature => TransferDataOutputAction::Sign,
                },
                output_type: transaction.output_type,
                transaction_type: transaction.transaction_type,
                approval: transaction.approval,
            },
        },
        recipient,
        value: transfer.map(|transfer| transfer.value).unwrap_or_default(),
        use_max_amount: false,
    }
}

pub(crate) fn payment_record_hash(link: &PaymentLink) -> Option<String> {
    match link {
        PaymentLink::WalletConnectPay { payment_id } => Some(payment_id.clone()),
        PaymentLink::SolanaPay { .. } => None,
    }
}

fn payment_transaction_update(hash: &str, update: PaymentUpdate) -> TransactionUpdate {
    match update.status {
        PaymentStatus::Succeeded => TransactionUpdate::new(TransactionState::Confirmed, update.transaction_id.map(|new| TransactionChange::HashChange { old: hash.to_string(), new }).into_iter().collect()),
        PaymentStatus::Failed | PaymentStatus::Expired | PaymentStatus::Cancelled => TransactionUpdate::new_state(TransactionState::Failed),
        PaymentStatus::RequiresAction | PaymentStatus::Processing => TransactionUpdate::new_state(TransactionState::Pending),
    }
}

fn payment_quote(input_type: &TransactionInputType) -> Option<(&PaymentInvoice, &PaymentQuote)> {
    let TransactionInputType::Payment { asset, invoice, .. } = input_type else {
        return None;
    };
    let quote = invoice.quotes.iter().find(|quote| quote.asset_id == asset.id)?;
    Some((invoice, quote))
}

fn transaction_data(transaction: &str) -> Vec<u8> {
    match transaction.starts_with("0x") {
        true => hex::decode_hex(transaction).unwrap_or_else(|_| transaction.as_bytes().to_vec()),
        false => transaction.as_bytes().to_vec(),
    }
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemPaymentRecipient {
    pub recipient: GemRecipient,
    #[uniffi(default = None)]
    pub amount: Option<String>,
}

#[derive(Debug, Clone, uniffi::Enum)]
#[allow(clippy::large_enum_variant)]
pub enum GemPaymentTarget {
    Confirm { transfer: GemTransferData },
    Verify { url: String, link: PaymentLink },
    Amount { asset: Asset, payment: GemPaymentRecipient },
    Recipient { asset: Asset, payment: GemPaymentRecipient },
    SelectAsset { payment: GemPaymentRecipient, chains: Vec<Chain> },
    Unsupported,
}

#[derive(Debug, Clone, uniffi::Enum)]
#[allow(clippy::large_enum_variant)]
pub enum GemPaymentStep {
    Confirm { transfer: GemTransferData },
    Amount { payment: GemPaymentRecipient },
    Recipient { payment: GemPaymentRecipient },
}

impl GemPaymentStep {
    fn target(self, asset: Asset) -> GemPaymentTarget {
        match self {
            Self::Confirm { transfer } => GemPaymentTarget::Confirm { transfer },
            Self::Amount { payment } => GemPaymentTarget::Amount { asset, payment },
            Self::Recipient { payment } => GemPaymentTarget::Recipient { asset, payment },
        }
    }
}

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
enum PaymentDestination {
    Transfer { asset: Asset, step: GemPaymentStep },
    SelectAsset { payment: GemPaymentRecipient, chains: Vec<Chain> },
    Unsupported,
}

fn payment_asset_id(transaction: &PaymentTransaction) -> AssetId {
    transaction.request.as_ref().and_then(|request| request.asset_id.clone()).unwrap_or_else(|| AssetId::from_chain(transaction.account.chain))
}

fn payment_destination(request: &GemPaymentRequest, assets: &[Asset]) -> PaymentDestination {
    let payable: Vec<&Asset> = match &request.asset_id {
        Some(asset_id) => assets.iter().filter(|asset| &asset.id == asset_id).collect(),
        None => assets.iter().filter(|asset| validate_address(&request.address, asset.id.chain)).collect(),
    };
    match payable.as_slice() {
        [] => PaymentDestination::Unsupported,
        [asset] => PaymentDestination::Transfer {
            step: payment_step(request, asset),
            asset: (*asset).clone(),
        },
        payable => {
            let mut chains: Vec<Chain> = Vec::new();
            for asset in payable {
                if !chains.contains(&asset.id.chain) {
                    chains.push(asset.id.chain);
                }
            }
            PaymentDestination::SelectAsset {
                payment: GemPaymentRecipient {
                    recipient: payment_recipient(request, None),
                    amount: exact_amount(request),
                },
                chains,
            }
        }
    }
}

pub(crate) fn asset_step(request: &GemPaymentRequest, asset: &Asset) -> Option<GemPaymentStep> {
    is_request_for(request, asset).then(|| payment_step(request, asset))
}

fn payment_step(request: &GemPaymentRequest, asset: &Asset) -> GemPaymentStep {
    let chain = asset.id.chain;
    let payment = || GemPaymentRecipient {
        recipient: payment_recipient(request, Some(chain)),
        amount: exact_amount(request),
    };
    if requires_memo(chain, request) || !validate_address(&checksum_address(&request.address, chain), chain) {
        return GemPaymentStep::Recipient { payment: payment() };
    }
    match request_transfer(request, asset) {
        Some(transfer) => GemPaymentStep::Confirm { transfer },
        None => GemPaymentStep::Amount { payment: payment() },
    }
}

fn is_request_for(request: &GemPaymentRequest, asset: &Asset) -> bool {
    request.asset_id.as_ref().is_none_or(|asset_id| asset_id == &asset.id)
}

fn recipient_request(payment: GemPaymentRecipient) -> GemPaymentRequest {
    GemPaymentRequest {
        address: payment.recipient.address,
        amount: payment.amount.map(|value| GemPaymentAmount::ExactValue { value }),
        memo: payment.recipient.memo,
        label: None,
        references: Some(payment.recipient.references),
        asset_id: None,
    }
}

fn payment_recipient(request: &GemPaymentRequest, chain: Option<Chain>) -> GemRecipient {
    GemRecipient {
        address: chain.map_or_else(|| request.address.clone(), |chain| checksum_address(&request.address, chain)),
        name: None,
        memo: request.memo.clone(),
        references: request.references.clone().unwrap_or_default(),
    }
}

fn exact_amount(request: &GemPaymentRequest) -> Option<String> {
    match request.amount.as_ref()? {
        GemPaymentAmount::ExactValue { value: amount } => Some(amount.clone()),
        GemPaymentAmount::AtomicValue { value: _ } => None,
    }
}

fn request_transfer(request: &GemPaymentRequest, asset: &Asset) -> Option<GemTransferData> {
    let chain = asset.id.chain;
    if !validate_address(&checksum_address(&request.address, chain), chain) {
        return None;
    }
    let value = transfer_value(request, asset.decimals)?;
    Some(GemTransferData {
        input_type: TransactionInputType::Transfer { asset: asset.clone() },
        recipient: payment_recipient(request, Some(chain)),
        value: BigInt::from(value),
        use_max_amount: false,
    })
}

fn requires_memo(chain: Chain, request: &GemPaymentRequest) -> bool {
    payment_memo_required(chain) && request.memo.as_deref().unwrap_or_default().is_empty()
}

fn payment_memo_required(chain: Chain) -> bool {
    is_memo_supported(chain) && chain.chain_type() != ChainType::Solana
}

fn transfer_value(request: &GemPaymentRequest, decimals: i32) -> Option<BigUint> {
    match request.amount.as_ref()? {
        GemPaymentAmount::ExactValue { value: amount } => BigNumberFormatter::value_from_amount_exact(amount, u32::try_from(decimals).ok()?).ok(),
        GemPaymentAmount::AtomicValue { value } => Some(value.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::payment::{GemPaymentAmount, GemPaymentLink, GemPaymentRequest};
    use crate::services::assets::testkit::MemoryAssetStore;
    use crate::testkit::TestAlienProvider;
    use crate::testkit::mock_payment_transaction;
    use futures::executor::block_on;
    use primitives::{Asset, AssetBasic, AssetId, AssetProperties, AssetScore, AssetType, Chain, PaymentInvoice};
    use std::sync::Arc;

    fn service_with(assets: &[Asset], sendable: &[&Asset]) -> (GemPaymentService, Arc<MemoryAssetStore>) {
        let store = Arc::new(MemoryAssetStore::default());
        *store.assets.lock().unwrap() = assets.iter().map(|asset| AssetBasic::new(asset.clone(), AssetProperties::default(asset.id.clone()), AssetScore::default())).collect();
        *store.filtered_asset_ids.lock().unwrap() = sendable.iter().map(|asset| asset.id.clone()).collect();
        let service = GemPaymentService::new(Arc::new(TestAlienProvider::with_status(200)), Arc::new(GemAssetsService::mock(Arc::new(TestAlienProvider::with_status(200)), store.clone())));
        (service, store)
    }

    #[test]
    fn test_a_scan_opens_selection_only_when_the_send_list_shows_more_than_one_match() {
        let (bitcoin, near) = (Asset::from_chain(Chain::Bitcoin), Asset::from_chain(Chain::Near));
        let scan = |sendable: &[&Asset]| {
            let (service, store) = service_with(&[bitcoin.clone(), near.clone()], sendable);
            let request = GemPaymentRequest {
                address: BITCOIN_ADDRESS.to_string(),
                ..GemPaymentRequest::mock()
            };
            let target = block_on(service.prepare(GemPayment::Request { request }, Wallet::mock())).unwrap();
            (target, store.wallet_asset_filters.lock().unwrap().clone())
        };

        let (target, filters) = scan(&[&bitcoin]);
        let GemPaymentTarget::Amount { asset, .. } = target else { panic!("{target:?}") };
        assert_eq!(asset.id, bitcoin.id);
        assert_eq!(filters, [GemAssetAction::Send.filters()]);

        let GemPaymentTarget::SelectAsset { chains, .. } = scan(&[&bitcoin, &near]).0 else { panic!() };
        assert_eq!(chains, [Chain::Bitcoin, Chain::Near]);
    }

    #[test]
    fn test_a_request_for_a_token_the_wallet_holds_confirms_whether_or_not_it_is_shown() {
        let token = Asset::new(
            AssetId::from(Chain::Ethereum, Some("0xdac17f958d2ee523a2206206994597c13d831ec7".to_string())),
            "Tether".to_string(),
            "USDT".to_string(),
            6,
            primitives::AssetType::ERC20,
        );
        let request = GemPaymentRequest {
            address: "0x1f9090aaE28b8a3dCeaDf281B0F12828e676c326".to_string(),
            amount: Some(GemPaymentAmount::AtomicValue { value: 1_000_000u32.into() }),
            asset_id: Some(token.id.clone()),
            ..GemPaymentRequest::mock()
        };
        let target = block_on(service_with(std::slice::from_ref(&token), &[]).0.prepare(GemPayment::Request { request }, Wallet::mock())).unwrap();
        let GemPaymentTarget::Confirm { transfer } = target else {
            panic!("a token the wallet holds is payable whether or not the screen shows it")
        };
        assert_eq!(transfer.input_type.get_asset().id, token.id);
        assert_eq!(transfer.value, 1_000_000u32.into());
    }

    #[test]
    fn test_a_request_for_a_token_the_wallet_does_not_hold_is_not_payable() {
        let request = GemPaymentRequest {
            address: "0x1f9090aaE28b8a3dCeaDf281B0F12828e676c326".to_string(),
            asset_id: Some(AssetId::from(Chain::Ethereum, Some("0xmissing".to_string()))),
            ..GemPaymentRequest::mock()
        };
        let GemPaymentTarget::Unsupported = block_on(service_with(&[], &[]).0.prepare(GemPayment::Request { request }, Wallet::mock())).unwrap() else {
            panic!("a token the wallet does not hold is not payable")
        };
    }

    #[test]
    fn test_payment_transaction_update() {
        let update = |status, transaction_id: Option<&str>| {
            payment_transaction_update(
                "pay_1",
                PaymentUpdate {
                    status,
                    transaction_id: transaction_id.map(str::to_string),
                },
            )
        };

        assert_eq!(
            update(PaymentStatus::Succeeded, Some("0xrelayed")),
            TransactionUpdate::new(
                TransactionState::Confirmed,
                vec![TransactionChange::HashChange {
                    old: "pay_1".to_string(),
                    new: "0xrelayed".to_string()
                }]
            )
        );
        assert_eq!(update(PaymentStatus::Succeeded, None), TransactionUpdate::new_state(TransactionState::Confirmed));
        assert_eq!(update(PaymentStatus::Processing, None), TransactionUpdate::new_state(TransactionState::Pending));
        assert_eq!(update(PaymentStatus::RequiresAction, None), TransactionUpdate::new_state(TransactionState::Pending));
        assert_eq!(update(PaymentStatus::Failed, None), TransactionUpdate::new_state(TransactionState::Failed));
        assert_eq!(update(PaymentStatus::Expired, None), TransactionUpdate::new_state(TransactionState::Failed));
        assert_eq!(update(PaymentStatus::Cancelled, None), TransactionUpdate::new_state(TransactionState::Failed));
    }

    #[test]
    fn test_confirm() {
        let solana_pay = TransactionInputType::Payment {
            asset: Asset::mock_sol(),
            invoice: PaymentInvoice {
                link: PaymentLink::SolanaPay {
                    url: "https://merchant.example/pay".to_string(),
                },
                quotes: vec![],
                ..PaymentInvoice::mock()
            },
            extra: TransferDataExtra::mock(),
        };

        assert_eq!(block_on(GemPaymentService::mock().confirm(&solana_pay, vec!["0xhash".to_string()])), Ok(()), "a rail without quotes has nothing to confirm");
    }

    #[test]
    fn test_payment_record_hash() {
        assert_eq!(payment_record_hash(&PaymentLink::WalletConnectPay { payment_id: "pay_1".to_string() }).as_deref(), Some("pay_1"));
        assert_eq!(payment_record_hash(&PaymentLink::SolanaPay { url: "solana:pay".to_string() }), None, "only a relayed payment is recorded by its id");
    }

    const BITCOIN_ADDRESS: &str = "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4";
    const ETHEREUM_ADDRESS: &str = "0x1f9090aaE28b8a3dCeaDf281B0F12828e676c326";
    const SOLANA_ADDRESS: &str = "HA4hQMs22nCuRN7iLDBsBkboz2SnLM1WkNtzLo6xEDY5";
    const XRP_ADDRESS: &str = "rEb8TK3gBgk5auZkwc6sHnwrGVJH8DuaLh";
    const USDC_MINT: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";

    fn solana_usdc() -> Asset {
        Asset::new(AssetId::from_token(Chain::Solana, USDC_MINT), "USD Coin".to_string(), "USDC".to_string(), 6, AssetType::SPL)
    }

    fn request(address: &str, amount: Option<GemPaymentAmount>, memo: Option<&str>) -> GemPaymentRequest {
        GemPaymentRequest {
            address: address.to_string(),
            amount,
            memo: memo.map(str::to_string),
            ..GemPaymentRequest::mock()
        }
    }

    fn exact(value: &str) -> Option<GemPaymentAmount> {
        Some(GemPaymentAmount::ExactValue { value: value.to_string() })
    }

    fn confirmed(step: GemPaymentStep) -> GemTransferData {
        match step {
            GemPaymentStep::Confirm { transfer } => transfer,
            step => panic!("expected confirm, got {step:?}"),
        }
    }

    fn amount(step: GemPaymentStep) -> GemPaymentRecipient {
        match step {
            GemPaymentStep::Amount { payment } => payment,
            step => panic!("expected amount, got {step:?}"),
        }
    }

    fn recipient(step: GemPaymentStep) -> GemPaymentRecipient {
        match step {
            GemPaymentStep::Recipient { payment } => payment,
            step => panic!("expected recipient, got {step:?}"),
        }
    }

    #[test]
    fn test_payment_destination() {
        let (bitcoin, ethereum, smartchain, usdc) = (Asset::from_chain(Chain::Bitcoin), Asset::from_chain(Chain::Ethereum), Asset::from_chain(Chain::SmartChain), solana_usdc());
        let assets = [bitcoin.clone(), ethereum, smartchain, usdc.clone()];

        let PaymentDestination::Transfer { asset, .. } = payment_destination(&request(BITCOIN_ADDRESS, None, None), &assets) else {
            panic!("an address of one network pays its asset")
        };
        assert_eq!(asset, bitcoin);

        let token = GemPaymentRequest {
            asset_id: Some(usdc.id.clone()),
            ..request(SOLANA_ADDRESS, None, None)
        };
        let PaymentDestination::Transfer { asset, .. } = payment_destination(&token, &assets) else {
            panic!("a token request pays that token")
        };
        assert_eq!(asset, usdc);

        let PaymentDestination::SelectAsset { payment, chains } = payment_destination(&request(&ETHEREUM_ADDRESS.to_lowercase(), exact("2"), None), &assets) else {
            panic!("an address of several networks asks which asset to send")
        };
        assert_eq!(payment.recipient.address, ETHEREUM_ADDRESS.to_lowercase());
        assert_eq!(payment.amount.as_deref(), Some("2"));
        assert_eq!(chains, vec![Chain::Ethereum, Chain::SmartChain]);

        let unknown_token = GemPaymentRequest {
            asset_id: Some(AssetId::from_token(Chain::Solana, "11111111111111111111111111111111")),
            ..request(SOLANA_ADDRESS, None, None)
        };
        let PaymentDestination::Unsupported = payment_destination(&unknown_token, &assets) else {
            panic!("a token the wallet does not hold is not payable")
        };
    }

    #[test]
    fn test_payment_step() {
        let (bitcoin, ethereum, xrp) = (Asset::from_chain(Chain::Bitcoin), Asset::from_chain(Chain::Ethereum), Asset::from_chain(Chain::Xrp));

        let transfer = confirmed(payment_step(&request(BITCOIN_ADDRESS, exact("0.0001"), None), &bitcoin));
        assert_eq!(transfer.value, 10_000.into());
        assert_eq!(transfer.recipient.address, BITCOIN_ADDRESS);
        assert_eq!(transfer.input_type.get_asset(), &bitcoin);

        let transfer = confirmed(payment_step(&request(&ETHEREUM_ADDRESS.to_lowercase(), Some(GemPaymentAmount::AtomicValue { value: 1u32.into() }), None), &ethereum));
        assert_eq!(transfer.recipient.address, ETHEREUM_ADDRESS);
        assert_eq!(transfer.value, 1.into());

        let transfer = confirmed(payment_step(&request(XRP_ADDRESS, exact("10"), Some("12345")), &xrp));
        assert_eq!(transfer.value, 10_000_000.into());
        assert_eq!(transfer.recipient.memo.as_deref(), Some("12345"));

        let transfer = confirmed(payment_step(&request(SOLANA_ADDRESS, exact("1"), None), &solana_usdc()));
        assert_eq!(transfer.value, 1_000_000.into(), "Solana confirms without a memo");

        let payment = amount(payment_step(&request(&ETHEREUM_ADDRESS.to_lowercase(), None, None), &ethereum));
        assert_eq!(payment.recipient.address, ETHEREUM_ADDRESS);
        assert_eq!(payment.amount, None);

        let payment = amount(payment_step(&request(BITCOIN_ADDRESS, exact("0.000000001"), None), &bitcoin));
        assert_eq!(payment.amount.as_deref(), Some("0.000000001"), "excess precision is never rounded");

        let payment = recipient(payment_step(&request(XRP_ADDRESS, exact("10"), None), &xrp));
        assert_eq!(payment.recipient.memo, None);
        assert_eq!(payment.amount.as_deref(), Some("10"));

        let payment = recipient(payment_step(&request("0x123", None, Some("order 7")), &ethereum));
        assert_eq!(payment.recipient.address, "0x123");
        assert_eq!(payment.recipient.memo.as_deref(), Some("order 7"));
    }

    #[test]
    fn test_asset_step() {
        let ethereum = Asset::from_chain(Chain::Ethereum);
        let for_bitcoin = GemPaymentRequest {
            asset_id: Some(AssetId::from_chain(Chain::Bitcoin)),
            ..request(ETHEREUM_ADDRESS, exact("1.5"), None)
        };
        assert!(asset_step(&for_bitcoin, &ethereum).is_none());

        let transfer = confirmed(asset_step(&request(ETHEREUM_ADDRESS, exact("1.5"), None), &ethereum).unwrap());
        assert_eq!(transfer.value, 1_500_000_000_000_000_000u64.into());
    }

    #[test]
    fn test_prepare_asset_takes_the_selected_payment_to_its_step() {
        let ethereum = Asset::from_chain(Chain::Ethereum);
        let xrp = Asset::from_chain(Chain::Xrp);
        let service = service_with(&[ethereum.clone(), xrp.clone()], &[]).0;
        let prepare = |address: &str, amount: Option<&str>, memo: Option<&str>, asset: &Asset| {
            let payment = GemPaymentRecipient {
                recipient: GemRecipient {
                    memo: memo.map(str::to_string),
                    ..GemRecipient::address(address.to_string())
                },
                amount: amount.map(str::to_string),
            };
            block_on(service.prepare_asset(payment, asset.id.clone())).unwrap()
        };

        let transfer = confirmed(prepare(&ETHEREUM_ADDRESS.to_lowercase(), Some("2"), None, &ethereum));
        assert_eq!(transfer.recipient.address, ETHEREUM_ADDRESS);
        assert_eq!(transfer.value, 2_000_000_000_000_000_000u64.into());

        assert_eq!(amount(prepare(&ETHEREUM_ADDRESS.to_lowercase(), None, None, &ethereum)).recipient.address, ETHEREUM_ADDRESS);
        assert_eq!(confirmed(prepare(XRP_ADDRESS, Some("10"), Some("12345"), &xrp)).recipient.memo.as_deref(), Some("12345"));
        recipient(prepare(XRP_ADDRESS, Some("10"), None, &xrp));
    }

    #[test]
    fn test_payment_asset_id_prefers_the_request_asset_over_the_chain_asset() {
        let usdc = AssetId::from_token(Chain::Solana, USDC_MINT);
        let request = GemPaymentRequest {
            address: SOLANA_ADDRESS.to_string(),
            ..GemPaymentRequest::mock()
        };

        assert_eq!(
            payment_asset_id(&PaymentTransaction {
                request: Some(GemPaymentRequest {
                    asset_id: Some(usdc.clone()),
                    ..request.clone()
                }),
                ..mock_payment_transaction()
            }),
            usdc
        );
        assert_eq!(
            payment_asset_id(&PaymentTransaction {
                request: Some(request),
                ..mock_payment_transaction()
            }),
            AssetId::from_chain(Chain::Solana)
        );
        assert_eq!(payment_asset_id(&mock_payment_transaction()), AssetId::from_chain(Chain::Solana));
    }

    #[test]
    fn test_transaction_transfer_data() {
        let solana_usdc = AssetId::from_token(Chain::Solana, USDC_MINT);
        let asset = Asset::new(solana_usdc.clone(), "USD Coin".to_string(), "USDC".to_string(), 6, AssetType::SPL);
        let transaction = PaymentTransaction {
            memo: Some("order 7".to_string()),
            ..mock_payment_transaction()
        };

        let decoded = transaction_transfer_data(
            PaymentTransaction {
                request: Some(GemPaymentRequest {
                    address: SOLANA_ADDRESS.to_string(),
                    amount: Some(GemPaymentAmount::AtomicValue { value: 19_000_000u32.into() }),
                    asset_id: Some(solana_usdc),
                    ..GemPaymentRequest::mock()
                }),
                ..transaction.clone()
            },
            asset.clone(),
        );
        assert_eq!(decoded.recipient.address, SOLANA_ADDRESS);
        assert_eq!(decoded.value, 19_000_000.into());
        match &decoded.input_type {
            TransactionInputType::Payment { invoice, extra, .. } => {
                assert_eq!(invoice, &PaymentInvoice::mock());
                assert_eq!(extra.to, SOLANA_ADDRESS);
                assert_eq!(extra.data.as_deref(), Some(b"encoded".as_slice()));
                assert_eq!(extra.output_type, TransferDataOutputType::EncodedTransaction);
            }
            input_type => panic!("expected a payment input type, got {input_type:?}"),
        }

        let signature = PaymentTransaction {
            output_type: TransferDataOutputType::Signature,
            ..transaction.clone()
        };
        match &transaction_transfer_data(signature, asset.clone()).input_type {
            TransactionInputType::Payment { extra, .. } => assert_eq!(extra.output_action, TransferDataOutputAction::Sign),
            input_type => panic!("expected a payment input type, got {input_type:?}"),
        }

        let hex_encoded = PaymentTransaction {
            transaction: "0x0a0b".to_string(),
            ..transaction.clone()
        };
        match &transaction_transfer_data(hex_encoded, asset.clone()).input_type {
            TransactionInputType::Payment { extra, .. } => assert_eq!(extra.data.as_deref(), Some([0x0a, 0x0b].as_slice())),
            input_type => panic!("expected a payment input type, got {input_type:?}"),
        }

        let for_another_asset = transaction_transfer_data(
            PaymentTransaction {
                request: Some(GemPaymentRequest {
                    address: SOLANA_ADDRESS.to_string(),
                    amount: Some(GemPaymentAmount::AtomicValue { value: 19_000_000u32.into() }),
                    asset_id: Some(AssetId::from_chain(Chain::Solana)),
                    ..GemPaymentRequest::mock()
                }),
                ..transaction.clone()
            },
            asset.clone(),
        );
        assert_eq!(for_another_asset.recipient.address, "", "a request for another asset is not decoded");

        let undecodable = transaction_transfer_data(transaction, asset);
        assert_eq!(undecodable.recipient.address, "");
        assert_eq!(undecodable.recipient.memo.as_deref(), Some("order 7"));
        assert_eq!(undecodable.value, 0.into());
    }

    #[test]
    fn test_request() {
        let decode_url = |url: &str| PaymentURLDecoder::decode(url);
        assert_eq!(
            decode_url("solana:3u3ta6yXYgpheLGc2GVF3QkLHAUwBrvX71Eg8XXjJHGw?amount=0.42301").unwrap(),
            GemPayment::Request {
                request: GemPaymentRequest {
                    address: "3u3ta6yXYgpheLGc2GVF3QkLHAUwBrvX71Eg8XXjJHGw".to_string(),
                    amount: Some(GemPaymentAmount::ExactValue { value: "0.42301".to_string() }),
                    memo: None,
                    label: None,
                    asset_id: Some(AssetId::from_chain(Chain::Solana)),
                    references: None,
                }
            }
        );
    }

    #[test]
    fn test_link() {
        let decode_url = |url: &str| PaymentURLDecoder::decode(url);
        const CONSTANT_K: &str = "https://www.constant-k.com/ck-txreq/?tok=MjYyfG9wZXJhdG9yfGFubnVhbHx8MTc4NzUyOTMxOXw3M2FiNDFhZmIwNTAxZWNjNjE2Y2E4NmIxZGE5N2FlOWZjM2Y1OGMzZWZhMGYxMjNiOGI4ZGYzZmU2YzQ3ZmM4";

        assert_eq!(
            decode_url("solana:https%3A%2F%2Fapi.spherepay.co%2Fv1%2Fpublic%2FpaymentLink%2Fpay%2FpaymentLink_1").unwrap(),
            GemPayment::Link {
                link: GemPaymentLink::SolanaPay {
                    url: "https://api.spherepay.co/v1/public/paymentLink/pay/paymentLink_1".to_string(),
                }
            }
        );
        assert_eq!(
            decode_url("solana:https%3A%2F%2Fwww.constant-k.com%2Fck-txreq%2F%3Ftok%3DMjYyfG9wZXJhdG9yfGFubnVhbHx8MTc4NzUyOTMxOXw3M2FiNDFhZmIwNTAxZWNjNjE2Y2E4NmIxZGE5N2FlOWZjM2Y1OGMzZWZhMGYxMjNiOGI4ZGYzZmU2YzQ3ZmM4").unwrap(),
            GemPayment::Link {
                link: GemPaymentLink::SolanaPay { url: CONSTANT_K.to_string() }
            }
        );
        assert_eq!(
            decode_url("https://pay.walletconnect.com/?pid=pay_123").unwrap(),
            GemPayment::Link {
                link: GemPaymentLink::WalletConnectPay { payment_id: "pay_123".to_string() },
            }
        );
    }
}
