use std::error::Error;

use primitives::StakeType;
use primitives::swap::{SwapQuoteData, SwapQuoteDataType};

use crate::models::TransactionPayload;
use crate::signer::{build_delegation_pool_transaction_payload, build_token_transfer_transaction_payload, build_transfer_transaction_payload};

pub fn build_swap_transaction_payload(token_id: &Option<String>, swap_data: &SwapQuoteData) -> Result<TransactionPayload, Box<dyn Error + Send + Sync>> {
    match swap_data.data_type {
        SwapQuoteDataType::Transfer => match token_id {
            None => Ok(build_transfer_transaction_payload(&swap_data.to, &swap_data.value.to_string()).into()),
            Some(token_id) => Ok(build_token_transfer_transaction_payload(token_id, &swap_data.to, &swap_data.value.to_string())?.into()),
        },
        SwapQuoteDataType::Contract => Ok(serde_json::from_str::<TransactionPayload>(&swap_data.data)?),
    }
}

pub fn build_delegation_pool_payload_data(stake_type: &StakeType, amount: &str) -> Result<Option<String>, serde_json::Error> {
    build_delegation_pool_transaction_payload(stake_type, amount)
        .map(|payload| serde_json::to_string(&TransactionPayload::from(payload)))
        .transpose()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::APTOS_TRANSFER_FUNCTION;
    use num_bigint::BigUint;

    const TEST_POOL_ADDRESS: &str = "0xdb5247f859ce63dbe8940cf8773be722a60dcc594a8be9aca4b76abceb251b8e";

    #[test]
    fn test_build_swap_transaction_payload_transfer_native() {
        let swap_data = SwapQuoteData::new_transfer(TEST_POOL_ADDRESS.to_string(), BigUint::from(100000000u64), None);
        let payload = build_swap_transaction_payload(&None, &swap_data).unwrap();

        assert_eq!(payload.function.as_deref(), Some(APTOS_TRANSFER_FUNCTION));
    }

    #[test]
    fn test_build_swap_transaction_payload_transfer_fungible_asset() {
        let swap_data = SwapQuoteData::new_transfer(TEST_POOL_ADDRESS.to_string(), BigUint::from(1u64), None);
        let token_id = Some("0x357b0b74bc833e95a115ad22604854d6b0fca151cecd94111770e5d6ffc9dc2b".to_string());
        let payload = build_swap_transaction_payload(&token_id, &swap_data).unwrap();

        assert_eq!(payload.function.as_deref(), Some("0x1::primary_fungible_store::transfer"));
    }

    #[test]
    fn test_build_swap_transaction_payload_contract() {
        let contract_payload = TransactionPayload::from(build_transfer_transaction_payload(TEST_POOL_ADDRESS, "100000000"));
        let data = serde_json::to_string(&contract_payload).unwrap();
        let swap_data = SwapQuoteData::new_contract(TEST_POOL_ADDRESS.to_string(), BigUint::from(0u64), data, None, None);
        let payload = build_swap_transaction_payload(&None, &swap_data).unwrap();

        assert_eq!(payload.function, contract_payload.function);
    }

    #[test]
    fn test_build_swap_transaction_payload_transfer_invalid_token_id() {
        let swap_data = SwapQuoteData::new_transfer(TEST_POOL_ADDRESS.to_string(), BigUint::from(1u64), None);
        let token_id = Some("invalid".to_string());
        let err = build_swap_transaction_payload(&token_id, &swap_data).unwrap_err();

        assert_eq!(err.to_string(), "Invalid Aptos token ID format");
    }

    #[test]
    fn test_build_swap_transaction_payload_contract_invalid_data() {
        let swap_data = SwapQuoteData::new_transfer(TEST_POOL_ADDRESS.to_string(), BigUint::from(0u64), None);
        let swap_data = SwapQuoteData {
            data_type: SwapQuoteDataType::Contract,
            ..swap_data
        };

        assert!(build_swap_transaction_payload(&None, &swap_data).is_err());
    }
}
