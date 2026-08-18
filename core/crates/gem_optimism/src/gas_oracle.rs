use std::collections::HashMap;
use std::error::Error;

use alloy_primitives::hex;
use async_trait::async_trait;
use gem_client::Client;
use gem_evm::provider::preload_mapper::{TransactionParams, get_extra_fee_gas_limit};
use gem_evm::rpc::{EthereumClient, EvmFeeCalculator};
use num_bigint::{BigInt, Sign};
use primitives::{GasPriceType, TransactionFee, TransactionInputType, TransactionLoadInput, contract_constants::OPTIMISM_GAS_PRICE_ORACLE_CONTRACT};

pub struct OptimismGasOracle<C: Client + Clone> {
    client: EthereumClient<C>,
}

#[async_trait]
impl<C: Client + Clone> EvmFeeCalculator for OptimismGasOracle<C> {
    async fn calculate_fee(&self, input: &TransactionLoadInput, params: &TransactionParams, gas_limit: &BigInt) -> Result<TransactionFee, Box<dyn Error + Sync + Send>> {
        let nonce = input.metadata.get_sequence()?;
        let chain_id = input.metadata.get_chain_id()?.parse::<u64>()?;

        let extra_gas_limit = get_extra_fee_gas_limit(input)?;

        let adjusted_value = match &input.input_type {
            TransactionInputType::Transfer(asset)
            | TransactionInputType::Deposit(asset)
            | TransactionInputType::TransferNft(asset, _)
            | TransactionInputType::Account(asset, _) => {
                if asset.id.is_native() && input.is_max_value {
                    let parsed_value = input.get_value()?;
                    parsed_value - gas_limit * &input.gas_price.gas_price()
                } else {
                    params.value.clone()
                }
            }
            _ => params.value.clone(),
        };

        let encoded = self.encode_transaction_for_l1_fee(
            gas_limit,
            &input.gas_price.gas_price(),
            &input.gas_price.priority_fee(),
            nonce,
            Some(&params.data),
            &params.to,
            chain_id,
            Some(&adjusted_value),
            input,
        )?;

        let l1_fee = self.get_l1_fee(&encoded).await?;
        let l2_fee = &input.gas_price.total_fee() * (gas_limit + &extra_gas_limit);

        let fee = l1_fee + l2_fee;

        Ok(TransactionFee::new_gas_price_type(
            GasPriceType::eip1559(input.gas_price.total_fee(), input.gas_price.priority_fee()),
            fee,
            gas_limit.clone(),
            HashMap::new(),
            input.input_type.get_fee_asset_id(),
        ))
    }
}

impl<C: Client + Clone> OptimismGasOracle<C> {
    pub fn new(client: EthereumClient<C>) -> Self {
        Self { client }
    }

    async fn get_l1_fee(&self, data: &[u8]) -> Result<BigInt, Box<dyn Error + Send + Sync>> {
        let mut call_data = Vec::with_capacity(4 + 32 + data.len());
        call_data.extend_from_slice(&hex::decode("49948e0e")?);
        call_data.extend_from_slice(&[0u8; 31]);
        call_data.push(0x20);
        let data_len = data.len();
        let len_bytes = BigInt::from(data_len).to_bytes_be().1;
        let padding = 32_usize.saturating_sub(len_bytes.len());
        call_data.extend_from_slice(&vec![0u8; padding]);
        call_data.extend_from_slice(&len_bytes);
        call_data.extend_from_slice(data);
        let data_padding = data.len().div_ceil(32) * 32 - data.len();
        call_data.extend_from_slice(&vec![0u8; data_padding]);

        Ok(BigInt::from_bytes_be(
            Sign::Plus,
            &self.client.eth_call(OPTIMISM_GAS_PRICE_ORACLE_CONTRACT, &call_data).await?,
        ))
    }

