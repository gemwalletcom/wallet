use crate::alien::{AlienError, AlienHttpMethod, AlienProvider, AlienResponse, AlienTarget};
use crate::models::gateway::GemFeeRate;
use crate::models::transaction::{GemFeeOptions, GemSignedTransaction, GemTransactionLoadFee};
use crate::services::error::GemServiceError;
use crate::services::preferences::{GemPreferencesStore, GemSecureStore};
use async_trait::async_trait;
use gem_client::{CONTENT_TYPE, ContentType};
use gem_wallet_connect::WCEthereumTransactionData;
use num_bigint::BigInt;
use payment::PaymentTransaction;
use primitives::testkit::signer_mock::{TEST_EVM_RECIPIENT, TEST_EVM_SENDER};
use primitives::{ApplicationMetadata, AssetId, Chain, ChainAddress, FeePriority, GasPriceType, TransactionType};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct TestAlienProvider {
    response: Arc<AlienResponse>,
    by_path: Vec<(String, Arc<AlienResponse>)>,
    requested: Mutex<Vec<String>>,
    error: Option<AlienError>,
}

impl TestAlienProvider {
    pub fn new(response: AlienResponse) -> Self {
        Self {
            response: Arc::new(response),
            by_path: Vec::new(),
            requested: Mutex::new(Vec::new()),
            error: None,
        }
    }

    pub fn offline() -> Self {
        Self {
            error: Some(AlienError::Offline),
            ..Self::with_status(200)
        }
    }

    pub fn with_json_by_path(status: u16, bodies: &[(&str, &str)]) -> Self {
        Self {
            by_path: bodies
                .iter()
                .map(|(path, body)| ((*path).to_string(), Arc::new(AlienResponse::new(Some(status), body.as_bytes().to_vec()))))
                .collect(),
            ..Self::with_json(status, "[]")
        }
    }

    pub fn with_status(status: u16) -> Self {
        Self::new(AlienResponse::new(Some(status), Vec::new()))
    }

    pub fn with_json(status: u16, body: &str) -> Self {
        Self::new(AlienResponse::new(Some(status), body.as_bytes().to_vec()))
    }

    pub fn requested_paths(&self) -> Vec<String> {
        self.requested.lock().unwrap().clone()
    }
}

#[async_trait]
impl AlienProvider for TestAlienProvider {
    async fn request(&self, target: AlienTarget) -> Result<Arc<AlienResponse>, AlienError> {
        let path = target.url.find("/v").map(|index| target.url[index..].to_string()).unwrap_or(target.url);
        self.requested.lock().unwrap().push(path.clone());
        if let Some(error) = &self.error {
            return Err(error.clone());
        }
        let matched = self.by_path.iter().find(|(fragment, _)| path.contains(fragment.as_str()));
        Ok(matched.map(|(_, response)| response.clone()).unwrap_or_else(|| self.response.clone()))
    }
}

pub fn mock_alien_target(request_type: &str) -> AlienTarget {
    AlienTarget {
        url: "https://example.com/info".to_string(),
        method: AlienHttpMethod::Post,
        headers: Some(HashMap::from([(CONTENT_TYPE.to_string(), ContentType::ApplicationJson.as_str().to_string())])),
        body: Some(serde_json::to_vec(&serde_json::json!({ "type": request_type })).unwrap()),
    }
}

pub fn mock_wc_ethereum_transaction_data() -> WCEthereumTransactionData {
    WCEthereumTransactionData {
        chain_id: None,
        from: TEST_EVM_SENDER.to_string(),
        to: TEST_EVM_RECIPIENT.to_string(),
        value: Some("0x2386f26fc10000".to_string()),
        gas: None,
        gas_limit: None,
        gas_price: None,
        max_fee_per_gas: None,
        max_priority_fee_per_gas: None,
        nonce: None,
        data: None,
    }
}

pub fn mock_payment_transaction() -> PaymentTransaction {
    PaymentTransaction {
        merchant: ApplicationMetadata::mock(),
        account: ChainAddress::new(Chain::Solana, "HA4hQMs22nCuRN7iLDBsBkboz2SnLM1WkNtzLo6xEDY5".to_string()),
        transaction: "encoded".to_string(),
        transaction_type: TransactionType::Transfer,
        memo: None,
        request: None,
    }
}

impl GemTransactionLoadFee {
    pub fn mock(fee: u64) -> Self {
        Self {
            fee: BigInt::from(fee),
            gas_price_type: GasPriceType::regular(1),
            gas_limit: BigInt::from(21_000),
            options: GemFeeOptions { options: HashMap::new() },
            fee_asset: AssetId::from_chain(Chain::Ethereum),
        }
    }
}

impl GemSignedTransaction {
    pub fn mock(transaction_type: TransactionType) -> Self {
        Self {
            data: "signed".to_string(),
            transaction_type,
        }
    }
}

impl GemFeeRate {
    pub fn mock(priority: FeePriority, gas_price: u64) -> Self {
        Self {
            priority,
            gas_price_type: GasPriceType::regular(gas_price),
        }
    }
}

#[derive(Debug, Default)]
pub struct EmptyPreferences;

impl GemSecureStore for EmptyPreferences {
    fn get(&self, _key: String) -> Result<Option<String>, GemServiceError> {
        Ok(None)
    }

    fn set(&self, _key: String, _value: String) -> Result<(), GemServiceError> {
        Ok(())
    }

    fn remove(&self, _key: String) -> Result<(), GemServiceError> {
        Ok(())
    }
}

impl GemPreferencesStore for EmptyPreferences {
    fn get(&self, _key: String) -> Option<String> {
        None
    }

    fn set(&self, _key: String, _value: String) -> Result<(), GemServiceError> {
        Ok(())
    }

    fn remove(&self, _key: String) -> Result<(), GemServiceError> {
        Ok(())
    }

    fn clear(&self) -> Result<(), GemServiceError> {
        Ok(())
    }
}
