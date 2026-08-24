use crate::testkit::signer_mock::TEST_EVM_RECIPIENT;
use crate::{AssetId, Chain, PaymentAction, PaymentLink, PaymentMerchant, PaymentQuote, PaymentQuoteData, PaymentQuotes, PaymentRequest};

pub const TEST_PAYMENT_ID: &str = "pay_123";

impl PaymentRequest {
    pub fn mock() -> Self {
        Self {
            address: "0xcB3028d6120802148f03d6c884D6AD6A210Df62A".to_string(),
            amount: None,
            memo: None,
            asset_id: None,
        }
    }
}

impl PaymentMerchant {
    pub fn mock() -> Self {
        Self {
            name: "Merchant".to_string(),
            icon_url: None,
        }
    }
}

impl PaymentQuote {
    pub fn mock() -> Self {
        Self::mock_with_chain(Chain::Ethereum)
    }

    pub fn mock_with_chain(chain: Chain) -> Self {
        Self {
            id: chain.as_ref().to_string(),
            link: PaymentLink::WalletConnectPay(TEST_PAYMENT_ID.to_string()),
            asset_id: AssetId::from_chain(chain),
            value: 1u32.into(),
            collect_data_url: None,
            provider_data: "{}".to_string(),
        }
    }
}

impl PaymentQuotes {
    pub fn mock(quotes: Vec<PaymentQuote>) -> Self {
        Self {
            merchant: PaymentMerchant::mock(),
            price: None,
            quotes,
        }
    }
}

impl PaymentAction {
    pub fn mock_send(chain: Chain) -> Self {
        Self::Send {
            chain,
            recipient: TEST_EVM_RECIPIENT.to_string(),
            value: 1u32.into(),
            data: String::new(),
        }
    }
}

impl PaymentQuoteData {
    pub fn mock(quoted: Chain, signing: Chain) -> Self {
        Self {
            quote: PaymentQuote::mock_with_chain(quoted),
            action: PaymentAction::mock_send(signing),
        }
    }
}
