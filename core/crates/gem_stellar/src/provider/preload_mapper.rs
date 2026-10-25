use crate::models::swap::StellarSwapData;
use num_bigint::BigInt;
use primitives::{AssetId, Chain, FeeOption, TransactionFee, TransactionInputType, TransactionLoadData, TransactionLoadInput, swap::SwapQuoteDataType};
use std::error::Error;

pub fn map_transaction_load(input: TransactionLoadInput) -> Result<TransactionLoadData, Box<dyn Error + Send + Sync>> {
    let fee = match &input.input_type {
        TransactionInputType::Swap { swap_data, .. } if swap_data.data.data_type == SwapQuoteDataType::Contract => {
            let swap: StellarSwapData = serde_json::from_str(&swap_data.data.data)?;
            TransactionFee::calculate(swap.operation_count(), &input.gas_price, AssetId::from_chain(Chain::Stellar))
        }
        _ if input.metadata.get_is_destination_address_exist()? => input.default_fee(),
        _ => TransactionFee::new_from_fee_with_option(input.gas_price.gas_price(), FeeOption::TokenAccountCreation, BigInt::ZERO, AssetId::from_chain(Chain::Stellar)),
    };

    Ok(TransactionLoadData { fee, metadata: input.metadata })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::swap::StellarSwapFee;
    use num_bigint::BigUint;
    use primitives::{Asset, Chain, GasPriceType, SwapProvider, TransactionInputType, TransactionLoadMetadata, swap::SwapData};

    #[test]
    fn test_map_transaction_load_destination_exists() {
        let input = TransactionLoadInput {
            metadata: TransactionLoadMetadata::Stellar {
                sequence: 1,
                is_destination_address_exist: true,
            },
            ..TransactionLoadInput::mock()
        };

        let result = map_transaction_load(input).unwrap();

        assert_eq!(result.fee.fee, BigInt::from(1000));
        assert!(result.fee.options.is_empty());
    }

    #[test]
    fn test_map_transaction_load_destination_not_exist() {
        let input = TransactionLoadInput {
            input_type: TransactionInputType::Transfer { asset: Asset::from_chain(Chain::Stellar) },
            value: BigUint::from(15000000u64),
            gas_price: GasPriceType::regular(BigInt::from(100)),
            metadata: TransactionLoadMetadata::Stellar {
                sequence: 1,
                is_destination_address_exist: false,
            },
            ..TransactionLoadInput::mock()
        };

        let result = map_transaction_load(input).unwrap();

        assert_eq!(result.fee.fee, BigInt::from(100));
        assert!(result.fee.options.contains_key(&FeeOption::TokenAccountCreation));
        assert_eq!(result.fee.options.get(&FeeOption::TokenAccountCreation), Some(&BigInt::ZERO));
    }

    #[test]
    fn test_map_transaction_load_swap_pays_the_rate_for_every_operation() {
        let swap = |change_trust: bool| StellarSwapData {
            send_amount: 1,
            destination_min: 1,
            path: vec![],
            change_trust,
            fee: Some(StellarSwapFee {
                destination: String::new(),
                asset: AssetId::from_chain(Chain::Stellar),
                amount: 1,
            }),
            max_time: 1,
        };
        let input = |swap: &StellarSwapData| TransactionLoadInput {
            input_type: TransactionInputType::Swap {
                from_asset: Asset::from_chain(Chain::Stellar),
                to_asset: Asset::from_chain(Chain::Stellar),
                swap_data: SwapData::mock_with_provider_data(SwapProvider::StellarDex, &serde_json::to_string(swap).unwrap(), None),
            },
            gas_price: GasPriceType::regular(BigInt::from(17_208)),
            metadata: TransactionLoadMetadata::Stellar {
                sequence: 1,
                is_destination_address_exist: false,
            },
            ..TransactionLoadInput::mock()
        };

        assert_eq!(map_transaction_load(input(&swap(true))).unwrap().fee.fee, BigInt::from(3 * 17_208));
        assert_eq!(map_transaction_load(input(&swap(false))).unwrap().fee.fee, BigInt::from(2 * 17_208));
    }
}
