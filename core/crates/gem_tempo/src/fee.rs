use alloy_primitives::Address;
use alloy_sol_types::SolCall;
use num_bigint::BigInt;
use primitives::{AssetId, Chain, EVMChain, TransactionInputType};

use crate::contracts::ITempoFeeManager;
use gem_evm::ethereum_address_checksum;
#[cfg(feature = "rpc")]
use gem_evm::rpc::model::TransactionReceipt;

pub const FEE_MANAGER_ADDRESS: &str = "0xfeEC000000000000000000000000000000000000";
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

/// The tokenized-native (pathUSD) contract, sourced from the chain config.
pub fn native_token_contract() -> &'static str {
    EVMChain::Tempo.weth_contract().expect("Tempo chain config defines the tokenized-native contract")
}

pub fn is_native_token_contract(contract_address: &str) -> bool {
    ethereum_address_checksum(contract_address).is_ok_and(|address| address == native_token_contract())
}

#[cfg(feature = "rpc")]
pub fn map_transaction_fee(receipt: &TransactionReceipt) -> (String, AssetId) {
    let fee = scale_fee_to_token_units(receipt.get_fee().into()).to_string();
    (fee, map_fee_asset_id(receipt.fee_token.as_deref()))
}

pub fn map_fee_asset_id(fee_token: Option<&str>) -> AssetId {
    match fee_token.and_then(|fee_token| ethereum_address_checksum(fee_token).ok()) {
        Some(fee_token) if !is_native_token_contract(&fee_token) => AssetId::from_token(Chain::Tempo, &fee_token),
        _ => AssetId::from_chain(Chain::Tempo),
    }
}

pub fn map_asset_id(asset_id: AssetId) -> AssetId {
    match asset_id.token_id.as_deref() {
        Some(contract_address) if is_native_token_contract(contract_address) => AssetId::from_chain(asset_id.chain),
        _ => asset_id,
    }
}

pub(crate) fn scale_fee_to_token_units(fee: BigInt) -> BigInt {
    let scale = BigInt::from(FEE_SCALE);
    (fee + &scale - BigInt::from(1)) / scale
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::{TEMPO_TEST_USER_FEE_TOKEN, mock_tempo_generic_input};
    use primitives::hex;

    #[test]
    fn test_decode_set_user_fee_token() {
        let calldata = hex::decode_hex("0xe789744400000000000000000000000020c00000000000000000000014f22ca97301eb73").unwrap();
        let token = TEMPO_TEST_USER_FEE_TOKEN.parse::<Address>().unwrap();

        assert_eq!(decode_set_user_fee_token(&mock_tempo_generic_input(FEE_MANAGER_ADDRESS, calldata.clone())), Some(token));
        assert_eq!(
            decode_set_user_fee_token(&mock_tempo_generic_input("0x0000000000000000000000000000000000000001", calldata)),
            None
        );
        assert_eq!(decode_set_user_fee_token(&mock_tempo_generic_input(FEE_MANAGER_ADDRESS, vec![0xab, 0xcd])), None);
        assert_eq!(
            decode_set_user_fee_token(&TransactionInputType::Transfer(primitives::Asset::from_chain(Chain::Tempo))),
            None
        );
    }

    #[test]
    fn test_scale_fee_to_token_units() {
        assert_eq!(scale_fee_to_token_units(BigInt::from(420_000_000_000_000u64)), BigInt::from(420u64));
        assert_eq!(scale_fee_to_token_units(BigInt::from(420_000_000_000_001u64)), BigInt::from(421u64));
    }
}
