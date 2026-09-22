use crate::{Asset, AssetId, Chain, PaymentInvoice, PaymentLink, PaymentMerchant, PaymentPrice, PaymentQuote, PaymentRequest, TransactionInputType, TransferDataExtra};
use num_bigint::BigUint;

impl PaymentRequest {
    pub fn mock() -> Self {
        Self {
            address: "0xcB3028d6120802148f03d6c884D6AD6A210Df62A".to_string(),
            amount: None,
            memo: None,
            label: None,
            references: None,
            asset_id: None,
        }
    }
}

impl PaymentInvoice {
    pub fn mock() -> Self {
        Self {
            link: PaymentLink::WalletConnectPay { payment_id: "pay_123".to_string() },
            merchant: PaymentMerchant::mock(),
            price: Some(PaymentPrice::mock()),
            quotes: vec![PaymentQuote::mock(AssetId::from_chain(Chain::Ethereum))],
            verification: None,
        }
    }
}

impl PaymentMerchant {
    pub fn mock() -> Self {
        Self {
            name: "Gem Wallet Test Merchant".to_string(),
            icon: "https://imagedelivery.net/_aTEfDRm7z3tKgu9JhfeKA/28f1b431-9d2a-4083-1bf8-5958939a2300/md".to_string(),
        }
    }
}

impl PaymentPrice {
    pub fn mock() -> Self {
        Self { currency: "USD".to_string(), amount: 0.10 }
    }
}

impl TransactionInputType {
    pub fn mock_payment(asset: Asset, extra: TransferDataExtra) -> Self {
        Self::Payment {
            asset,
            invoice: PaymentInvoice::mock(),
            extra,
        }
    }
}

impl PaymentQuote {
    pub fn mock(asset_id: AssetId) -> Self {
        Self {
            id: format!("option-{asset_id}"),
            asset_id,
            value: BigUint::from(1_000_000_000_000_000u64),
        }
    }
}
