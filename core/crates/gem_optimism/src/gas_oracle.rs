use std::error::Error;

use alloy_primitives::Bytes;
use async_trait::async_trait;
use gem_client::Client;
use gem_evm::provider::preload;
use gem_evm::provider::preload_mapper::TransactionParams;
use gem_evm::rpc::{EthereumClient, EvmFeeCalculator, EvmStakingClient};
use gem_evm::u256::u256_to_biguint;
use num_bigint::BigInt;
use primitives::contract_constants::OPTIMISM_GAS_PRICE_ORACLE_CONTRACT;
use primitives::{TransactionFee, TransactionInputType, TransactionLoadInput, decode_hex};

use crate::contracts::IGasPriceOracle;

const EIP1559_TRANSACTION_TYPE: u8 = 0x02;
const RLP_EMPTY_LIST: u8 = 0xc0;

pub struct OptimismGasOracle<C: Client + Clone> {
    client: EthereumClient<C>,
}

#[async_trait]
impl<C: Client + Clone> EvmStakingClient for OptimismGasOracle<C> {}

#[async_trait]
impl<C: Client + Clone> EvmFeeCalculator for OptimismGasOracle<C> {
    async fn calculate_fee(&self, input: &TransactionLoadInput, params: &TransactionParams, gas_limit: &BigInt) -> Result<TransactionFee, Box<dyn Error + Sync + Send>> {
        let transaction_fee = preload::calculate_fee(input, gas_limit)?;
        Ok(TransactionFee {
            fee: transaction_fee.fee + self.l1_fee(&encode_transaction_for_l1_fee(input, params, gas_limit)?).await?,
            ..transaction_fee
        })
    }
}

impl<C: Client + Clone> OptimismGasOracle<C> {
    pub fn new(client: EthereumClient<C>) -> Self {
        Self { client }
    }

    async fn l1_fee(&self, data: &[u8]) -> Result<BigInt, Box<dyn Error + Sync + Send>> {
        let fee = self
            .client
            .call_contract(
                OPTIMISM_GAS_PRICE_ORACLE_CONTRACT.parse()?,
                IGasPriceOracle::getL1FeeCall {
                    data: Bytes::copy_from_slice(data),
                },
            )
            .await?;
        Ok(BigInt::from(u256_to_biguint(&fee)))
    }
}

fn l1_fee_value(input: &TransactionLoadInput, params: &TransactionParams, gas_limit: &BigInt) -> Result<BigInt, Box<dyn Error + Sync + Send>> {
    if spends_native_asset(&input.input_type) && input.is_max_value {
        Ok(input.value_as_bigint()? - gas_limit * input.gas_price.gas_price())
    } else {
        Ok(params.value.clone())
    }
}

fn spends_native_asset(input_type: &TransactionInputType) -> bool {
    match input_type {
        TransactionInputType::Transfer(asset) | TransactionInputType::Deposit(asset) | TransactionInputType::TransferNft(asset, _) | TransactionInputType::Account(asset, _) => {
            asset.id.is_native()
        }
        _ => false,
    }
}

fn encode_transaction_for_l1_fee(input: &TransactionLoadInput, params: &TransactionParams, gas_limit: &BigInt) -> Result<Vec<u8>, Box<dyn Error + Sync + Send>> {
    let value = l1_fee_value(input, params, gas_limit)?;
    let mut encoded = vec![EIP1559_TRANSACTION_TYPE];
    encoded.extend_from_slice(&input.metadata.get_chain_id_u64()?.to_be_bytes());
    encoded.extend_from_slice(&input.metadata.get_sequence()?.to_be_bytes());
    encoded.extend_from_slice(&input.gas_price.priority_fee().to_bytes_be().1);
    encoded.extend_from_slice(&input.gas_price.gas_price().to_bytes_be().1);
    encoded.extend_from_slice(&gas_limit.to_bytes_be().1);
    encoded.extend_from_slice(&decode_hex(&params.to)?);
    encoded.extend_from_slice(&value.to_bytes_be().1);
    encoded.extend_from_slice(&params.data);
    encoded.push(RLP_EMPTY_LIST);

    if spends_native_asset(&input.input_type) && encoded.len() > 3 {
        encoded.remove(2);
    }

    Ok(encoded)
}

#[cfg(test)]
mod tests {
    use alloy_primitives::hex::encode;
    use num_bigint::BigInt;
    use primitives::{Asset, Chain, GasPriceType, TransactionInputType, TransactionLoadInput, TransactionLoadMetadata};

    use super::*;

    fn encode_mock(input_type: TransactionInputType) -> Vec<u8> {
        let params = TransactionParams::new("0x000000000000000000000000000000000000dead".to_string(), vec![], BigInt::from(1_000_000_000_000_000_000u64));
        let mut input = TransactionLoadInput::mock_evm_with_metadata(input_type, "1000000000000000000", TransactionLoadMetadata::mock_evm(5, 10));
        input.gas_price = GasPriceType::eip1559(BigInt::from(2_000_000_000u64), BigInt::from(1_000_000_000u64));
        encode_transaction_for_l1_fee(&input, &params, &BigInt::from(21_000u64)).unwrap()
    }

    #[test]
    fn test_encode_transaction_for_l1_fee() {
        let encoded = encode_mock(TransactionInputType::Transfer(Asset::from_chain(Chain::Optimism)));
        assert_eq!(
            encode(&encoded),
            "020000000000000a00000000000000053b9aca00773594005208000000000000000000000000000000000000dead0de0b6b3a7640000c0"
        );

        let encoded_token = encode_mock(TransactionInputType::Transfer(Asset::mock_erc20()));
        assert_eq!(
            encode(&encoded_token),
            "02000000000000000a00000000000000053b9aca00773594005208000000000000000000000000000000000000dead0de0b6b3a7640000c0"
        );
        assert_eq!(encoded_token.len(), encoded.len() + 1);
        assert_eq!(encoded_token[0], EIP1559_TRANSACTION_TYPE);
    }
}
