use std::sync::Arc;

use crate::GemstoneError;
use crate::address::{checksum_address, validate_address};
use crate::alien::{AlienProvider, AlienProviderWrapper};
use crate::models::custom_types::GemBigUint;
use crate::models::payment::{GemPayment, GemPaymentAmount, GemPaymentLink, GemPaymentRequest, GemPaymentTransaction};
use crate::models::transaction::{GemTransactionInputType, GemTransferDataExtra};
use crate::services::transfer::model::{GemRecipient, GemTransferData};
use num_bigint::BigUint;
use number_formatter::BigNumberFormatter;
use payment::PaymentService as CorePaymentService;
use primitives::{Asset, AssetId, Chain, ChainAddress, ChainType, PaymentURLDecoder, TransferDataOutputAction, TransferDataOutputType, hex};

pub type GemPaymentError = payment::PaymentError;

#[uniffi::remote(Enum)]
pub enum GemPaymentError {
    NoPaymentOptions,
    InvalidRequest { reason: String },
    Network { reason: String },
}

#[derive(uniffi::Object)]
pub struct GemPaymentService {
    service: CorePaymentService,
}

#[uniffi::export]
impl GemPaymentService {
    #[uniffi::constructor]
    pub fn new(provider: Arc<dyn AlienProvider>) -> Self {
        Self {
            service: CorePaymentService::new(Arc::new(AlienProviderWrapper::new(provider))),
        }
    }

    pub async fn load(&self, link: GemPaymentLink, addresses: Vec<ChainAddress>) -> Result<GemPaymentTransaction, GemPaymentError> {
        self.service.load(&link, &addresses).await
    }

    pub fn decode_url(&self, string: String) -> Result<GemPayment, GemstoneError> {
        Ok(PaymentURLDecoder::decode(&string)?)
    }

    pub fn destination(&self, request: GemPaymentRequest, assets: Vec<GemPaymentWalletAsset>) -> GemPaymentDestination {
        payment_destination(&request, assets)
    }

    pub fn transfer_destination(&self, request: GemPaymentRequest, asset: GemPaymentWalletAsset) -> GemPaymentDestination {
        payment_transfer_destination(&request, asset)
    }

    pub fn transfer_data(&self, transfer: GemPaymentConfirmTransfer, asset: Asset) -> GemTransferData {
        transfer_data(&transfer, asset)
    }

    pub fn transaction_asset_id(&self, transaction: GemPaymentTransaction) -> AssetId {
        payment_asset_id(&transaction)
    }

    pub fn transaction_transfer_data(&self, transaction: GemPaymentTransaction, asset: Asset) -> GemTransferData {
        let wallet_asset = GemPaymentWalletAsset {
            asset_id: asset.id.clone(),
            decimals: asset.decimals,
        };
        let transfer = transaction
            .request
            .as_ref()
            .and_then(|request| payment_decoded_transfer(request, wallet_asset))
            .map(|transfer| transfer_data(&transfer, asset.clone()));
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
            input_type: GemTransactionInputType::Generic {
                asset,
                metadata: transaction.merchant,
                extra: GemTransferDataExtra {
                    to: recipient.address.clone(),
                    gas_limit: None,
                    gas_price: None,
                    data: Some(transaction_data(&transaction.transaction)),
                    output_type: TransferDataOutputType::EncodedTransaction,
                    output_action: TransferDataOutputAction::Send,
                    transaction_type: transaction.transaction_type,
                    approval: None,
                },
            },
            recipient,
            value: transfer.map(|transfer| transfer.value).unwrap_or_default(),
            use_max_amount: false,
            minimum_value: None,
        }
    }
}

fn transaction_data(transaction: &str) -> Vec<u8> {
    match transaction.starts_with("0x") {
        true => hex::decode_hex(transaction).unwrap_or_else(|_| transaction.as_bytes().to_vec()),
        false => transaction.as_bytes().to_vec(),
    }
}

