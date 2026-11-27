use std::error::Error;

use alloy_primitives::Bytes;
use async_trait::async_trait;
use futures::future::try_join_all;
use gem_client::Client;
use gem_evm::provider::preload;
use gem_evm::rpc::{EthereumClient, EvmFeeCalculator, EvmStakingClient};
use gem_evm::signer::Eip1559Transaction;
use gem_evm::transaction_params::TransactionCalls;
use gem_evm::u256::u256_to_biguint;
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
    async fn calculate_fee(&self, input: &TransactionLoadInput, calls: &TransactionCalls, gas_limit: &BigInt) -> Result<TransactionFee, Box<dyn Error + Sync + Send>> {
        let transaction_fee = preload::calculate_fee(input, calls, gas_limit)?;
        let l1_fees = try_join_all(l1_fee_transactions(input, calls, &transaction_fee)?.iter().map(|transaction| self.l1_fee(transaction.encoded_unsigned()))).await?;
        Ok(TransactionFee {
            fee: transaction_fee.fee + l1_fees.into_iter().sum::<BigInt>(),
            ..transaction_fee
        })
    }
}

impl<C: Client + Clone> OptimismGasOracle<C> {
    pub fn new(client: EthereumClient<C>) -> Self {
        Self { client }
    }

    async fn l1_fee(&self, data: Vec<u8>) -> Result<BigInt, Box<dyn Error + Sync + Send>> {
        let fee = self.client.call_contract(OPTIMISM_GAS_PRICE_ORACLE_CONTRACT.parse()?, IGasPriceOracle::getL1FeeCall { data: Bytes::from(data) }).await?;
        Ok(BigInt::from(u256_to_biguint(&fee)))
    }
}

fn l1_fee_transactions(input: &TransactionLoadInput, calls: &TransactionCalls, fee: &TransactionFee) -> Result<Vec<Eip1559Transaction>, Box<dyn Error + Sync + Send>> {
    Ok(Eip1559Transaction::from_calls(input, fee, calls)?)
}

#[cfg(test)]
mod tests {
    use alloy_primitives::hex;
    use gem_evm::transaction_params::transaction_calls;
    use num_bigint::BigUint;
    use primitives::asset_constants::OPTIMISM_USDC_TOKEN_ID;
    use primitives::swap::{ApprovalData, SwapData, SwapQuoteData};
    use primitives::{Asset, Chain, GasPriceType, TransactionInputType, TransactionLoadMetadata};

    use super::*;

    const TEST_SWAP_ROUTER: &str = "0x233c5370ccfb3cd7409d9a3fb98ab94de94cb4cd";

    #[test]
    fn test_l1_fee_transactions() {
        let encoded = |input: &TransactionLoadInput, gas_limit: u64| {
            let calls = transaction_calls(input).unwrap();
            let fee = preload::calculate_fee(input, &calls, &BigInt::from(gas_limit)).unwrap();
            let transactions = l1_fee_transactions(input, &calls, &fee).unwrap();
            transactions.iter().map(|transaction| hex::encode(transaction.encoded_unsigned())).collect::<Vec<_>>()
        };

        let transfer = TransactionLoadInput {
            destination_address: "0x2787d7c796f5d4f46b8fdc3b859f82a9ef74422c".to_string(),
            gas_price: GasPriceType::eip1559(0u64, 100_151_000u64),
            ..TransactionLoadInput::mock_evm_with_metadata(
                TransactionInputType::Transfer { asset: Asset::from_chain(Chain::Optimism) },
                "50569306920000000000",
                TransactionLoadMetadata::mock_evm(16_465, 10),
            )
        };
        assert_eq!(encoded(&transfer, 42_000), ["02f20a8240518405f82ed88405f82ed882a410942787d7c796f5d4f46b8fdc3b859f82a9ef74422c8902bdca44b7504d900080c0"]);

        let token_transfer = TransactionLoadInput {
            destination_address: "0x4c2c0f0bb2631b02ac9299c59690914ee7a200b8".to_string(),
            gas_price: GasPriceType::eip1559(3_046u64, 1_000_000u64),
            ..TransactionLoadInput::mock_evm_with_metadata(TransactionInputType::Transfer { asset: Asset::mock_optimism_usdc() }, "257", TransactionLoadMetadata::mock_evm(3, 10))
        };
        assert_eq!(
            encoded(&token_transfer, 45_415),
            ["02f86a0a03830f4240830f4e2682b167940b2c639c533813f4aa9d7837caf62653d097ff8580b844a9059cbb0000000000000000000000004c2c0f0bb2631b02ac9299c59690914ee7a200b80000000000000000000000000000000000000000000000000000000000000101c0"]
        );

        let swap = TransactionLoadInput {
            gas_price: GasPriceType::eip1559(1_788u64, 4_500_000u64),
            ..TransactionLoadInput::mock_evm_with_metadata(
                TransactionInputType::Swap {
                    from_asset: Asset::mock_optimism_usdc(),
                    to_asset: Asset::from_chain(Chain::Optimism),
                    swap_data: SwapData {
                        data: SwapQuoteData {
                            approval: Some(ApprovalData::make(OPTIMISM_USDC_TOKEN_ID, TEST_SWAP_ROUTER, BigUint::ZERO, true)),
                            gas_limit: Some("193588".to_string()),
                            ..SwapQuoteData::mock_contract_call(
                                TEST_SWAP_ROUTER,
                                "0",
                                "0x50df638b00000000000000000000000000000000000000000000000000000000000000600000000000000000000000000b2c639c533813f4aa9d7837caf62653d097ff8500000000000000000000000000000000000000000000000000000000042b94c800000000000000000000000000000000000000000000000000000000000000203d17e83257a0f142b55fdc67dc61e2b588acd25340f013863bccc7d4b68d25c8",
                                None,
                            )
                        },
                        ..SwapData::mock()
                    },
                },
                "70030536",
                TransactionLoadMetadata::mock_evm(0, 10),
            )
        };
        assert_eq!(
            encoded(&swap, 73_565),
            [
                "02f86b0a808344aa208344b11c83011f5d940b2c639c533813f4aa9d7837caf62653d097ff8580b844095ea7b3000000000000000000000000233c5370ccfb3cd7409d9a3fb98ab94de94cb4cdffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffc0",
                "02f8cb0a018344aa208344b11c8302f43494233c5370ccfb3cd7409d9a3fb98ab94de94cb4cd80b8a450df638b00000000000000000000000000000000000000000000000000000000000000600000000000000000000000000b2c639c533813f4aa9d7837caf62653d097ff8500000000000000000000000000000000000000000000000000000000042b94c800000000000000000000000000000000000000000000000000000000000000203d17e83257a0f142b55fdc67dc61e2b588acd25340f013863bccc7d4b68d25c8c0",
            ]
        );
    }
}
