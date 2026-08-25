use std::sync::Arc;

use ::simulation::evm::SimulationClient;
use chain_traits::ChainSimulation;
use gem_evm::jsonrpc::TransactionObject;
use gem_evm::rpc::{EthereumClient, EthereumProvider};
use gem_jsonrpc::grpc::AlienGrpcTransport;
use gem_solana::rpc::{SolanaClient, SolanaProvider};
use gem_sui::rpc::{SuiClient, SuiProvider};
use gem_ton::rpc::client::TonClient;
use gem_tron::rpc::{TronProvider, client::TronClient};
use gem_wallet_connect::{
    SignDigestType as WcSignDigestType, WCEthereumTransactionData as WcEthereumTransactionData, WalletConnectTransactionType as WcWalletConnectTransactionType,
};
use primitives::{Chain, EVMChain, SimulationInput, SimulationResult};

use crate::{
    GemstoneError,
    alien::{AlienClient, AlienProvider, AlienProviderWrapper, coalescing_provider, new_alien_client},
    message::sign_type::SignDigestType,
    network::JsonRpcClient,
    wallet_connect::{WalletConnectTransactionType, simulation},
};

#[derive(uniffi::Object)]
pub struct TransactionSimulationService {
    provider: Arc<dyn AlienProvider>,
}

#[uniffi::export]
impl TransactionSimulationService {
    #[uniffi::constructor]
    pub fn new(provider: Arc<dyn AlienProvider>) -> Self {
        Self {
            provider: coalescing_provider(provider),
        }
    }

    pub async fn simulate_sign_message(&self, chain: Chain, sign_type: SignDigestType, data: String, session_domain: String) -> Result<SimulationResult, GemstoneError> {
        let sign_type: WcSignDigestType = sign_type.into();
        let validation_warnings = simulation::sign_message_validation_warnings(chain, &sign_type, &data, &session_domain);

        let simulation = match sign_type {
            WcSignDigestType::Eip712 => match simulation::parse_eip712_message(&data) {
                Some(message) => self.simulate_eip712_message(chain, &message).await?,
                None => SimulationResult::default(),
            },
            _ => SimulationResult::default(),
        };

        Ok(simulation.prepend_warnings(validation_warnings))
    }

    pub async fn simulate_send_transaction(&self, chain: Chain, transaction_type: WalletConnectTransactionType, data: String) -> Result<SimulationResult, GemstoneError> {
        let transaction_type: WcWalletConnectTransactionType = transaction_type.into();
        let validation_warnings = simulation::send_transaction_validation_warnings(&transaction_type, &data);

        let simulation = match &transaction_type {
            WcWalletConnectTransactionType::Ethereum => self.simulate_ethereum_transaction(chain, &data).await,
            WcWalletConnectTransactionType::Solana { .. } | WcWalletConnectTransactionType::Sui { .. } => self.simulate_encoded_transaction(&transaction_type, &data).await,
            WcWalletConnectTransactionType::Ton { .. } => self.simulate_chain_transaction(Chain::Ton, SimulationInput::new(&data)).await,
            WcWalletConnectTransactionType::Tron { .. } => self.simulate_chain_transaction(Chain::Tron, SimulationInput::new(&data)).await,
        }
        .unwrap_or_default();

        Ok(simulation.prepend_warnings(validation_warnings))
    }

    pub async fn simulate_transaction(&self, chain: Chain, encoded_transaction: String, signer_address: Option<String>) -> Result<SimulationResult, GemstoneError> {
        self.simulate_chain_transaction(
            chain,
            SimulationInput {
                encoded_transaction,
                signer_address,
            },
        )
        .await
    }
}

impl TransactionSimulationService {
    async fn simulate_eip712_message(&self, chain: Chain, message: &gem_evm::eip712::EIP712Message) -> Result<SimulationResult, GemstoneError> {
        let provider = self.ethereum_provider(chain)?;
        Ok(SimulationClient::new(&provider).simulate_eip712_message(chain, message).await?)
    }

    async fn simulate_ethereum_transaction(&self, chain: Chain, data: &str) -> Result<SimulationResult, GemstoneError> {
        let transaction = simulation::decode_ethereum_transaction(data)?;
        let calldata = simulation::decode_ethereum_calldata(&transaction);
        let provider = self.ethereum_provider(chain)?;

        if ::simulation::evm::is_approval(chain, &calldata, &transaction.to) {
            return Ok(SimulationClient::new(&provider).simulate_evm_calldata(chain, &calldata, &transaction.to).await?);
        }

        let (calldata_result, balance_result) = self.simulate_calldata_and_balance_changes(chain, &calldata, &provider, &transaction).await;
        let calldata_result = calldata_result?;
        let balance_result = balance_result.unwrap_or_default();

        Ok(SimulationResult {
            balance_changes: balance_result.balance_changes,
            ..calldata_result
        }
        .prepend_warnings(balance_result.warnings))
    }

    async fn simulate_calldata_and_balance_changes(
        &self,
        chain: Chain,
        calldata: &[u8],
        provider: &EthereumProvider<AlienClient>,
        transaction: &WcEthereumTransactionData,
    ) -> (Result<SimulationResult, GemstoneError>, Result<SimulationResult, GemstoneError>) {
        let calldata_task = async {
            if calldata.is_empty() {
                Ok(SimulationResult::default())
            } else {
                SimulationClient::new(provider)
                    .simulate_evm_calldata(chain, calldata, &transaction.to)
                    .await
                    .map_err(GemstoneError::from)
            }
        };
        futures::join!(calldata_task, self.simulate_ethereum_balance_changes(provider, transaction))
    }