fn transfer_data(transfer: &GemPaymentConfirmTransfer, asset: Asset) -> GemTransferData {
    GemTransferData {
        input_type: GemTransactionInputType::Transfer { asset },
        recipient: GemRecipient {
            address: transfer.address.clone(),
            name: None,
            memo: transfer.memo.clone(),
            references: transfer.references.clone(),
        },
        value: transfer.value.clone().into(),
        use_max_amount: false,
        minimum_value: None,
    }
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemPaymentWalletAsset {
    pub asset_id: AssetId,
    pub decimals: i32,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemPaymentConfirmTransfer {
    pub asset_id: AssetId,
    pub address: String,
    pub value: GemBigUint,
    pub memo: Option<String>,
    pub references: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemPaymentRecipient {
    pub recipient: GemRecipient,
    #[uniffi(default = None)]
    pub amount: Option<String>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemPaymentDestination {
    Confirm { transfer: GemPaymentConfirmTransfer },
    Recipient { asset_id: AssetId, payment: GemPaymentRecipient },
    SelectAsset { payment: GemPaymentRecipient, chains: Vec<Chain> },
    Unsupported,
}

fn payment_asset_id(transaction: &GemPaymentTransaction) -> AssetId {
    transaction
        .request
        .as_ref()
        .and_then(|request| request.asset_id.clone())
        .unwrap_or_else(|| AssetId::from_chain(transaction.account.chain))
}

fn payment_destination(request: &GemPaymentRequest, assets: Vec<GemPaymentWalletAsset>) -> GemPaymentDestination {
    let payable: Vec<&GemPaymentWalletAsset> = match &request.asset_id {
        Some(asset_id) => assets.iter().filter(|asset| &asset.asset_id == asset_id).collect(),
        None => assets.iter().filter(|asset| validate_address(&request.address, asset.asset_id.chain)).collect(),
    };
    match payable.as_slice() {
        [] => GemPaymentDestination::Unsupported,
        [asset] => transfer_destination(asset, request),
        payable => {
            let mut chains: Vec<Chain> = Vec::new();
            for asset in payable {
                if !chains.contains(&asset.asset_id.chain) {
                    chains.push(asset.asset_id.chain);
                }
            }
            GemPaymentDestination::SelectAsset {
                payment: GemPaymentRecipient {
                    recipient: payment_recipient(request, None),
                    amount: exact_amount(request),
                },
                chains,
            }
        }
    }
}

fn payment_transfer_destination(request: &GemPaymentRequest, asset: GemPaymentWalletAsset) -> GemPaymentDestination {
    match &request.asset_id {
        Some(asset_id) if asset_id != &asset.asset_id => GemPaymentDestination::Unsupported,
        _ => transfer_destination(&asset, request),
    }
}

fn payment_decoded_transfer(request: &GemPaymentRequest, asset: GemPaymentWalletAsset) -> Option<GemPaymentConfirmTransfer> {
    match &request.asset_id {
        Some(asset_id) if asset_id != &asset.asset_id => None,
        _ => confirm_transfer(&asset, request),
    }
}

fn transfer_destination(asset: &GemPaymentWalletAsset, request: &GemPaymentRequest) -> GemPaymentDestination {
    if requires_memo(asset.asset_id.chain, request) {
        return recipient_destination(asset, request);
    }
    match confirm_transfer(asset, request) {
        Some(transfer) => GemPaymentDestination::Confirm { transfer },
        None => recipient_destination(asset, request),
    }
}

fn recipient_destination(asset: &GemPaymentWalletAsset, request: &GemPaymentRequest) -> GemPaymentDestination {
    GemPaymentDestination::Recipient {
        asset_id: asset.asset_id.clone(),
        payment: GemPaymentRecipient {
            recipient: payment_recipient(request, Some(asset.asset_id.chain)),
            amount: exact_amount(request),
        },
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
        GemPaymentAmount::ExactValue(amount) => Some(amount.clone()),
        GemPaymentAmount::AtomicValue(_) => None,
    }
}

fn confirm_transfer(asset: &GemPaymentWalletAsset, request: &GemPaymentRequest) -> Option<GemPaymentConfirmTransfer> {
    let chain = asset.asset_id.chain;
    let address = checksum_address(&request.address, chain);
    if !validate_address(&address, chain) {
        return None;
    }
    let value = transfer_value(request, asset.decimals)?;
    Some(GemPaymentConfirmTransfer {
        asset_id: asset.asset_id.clone(),
        address,
        value,
        memo: request.memo.clone(),
        references: request.references.clone().unwrap_or_default(),
    })
}

fn requires_memo(chain: Chain, request: &GemPaymentRequest) -> bool {
    payment_memo_required(chain) && request.memo.as_deref().unwrap_or_default().is_empty()
}

fn payment_memo_required(chain: Chain) -> bool {
    match chain.chain_type() {
        ChainType::Cosmos | ChainType::Ton | ChainType::Xrp | ChainType::Stellar | ChainType::Algorand => true,
        ChainType::Solana
        | ChainType::Ethereum
        | ChainType::Bitcoin
        | ChainType::Near
        | ChainType::Tron
        | ChainType::Aptos
        | ChainType::Sui
        | ChainType::Polkadot
        | ChainType::Cardano
        | ChainType::HyperCore => false,
    }
}

fn transfer_value(request: &GemPaymentRequest, decimals: i32) -> Option<BigUint> {
    match request.amount.as_ref()? {
        GemPaymentAmount::ExactValue(amount) => BigNumberFormatter::value_from_amount_exact(amount, u32::try_from(decimals).ok()?).ok(),
        GemPaymentAmount::AtomicValue(value) => Some(value.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::payment::{GemPaymentAmount, GemPaymentLink, GemPaymentRequest};
    use crate::testkit::TestAlienProvider;
    use primitives::{ApplicationMetadata, Asset, AssetId, AssetType, Chain, ChainAddress, TransactionType};

    const BITCOIN_ADDRESS: &str = "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4";
    const ETHEREUM_ADDRESS: &str = "0x1f9090aaE28b8a3dCeaDf281B0F12828e676c326";
    const SOLANA_ADDRESS: &str = "HA4hQMs22nCuRN7iLDBsBkboz2SnLM1WkNtzLo6xEDY5";
    const XRP_ADDRESS: &str = "rEb8TK3gBgk5auZkwc6sHnwrGVJH8DuaLh";
    const USDC_MINT: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";

    fn request(address: &str, amount: Option<GemPaymentAmount>, memo: Option<&str>, asset_id: Option<AssetId>) -> GemPaymentRequest {
        GemPaymentRequest {
            address: address.to_string(),
            amount,
            memo: memo.map(str::to_string),
            label: None,
            references: None,
            asset_id,
        }
    }

    fn wallet_asset(asset_id: AssetId, decimals: i32) -> GemPaymentWalletAsset {
        GemPaymentWalletAsset { asset_id, decimals }
    }

    #[test]
    fn test_payment_destination() {
        let bitcoin = wallet_asset(AssetId::from_chain(Chain::Bitcoin), 8);
        let ethereum = wallet_asset(AssetId::from_chain(Chain::Ethereum), 18);
        let smartchain = wallet_asset(AssetId::from_chain(Chain::SmartChain), 18);
        let xrp = wallet_asset(AssetId::from_chain(Chain::Xrp), 6);
        let solana_usdc = wallet_asset(AssetId::from_token(Chain::Solana, USDC_MINT), 6);

        let exact_bitcoin = request(BITCOIN_ADDRESS, Some(GemPaymentAmount::ExactValue("0.0001".to_string())), None, None);
        match payment_destination(&exact_bitcoin, vec![bitcoin.clone()]) {
            GemPaymentDestination::Confirm { transfer } => {
                assert_eq!(transfer.value, BigUint::from(10_000u32));
                assert_eq!(transfer.address, BITCOIN_ADDRESS);
                assert_eq!(transfer.asset_id, bitcoin.asset_id);
            }
            destination => panic!("expected confirm, got {destination:?}"),
        }

        let address_only = request(BITCOIN_ADDRESS, None, None, None);
        match payment_destination(&address_only, vec![bitcoin.clone()]) {
            GemPaymentDestination::Recipient { asset_id, payment } => {
                assert_eq!(asset_id, bitcoin.asset_id);
                assert_eq!(payment.recipient.address, BITCOIN_ADDRESS);
                assert_eq!(payment.amount, None);
            }
            destination => panic!("expected recipient, got {destination:?}"),
        }

        let too_precise = request(BITCOIN_ADDRESS, Some(GemPaymentAmount::ExactValue("0.000000001".to_string())), None, None);
        match payment_destination(&too_precise, vec![bitcoin.clone()]) {
            GemPaymentDestination::Recipient { payment, .. } => assert_eq!(payment.amount.as_deref(), Some("0.000000001")),
            destination => panic!("expected recipient for excess precision, got {destination:?}"),
        }

        let multiple_chains = request(&ETHEREUM_ADDRESS.to_lowercase(), Some(GemPaymentAmount::ExactValue("2".to_string())), None, None);
        match payment_destination(&multiple_chains, vec![bitcoin.clone(), ethereum.clone(), smartchain.clone()]) {
            GemPaymentDestination::SelectAsset { payment, chains } => {
                assert_eq!(payment.recipient.address, ETHEREUM_ADDRESS.to_lowercase());
                assert_eq!(payment.amount.as_deref(), Some("2"));
                assert_eq!(chains, vec![Chain::Ethereum, Chain::SmartChain]);
            }
            destination => panic!("expected asset selection, got {destination:?}"),
        }

        let tagged_xrp = request(XRP_ADDRESS, Some(GemPaymentAmount::ExactValue("10".to_string())), Some("12345"), Some(xrp.asset_id.clone()));
        match payment_destination(&tagged_xrp, vec![xrp.clone()]) {
            GemPaymentDestination::Confirm { transfer } => {
                assert_eq!(transfer.value, BigUint::from(10_000_000u32));
                assert_eq!(transfer.memo.as_deref(), Some("12345"));
            }
            destination => panic!("expected confirm, got {destination:?}"),
        }

        let untagged_xrp = request(XRP_ADDRESS, Some(GemPaymentAmount::ExactValue("10".to_string())), None, Some(xrp.asset_id.clone()));
        match payment_destination(&untagged_xrp, vec![xrp]) {
            GemPaymentDestination::Recipient { payment, .. } => {
                assert_eq!(payment.recipient.memo, None);
                assert_eq!(payment.amount.as_deref(), Some("10"));
            }
            destination => panic!("expected recipient without a destination tag, got {destination:?}"),
        }

        let solana_usdc_payment = request(
            SOLANA_ADDRESS,
            Some(GemPaymentAmount::ExactValue("1".to_string())),
            None,
            Some(solana_usdc.asset_id.clone()),
        );
        match payment_destination(&solana_usdc_payment, vec![solana_usdc.clone()]) {
            GemPaymentDestination::Confirm { transfer } => {
                assert_eq!(transfer.value, BigUint::from(1_000_000u32));
                assert_eq!(transfer.memo, None);
            }
            destination => panic!("expected confirm for a Solana payment without a memo, got {destination:?}"),
        }

        let unknown_token = request(SOLANA_ADDRESS, None, None, Some(AssetId::from_token(Chain::Solana, "11111111111111111111111111111111")));
        assert_eq!(
            payment_destination(&unknown_token, vec![bitcoin, ethereum, solana_usdc]),
            GemPaymentDestination::Unsupported
        );
    }

    #[test]
    fn test_payment_transfer_destination() {
        let ethereum = wallet_asset(AssetId::from_chain(Chain::Ethereum), 18);

        let invalid_address = request("0x123", None, Some("order 7"), None);
        match payment_transfer_destination(&invalid_address, ethereum.clone()) {
            GemPaymentDestination::Recipient { asset_id, payment } => {
                assert_eq!(asset_id, ethereum.asset_id);
                assert_eq!(payment.recipient.address, "0x123");
                assert_eq!(payment.recipient.memo.as_deref(), Some("order 7"));
                assert_eq!(payment.amount, None);
            }
            destination => panic!("expected recipient review for an invalid address, got {destination:?}"),
        }

        let lowercase = request(&ETHEREUM_ADDRESS.to_lowercase(), Some(GemPaymentAmount::AtomicValue(BigUint::from(1u32))), None, None);
        match payment_transfer_destination(&lowercase, ethereum.clone()) {
            GemPaymentDestination::Confirm { transfer } => assert_eq!(transfer.address, ETHEREUM_ADDRESS),
            destination => panic!("expected confirm, got {destination:?}"),
        }

        let lowercase_without_amount = request(&ETHEREUM_ADDRESS.to_lowercase(), None, None, None);
        match payment_transfer_destination(&lowercase_without_amount, ethereum.clone()) {
            GemPaymentDestination::Recipient { payment, .. } => assert_eq!(payment.recipient.address, ETHEREUM_ADDRESS),
            destination => panic!("expected recipient, got {destination:?}"),
        }

        let mismatched = request(ETHEREUM_ADDRESS, None, None, Some(AssetId::from_chain(Chain::Bitcoin)));
        assert_eq!(payment_transfer_destination(&mismatched, ethereum.clone()), GemPaymentDestination::Unsupported);

        let payable = request(ETHEREUM_ADDRESS, Some(GemPaymentAmount::ExactValue("1.5".to_string())), None, None);
        match payment_transfer_destination(&payable, ethereum) {
            GemPaymentDestination::Confirm { transfer } => assert_eq!(transfer.value, BigUint::from(1_500_000_000_000_000_000u64)),
            destination => panic!("expected confirm, got {destination:?}"),
        }
    }

    #[test]
    fn test_payment_decoded_transfer() {
        let solana_usdc = wallet_asset(AssetId::from_token(Chain::Solana, USDC_MINT), 6);

        let decoded = request(
            SOLANA_ADDRESS,
            Some(GemPaymentAmount::AtomicValue(19_000_000u32.into())),
            None,
            Some(solana_usdc.asset_id.clone()),
        );
        let transfer = payment_decoded_transfer(&decoded, solana_usdc.clone()).expect("expected transfer without a memo");
        assert_eq!(transfer.value, BigUint::from(19_000_000u32));
        assert_eq!(transfer.address, SOLANA_ADDRESS);
        assert_eq!(transfer.memo, None);

        let mismatched = request(
            SOLANA_ADDRESS,
            Some(GemPaymentAmount::AtomicValue(19_000_000u32.into())),
            None,
            Some(AssetId::from_chain(Chain::Solana)),
        );
        assert_eq!(payment_decoded_transfer(&mismatched, solana_usdc.clone()), None);

        let missing_value = request(SOLANA_ADDRESS, None, None, Some(solana_usdc.asset_id.clone()));
        assert_eq!(payment_decoded_transfer(&missing_value, solana_usdc), None);
    }

    #[test]
    fn test_payment_asset_id_prefers_the_request_asset_over_the_chain_asset() {
        let transaction = |asset_id: Option<AssetId>| GemPaymentTransaction {
            merchant: ApplicationMetadata::mock(),
            account: ChainAddress::new(Chain::Solana, SOLANA_ADDRESS.to_string()),
            transaction: "encoded".to_string(),
            transaction_type: TransactionType::Transfer,
            memo: None,
            request: Some(GemPaymentRequest {
                address: SOLANA_ADDRESS.to_string(),
                amount: None,
                memo: None,
                label: None,
                references: None,
                asset_id,
            }),
        };
        let usdc = AssetId::from_token(Chain::Solana, USDC_MINT);

        assert_eq!(payment_asset_id(&transaction(Some(usdc.clone()))), usdc);
        assert_eq!(payment_asset_id(&transaction(None)), AssetId::from_chain(Chain::Solana));
        assert_eq!(
            payment_asset_id(&GemPaymentTransaction {
                request: None,
                ..transaction(None)
            }),
            AssetId::from_chain(Chain::Solana)
        );
    }

    #[test]
    fn test_transaction_transfer_data() {
        let solana_usdc = AssetId::from_token(Chain::Solana, USDC_MINT);
        let asset = Asset::new(solana_usdc.clone(), "USD Coin".to_string(), "USDC".to_string(), 6, AssetType::SPL);
        let transaction = |request: Option<GemPaymentRequest>| GemPaymentTransaction {
            merchant: ApplicationMetadata::mock(),
            account: ChainAddress::new(Chain::Solana, SOLANA_ADDRESS.to_string()),
            transaction: "encoded".to_string(),
            transaction_type: TransactionType::Transfer,
            memo: Some("order 7".to_string()),
            request,
        };
        let service = GemPaymentService::new(Arc::new(TestAlienProvider::with_status(200)));

        let decoded = service.transaction_transfer_data(
            transaction(Some(request(
                SOLANA_ADDRESS,
                Some(GemPaymentAmount::AtomicValue(19_000_000u32.into())),
                None,
                Some(solana_usdc),
            ))),
            asset.clone(),
        );
        assert_eq!(decoded.recipient.address, SOLANA_ADDRESS);
        assert_eq!(decoded.value, 19_000_000.into());
        match &decoded.input_type {
            GemTransactionInputType::Generic { extra, .. } => {
                assert_eq!(extra.to, SOLANA_ADDRESS);
                assert_eq!(extra.data.as_deref(), Some(b"encoded".as_slice()));
                assert_eq!(extra.output_type, TransferDataOutputType::EncodedTransaction);
            }
            input_type => panic!("expected a generic input type, got {input_type:?}"),
        }

        let hex_encoded = GemPaymentTransaction {
            transaction: "0x0a0b".to_string(),
            ..transaction(None)
        };
        match &service.transaction_transfer_data(hex_encoded, asset.clone()).input_type {
            GemTransactionInputType::Generic { extra, .. } => assert_eq!(extra.data.as_deref(), Some([0x0a, 0x0b].as_slice())),
            input_type => panic!("expected a generic input type, got {input_type:?}"),
        }

        let undecodable = service.transaction_transfer_data(transaction(None), asset);
        assert_eq!(undecodable.recipient.address, "");
        assert_eq!(undecodable.recipient.memo.as_deref(), Some("order 7"));
        assert_eq!(undecodable.value, 0.into());
    }

    #[test]
    fn test_request() {
        let decode_url = |url: &str| GemPaymentService::new(Arc::new(TestAlienProvider::with_status(200))).decode_url(url.to_string());
        assert_eq!(
            decode_url("solana:3u3ta6yXYgpheLGc2GVF3QkLHAUwBrvX71Eg8XXjJHGw?amount=0.42301").unwrap(),
            GemPayment::Request(GemPaymentRequest {
                address: "3u3ta6yXYgpheLGc2GVF3QkLHAUwBrvX71Eg8XXjJHGw".to_string(),
                amount: Some(GemPaymentAmount::ExactValue("0.42301".to_string())),
                memo: None,
                label: None,
                asset_id: Some(AssetId::from_chain(Chain::Solana)),
                references: None,
            })
        );
    }

    #[test]
    fn test_link() {
        let decode_url = |url: &str| GemPaymentService::new(Arc::new(TestAlienProvider::with_status(200))).decode_url(url.to_string());
        const CONSTANT_K: &str = "https://www.constant-k.com/ck-txreq/?tok=MjYyfG9wZXJhdG9yfGFubnVhbHx8MTc4NzUyOTMxOXw3M2FiNDFhZmIwNTAxZWNjNjE2Y2E4NmIxZGE5N2FlOWZjM2Y1OGMzZWZhMGYxMjNiOGI4ZGYzZmU2YzQ3ZmM4";

        assert_eq!(
            decode_url("solana:https%3A%2F%2Fapi.spherepay.co%2Fv1%2Fpublic%2FpaymentLink%2Fpay%2FpaymentLink_1").unwrap(),
            GemPayment::Link(GemPaymentLink::SolanaPay {
                url: "https://api.spherepay.co/v1/public/paymentLink/pay/paymentLink_1".to_string(),
            })
        );
        assert_eq!(
            decode_url("solana:https%3A%2F%2Fwww.constant-k.com%2Fck-txreq%2F%3Ftok%3DMjYyfG9wZXJhdG9yfGFubnVhbHx8MTc4NzUyOTMxOXw3M2FiNDFhZmIwNTAxZWNjNjE2Y2E4NmIxZGE5N2FlOWZjM2Y1OGMzZWZhMGYxMjNiOGI4ZGYzZmU2YzQ3ZmM4").unwrap(),
            GemPayment::Link(GemPaymentLink::SolanaPay {
                url: CONSTANT_K.to_string(),
            })
        );
        assert!(decode_url("https://pay.walletconnect.com/?pid=pay_123").is_err());
    }
}
