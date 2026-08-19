use alloy_primitives::Address;
use alloy_sol_types::SolCall;
use num_bigint::BigInt;
use primitives::TransactionInputType;

use crate::contracts::ITempoFeeManager;
use gem_evm::ethereum_address_checksum;

pub(crate) const FEE_MANAGER_ADDRESS: &str = "0xfeEC000000000000000000000000000000000000";
#[cfg(feature = "rpc")]
pub(crate) const USD_CURRENCY: &str = "USD";
const FEE_SCALE: u64 = 1_000_000_000_000;

pub(crate) fn decode_set_user_fee_token(input_type: &TransactionInputType) -> Option<Address> {
    let TransactionInputType::Generic(_, _, extra) = input_type else {
        return None;
    };
    if ethereum_address_checksum(&extra.to).ok().as_deref() != Some(FEE_MANAGER_ADDRESS) {
        return None;
    }
    ITempoFeeManager::setUserTokenCall::abi_decode(extra.data.as_deref()?).ok().map(|call| call.token)
}

pub(crate) fn scale_fee_to_token_units(fee: BigInt) -> BigInt {
    let scale = BigInt::from(FEE_SCALE);
    (fee + &scale - BigInt::from(1)) / scale
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::mock_tempo_generic_input;
    use primitives::{Asset, Chain, asset_constants::TEMPO_USDC_TOKEN_ID, hex};

    #[test]
    fn test_decode_set_user_fee_token() {
        let calldata = hex::decode_hex("0xe789744400000000000000000000000020c00000000000000000000014f22ca97301eb73").unwrap();
        let token = TEMPO_USDC_TOKEN_ID.parse::<Address>().unwrap();

        assert_eq!(decode_set_user_fee_token(&mock_tempo_generic_input(FEE_MANAGER_ADDRESS, calldata.clone())), Some(token));
        assert_eq!(
            decode_set_user_fee_token(&mock_tempo_generic_input("0x0000000000000000000000000000000000000001", calldata)),
            None
        );
        assert_eq!(decode_set_user_fee_token(&mock_tempo_generic_input(FEE_MANAGER_ADDRESS, vec![0xab, 0xcd])), None);
        assert_eq!(decode_set_user_fee_token(&TransactionInputType::Transfer(Asset::mock_with_chain(Chain::Tempo))), None);
    }

    #[test]
    fn test_scale_fee_to_token_units() {
        assert_eq!(scale_fee_to_token_units(BigInt::from(420_000_000_000_000u64)), BigInt::from(420u64));
        assert_eq!(scale_fee_to_token_units(BigInt::from(420_000_000_000_001u64)), BigInt::from(421u64));
    }
}