    async fn simulate_ethereum_balance_changes(
        &self,
        provider: &EthereumProvider<AlienClient>,
        transaction: &WcEthereumTransactionData,
    ) -> Result<SimulationResult, GemstoneError> {
        let encoded_transaction = serde_json::to_string(&map_transaction_object(transaction)).map_err(|error| error.to_string())?;

        Ok(provider.simulate_transaction(SimulationInput::new(encoded_transaction)).await?)
    }

    async fn simulate_encoded_transaction(&self, transaction_type: &WcWalletConnectTransactionType, data: &str) -> Result<SimulationResult, GemstoneError> {
        let chain = match transaction_type {
            WcWalletConnectTransactionType::Solana { .. } => Chain::Solana,
            WcWalletConnectTransactionType::Sui { .. } => Chain::Sui,
            _ => return Err("Chain does not use encoded transaction simulation".into()),
        };
        let input: SimulationInput = serde_json::from_str(data).map_err(|error| error.to_string())?;
        self.simulate_chain_transaction(chain, input).await
    }

    async fn simulate_chain_transaction(&self, chain: Chain, input: SimulationInput) -> Result<SimulationResult, GemstoneError> {
        Ok(self.chain_simulation(chain)?.simulate_transaction(input).await?)
    }

    fn ethereum_provider(&self, chain: Chain) -> Result<EthereumProvider<AlienClient>, GemstoneError> {
        let chain = EVMChain::from_chain(chain).ok_or_else(|| format!("{chain} is not an EVM chain"))?;
        let url = self.provider.get_endpoint(chain.to_chain())?;
        let client = new_alien_client(url, self.provider.clone());
        Ok(EthereumProvider::new_rpc_only(EthereumClient::new(JsonRpcClient::new(client), chain)))
    }

    fn chain_simulation(&self, chain: Chain) -> Result<Box<dyn ChainSimulation>, GemstoneError> {
        let url = self.provider.get_endpoint(chain)?;
        let new_client = || new_alien_client(url.clone(), self.provider.clone());
        match chain {
            Chain::Solana => Ok(Box::new(SolanaProvider::new_rpc_only(SolanaClient::new(JsonRpcClient::new(new_client()))))),
            Chain::Sui => {
                let transport = AlienGrpcTransport::new(Arc::new(AlienProviderWrapper::new(self.provider.clone())));
                Ok(Box::new(SuiProvider::new_rpc_only(SuiClient::new_with_transport(url, Arc::new(transport)))))
            }
            Chain::Ton => Ok(Box::new(TonClient::new(new_client()))),
            Chain::Tron => Ok(Box::new(TronProvider::new_rpc_only(TronClient::new(new_client())))),
            _ => Err(format!("{chain} does not support transaction simulation").into()),
        }
    }
}

/// Keeps the gas limit so out-of-gas failures surface, but omits fee prices - they make the trace charge gas and leak fee accounting into the signer's balance diff.
fn map_transaction_object(transaction: &WcEthereumTransactionData) -> TransactionObject {
    TransactionObject {
        from: Some(transaction.from.clone()),
        to: transaction.to.clone(),
        gas: transaction.gas.clone().or_else(|| transaction.gas_limit.clone()),
        gas_price: None,
        max_fee_per_gas: None,
        max_priority_fee_per_gas: None,
        value: transaction.value.clone(),
        data: transaction.data.clone().unwrap_or_default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alien::{AlienError, AlienResponse, AlienTarget};
    use async_trait::async_trait;
    use primitives::testkit::signer_mock::{TEST_EVM_RECIPIENT, TEST_EVM_SENDER};

    #[derive(Debug)]
    struct TestProvider;

    #[async_trait]
    impl AlienProvider for TestProvider {
        async fn request(&self, _target: AlienTarget) -> Result<Arc<AlienResponse>, AlienError> {
            unreachable!()
        }

        fn get_endpoint(&self, _chain: Chain) -> Result<String, AlienError> {
            Ok("https://example.invalid".to_string())
        }
    }

    fn mock_wc_transaction() -> WcEthereumTransactionData {
        WcEthereumTransactionData {
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

    #[test]
    fn test_map_transaction_object_normalizes_empty_calldata_to_0x() {
        let transaction_object = map_transaction_object(&mock_wc_transaction());

        assert_eq!(serde_json::to_value(transaction_object).unwrap()["data"], "0x");
    }

    #[test]
    fn test_map_transaction_object_passes_gas_limit_and_omits_fee_prices() {
        let transaction = WcEthereumTransactionData {
            gas_limit: Some("0x5208".to_string()),
            gas_price: Some("0x9502f900".to_string()),
            max_fee_per_gas: Some("0x59682f10".to_string()),
            max_priority_fee_per_gas: Some("0x3b9aca00".to_string()),
            ..mock_wc_transaction()
        };

        let transaction_object = map_transaction_object(&transaction);

        assert_eq!(transaction_object.gas.as_deref(), Some("0x5208"));
        assert_eq!(transaction_object.gas_price, None);
        assert_eq!(transaction_object.max_fee_per_gas, None);
        assert_eq!(transaction_object.max_priority_fee_per_gas, None);
    }

    #[test]
    fn test_wallet_connect_send_transaction_ignores_simulation_error() {
        futures::executor::block_on(async {
            let service = TransactionSimulationService::new(Arc::new(TestProvider));

            let result = service
                .simulate_send_transaction(
                    Chain::Solana,
                    WalletConnectTransactionType::Solana {
                        output_type: primitives::TransferDataOutputType::EncodedTransaction,
                    },
                    "invalid simulation input".to_string(),
                )
                .await;

            assert!(result.is_ok());
        });
    }
}