    fn encode_transaction_for_l1_fee(
        &self,
        gas_limit: &BigInt,
        gas_price: &BigInt,
        priority_fee: &BigInt,
        nonce: u64,
        call_data: Option<&[u8]>,
        to: &str,
        chain_id: u64,
        value: Option<&BigInt>,
        input: &TransactionLoadInput,
    ) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
        let mut encoded = Vec::new();

        encoded.push(0x02);

        let mut rlp_data = Vec::new();

        rlp_data.extend_from_slice(&chain_id.to_be_bytes());
        rlp_data.extend_from_slice(&nonce.to_be_bytes());
        let priority_bytes = priority_fee.to_bytes_be().1;
        rlp_data.extend_from_slice(&priority_bytes);
        let gas_price_bytes = gas_price.to_bytes_be().1;
        rlp_data.extend_from_slice(&gas_price_bytes);
        let gas_limit_bytes = gas_limit.to_bytes_be().1;
        rlp_data.extend_from_slice(&gas_limit_bytes);
        let to_bytes = hex::decode(to.strip_prefix("0x").unwrap_or(to))?;
        rlp_data.extend_from_slice(&to_bytes);
        if let Some(v) = value {
            let value_bytes = v.to_bytes_be().1;
            rlp_data.extend_from_slice(&value_bytes);
        } else {
            rlp_data.push(0x80);
        }
        if let Some(d) = call_data {
            rlp_data.extend_from_slice(d);
        } else {
            rlp_data.push(0x80);
        }

        rlp_data.push(0xc0);

        encoded.extend_from_slice(&rlp_data);

        match &input.input_type {
            TransactionInputType::Transfer(asset)
            | TransactionInputType::Deposit(asset)
            | TransactionInputType::TransferNft(asset, _)
            | TransactionInputType::Account(asset, _)
                if asset.id.is_native() && encoded.len() > 3 =>
            {
                encoded.remove(2);
            }
            _ => {}
        }

        Ok(encoded)
    }
}

#[cfg(test)]
mod tests {
    use gem_client::testkit::MockClient;
    use gem_jsonrpc::JsonRpcClient;
    use primitives::{Asset, Chain, EVMChain};

    use super::*;

    #[test]
    fn test_encode_transaction_for_l1_fee() {
        let oracle = OptimismGasOracle::new(EthereumClient::new(JsonRpcClient::new(MockClient::new()), EVMChain::Optimism));
        let native_transfer = TransactionLoadInput::mock_evm(TransactionInputType::Transfer(Asset::from_chain(Chain::Optimism)), "1000000000000000000");

        let encoded = oracle
            .encode_transaction_for_l1_fee(
                &BigInt::from(21_000u64),
                &BigInt::from(2_000_000_000u64),
                &BigInt::from(1_000_000_000u64),
                5,
                None,
                "0x000000000000000000000000000000000000dead",
                10,
                Some(&BigInt::from(1_000_000_000_000_000_000u64)),
                &native_transfer,
            )
            .unwrap();

        assert_eq!(
            alloy_primitives::hex::encode(&encoded),
            "020000000000000a00000000000000053b9aca00773594005208000000000000000000000000000000000000dead0de0b6b3a764000080c0"
        );

        let token_transfer = TransactionLoadInput::mock_evm(TransactionInputType::Transfer(Asset::mock_erc20()), "1000000000000000000");
        let encoded_token = oracle
            .encode_transaction_for_l1_fee(
                &BigInt::from(21_000u64),
                &BigInt::from(2_000_000_000u64),
                &BigInt::from(1_000_000_000u64),
                5,
                None,
                "0x000000000000000000000000000000000000dead",
                10,
                Some(&BigInt::from(1_000_000_000_000_000_000u64)),
                &token_transfer,
            )
            .unwrap();

        // Token transfers keep the full prefix; native transfers drop the byte at index 2.
        assert_eq!(encoded_token.len(), encoded.len() + 1);
        assert_eq!(encoded_token[0], 0x02);
    }
}
