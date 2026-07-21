use std::sync::Arc;

use ::simulation::evm::SimulationClient;
use chain_traits::ChainSimulation;
use gem_evm::jsonrpc::TransactionObject;
use gem_evm::rpc::EthereumClient;
use gem_jsonrpc::grpc::AlienGrpcTransport;
use gem_solana::{SolanaClient, rpc::SolanaIndexer};
use gem_sui::rpc::client::SuiClient;
use gem_tron::rpc::{client::TronClient, trongrid::client::TronGridClient};
use gem_wallet_connect::{
    SignDigestType as WcSignDigestType, WCEthereumTransactionData as WcEthereumTransactionData, WalletConnectTransactionType as WcWalletConnectTransactionType,
};
use primitives::{Chain, EVMChain, SimulationInput, SimulationResult};

use crate::{
    GemstoneError,
    alien::{AlienClient, AlienProvider, AlienProviderWrapper, coalescing_provider, new_alien_client},
    message::sign_type::SignDigestType,
    network::JsonRpcClient,
};

use super::{WalletConnectTransactionType, simulation};

#[derive(uniffi::Object)]
pub struct WalletConnectSimulationClient {
    provider: Arc<dyn AlienProvider>,
}

#[uniffi::export]
impl WalletConnectSimulationClient {
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
            WcWalletConnectTransactionType::Tron { .. } => self.simulate_tron_transaction(&data).await,
            _ => Ok(SimulationResult::default()),
        }
        .unwrap_or_default();

        Ok(simulation.prepend_warnings(validation_warnings))
    }
}

impl WalletConnectSimulationClient {
    async fn simulate_eip712_message(&self, chain: Chain, message: &gem_evm::eip712::EIP712Message) -> Result<SimulationResult, GemstoneError> {
        let client = self.ethereum_client(chain).ok_or("No RPC client available")?;
        Ok(SimulationClient::new(&client).simulate_eip712_message(chain, message).await?)
    }

    async fn simulate_ethereum_transaction(&self, chain: Chain, data: &str) -> Result<SimulationResult, GemstoneError> {
        let transaction = simulation::decode_ethereum_transaction(data)?;
        let calldata = simulation::decode_ethereum_calldata(&transaction);
        let client = self.ethereum_client(chain).ok_or("No RPC client available")?;

        if ::simulation::evm::is_approval(chain, &calldata, &transaction.to) {
            return Ok(SimulationClient::new(&client).simulate_evm_calldata(chain, &calldata, &transaction.to).await?);
        }

        let (calldata_result, balance_result) = self.simulate_calldata_and_balance_changes(chain, &calldata, &client, &transaction).await;
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
        client: &EthereumClient<AlienClient>,
        transaction: &WcEthereumTransactionData,
    ) -> (Result<SimulationResult, GemstoneError>, Result<SimulationResult, GemstoneError>) {
        let calldata_task = async {
            if calldata.is_empty() {
                Ok(SimulationResult::default())
            } else {
                SimulationClient::new(client)
                    .simulate_evm_calldata(chain, calldata, &transaction.to)
                    .await
                    .map_err(GemstoneError::from)
            }
        };
        futures::join!(calldata_task, self.simulate_ethereum_balance_changes(client, transaction))
    }

    async fn simulate_ethereum_balance_changes(&self, client: &EthereumClient<AlienClient>, transaction: &WcEthereumTransactionData) -> Result<SimulationResult, GemstoneError> {
        let encoded_transaction = serde_json::to_string(&map_transaction_object(transaction)).map_err(|error| error.to_string())?;

        Ok(client.simulate_transaction(SimulationInput::new(encoded_transaction)).await?)
    }

    async fn simulate_encoded_transaction(&self, transaction_type: &WcWalletConnectTransactionType, data: &str) -> Result<SimulationResult, GemstoneError> {
        let input: SimulationInput = serde_json::from_str(data).map_err(|error| error.to_string())?;
        let client: Box<dyn ChainSimulation> = match transaction_type {
            WcWalletConnectTransactionType::Solana { .. } => Box::new(self.solana_client().ok_or("No RPC client available")?),
            WcWalletConnectTransactionType::Sui { .. } => Box::new(self.sui_client().ok_or("No RPC client available")?),
            _ => return Err("Chain does not use encoded transaction simulation".into()),
        };
        Ok(client.simulate_transaction(input).await?)
    }

    async fn simulate_tron_transaction(&self, data: &str) -> Result<SimulationResult, GemstoneError> {
        let client = self.tron_client().ok_or("No RPC client available")?;
        Ok(client.simulate_transaction(SimulationInput::new(data)).await?)
    }

    fn ethereum_client(&self, chain: Chain) -> Option<EthereumClient<AlienClient>> {
        let chain = EVMChain::from_chain(chain)?;
        let url = self.provider.get_endpoint(chain.to_chain()).ok()?;
        let client = new_alien_client(url, self.provider.clone());
        Some(EthereumClient::new(JsonRpcClient::new(client), chain))
    }

    fn solana_client(&self) -> Option<SolanaClient<AlienClient>> {
        let url = self.provider.get_endpoint(Chain::Solana).ok()?;
        let client = new_alien_client(url, self.provider.clone());
        let client = JsonRpcClient::new(client);
        Some(SolanaClient::new(client.clone(), SolanaIndexer::new(client)))
    }

    fn sui_client(&self) -> Option<SuiClient> {
        let url = self.provider.get_endpoint(Chain::Sui).ok()?;
        let transport = AlienGrpcTransport::new(Arc::new(AlienProviderWrapper::new(self.provider.clone())));
        Some(SuiClient::new_with_transport(url, Arc::new(transport)))
    }

    fn tron_client(&self) -> Option<TronClient<AlienClient>> {
        let url = self.provider.get_endpoint(Chain::Tron).ok()?;
        let client = new_alien_client(url, self.provider.clone());
        Some(TronClient::new(client.clone(), TronGridClient::new(client, String::new())))
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
    use primitives::testkit::signer_mock::{TEST_EVM_RECIPIENT, TEST_EVM_SENDER};

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
}
