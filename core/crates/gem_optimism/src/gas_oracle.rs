use std::error::Error;

use alloy_primitives::Bytes;
use async_trait::async_trait;
use gem_client::Client;
use gem_evm::provider::preload;
use gem_evm::provider::preload_mapper::TransactionParams;
use gem_evm::rpc::{EthereumClient, EvmFeeCalculator, EvmStakingClient};
use gem_evm::signer::Eip1559Transaction;
use gem_evm::u256::{bigint_to_u256, u256_to_biguint};
use num_bigint::BigInt;
use primitives::contract_constants::OPTIMISM_GAS_PRICE_ORACLE_CONTRACT;
use primitives::{TransactionFee, TransactionLoadInput};

use crate::contracts::IGasPriceOracle;

pub struct OptimismGasOracle<C: Client + Clone> {
    client: EthereumClient<C>,
}

#[async_trait]
impl<C: Client + Clone> EvmStakingClient for OptimismGasOracle<C> {}

#[async_trait]
impl<C: Client + Clone> EvmFeeCalculator for OptimismGasOracle<C> {
    async fn calculate_fee(&self, input: &TransactionLoadInput, params: &TransactionParams, gas_limit: &BigInt) -> Result<TransactionFee, Box<dyn Error + Sync + Send>> {
        let transaction_fee = preload::calculate_fee(input, gas_limit)?;
        let l1_fee = self.l1_fee(&l1_fee_transaction(input, params, &transaction_fee)?.encoded_unsigned()).await?;
        Ok(TransactionFee {
            fee: transaction_fee.fee + l1_fee,
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
            .call_contract(OPTIMISM_GAS_PRICE_ORACLE_CONTRACT.parse()?, IGasPriceOracle::getL1FeeCall { data: Bytes::copy_from_slice(data) })
            .await?;
        Ok(BigInt::from(u256_to_biguint(&fee)))
    }
}

fn l1_fee_transaction(input: &TransactionLoadInput, params: &TransactionParams, fee: &TransactionFee) -> Result<Eip1559Transaction, Box<dyn Error + Sync + Send>> {
    Ok(Eip1559Transaction {
        chain_id: input.metadata.get_chain_id_u64()?,
        nonce: input.metadata.get_sequence()?,
        gas_limit: fee.gas_limit()?,
        max_fee_per_gas: fee.gas_price_u64()?.into(),
        max_priority_fee_per_gas: fee.priority_fee_u64()?.into(),
        to: params.to.parse()?,
        value: bigint_to_u256(&params.value)?,
        input: Bytes::from(params.data.clone()),
    })
}

#[cfg(test)]
mod tests {
    use alloy_primitives::hex;
    use primitives::{Asset, Chain, GasPriceType, TransactionInputType, TransactionLoadMetadata};

    use super::*;

    #[test]
    fn test_l1_fee_transaction() {
        let transfer = TransactionLoadInput {
            gas_price: GasPriceType::eip1559(0u64, 100_151_000u64),
            ..TransactionLoadInput::mock_evm_with_metadata(
                TransactionInputType::Transfer { asset: Asset::from_chain(Chain::Optimism) },
                "50569306920000000000",
                TransactionLoadMetadata::mock_evm(16_465, 10),
            )
        };
        let transfer_params = TransactionParams::new("0x2787d7c796f5d4f46b8fdc3b859f82a9ef74422c", vec![], transfer.value_as_bigint());
        let transfer_fee = preload::calculate_fee(&transfer, &BigInt::from(42_000)).unwrap();
        assert_eq!(
            hex::encode(l1_fee_transaction(&transfer, &transfer_params, &transfer_fee).unwrap().encoded_unsigned()),
            "02f20a8240518405f82ed88405f82ed882a410942787d7c796f5d4f46b8fdc3b859f82a9ef74422c8902bdca44b7504d900080c0"
        );

        let token_transfer = TransactionLoadInput {
            gas_price: GasPriceType::eip1559(3_046u64, 1_000_000u64),
            ..TransactionLoadInput::mock_evm_with_metadata(TransactionInputType::Transfer { asset: Asset::mock_erc20() }, "257", TransactionLoadMetadata::mock_evm(3, 10))
        };
        let token_params = TransactionParams::new(
            "0x0b2c639c533813f4aa9d7837caf62653d097ff85",
            hex::decode("a9059cbb0000000000000000000000004c2c0f0bb2631b02ac9299c59690914ee7a200b80000000000000000000000000000000000000000000000000000000000000101").unwrap(),
            BigInt::from(0),
        );
        let token_fee = preload::calculate_fee(&token_transfer, &BigInt::from(45_415)).unwrap();
        assert_eq!(
            hex::encode(l1_fee_transaction(&token_transfer, &token_params, &token_fee).unwrap().encoded_unsigned()),
            "02f86a0a03830f4240830f4e2682b167940b2c639c533813f4aa9d7837caf62653d097ff8580b844a9059cbb0000000000000000000000004c2c0f0bb2631b02ac9299c59690914ee7a200b80000000000000000000000000000000000000000000000000000000000000101c0"
        );
    }
}
