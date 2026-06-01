use alloy_primitives::{Address, U256};
use alloy_sol_types::SolCall;
use num_bigint::BigInt;
use std::error::Error;
use std::str::FromStr;

use crate::contracts::{IERC20, IERC721, IERC1155};

pub fn encode_erc20_transfer(to: &str, amount: &BigInt) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
    Ok(IERC20::transferCall {
        to: Address::from_str(to)?,
        value: U256::from_str(&amount.to_string())?,
    }
    .abi_encode())
}

pub fn encode_erc20_approve_max_value(spender: &str) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
    Ok(IERC20::approveCall {
        spender: Address::from_str(spender)?,
        value: U256::MAX,
    }
    .abi_encode())
}

pub fn encode_erc721_transfer(from: &str, to: &str, token_id: &str) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
    Ok(IERC721::safeTransferFromCall {
        from: Address::from_str(from)?,
        to: Address::from_str(to)?,
        tokenId: U256::from_str(token_id)?,
    }
    .abi_encode())
}

pub fn encode_erc1155_transfer(from: &str, to: &str, token_id: &str) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
    Ok(IERC1155::safeTransferFromCall {
        from: Address::from_str(from)?,
        to: Address::from_str(to)?,
        id: U256::from_str(token_id)?,
        amount: U256::from(1),
        data: vec![].into(),
    }
    .abi_encode())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_erc20_approve_max_value() {
        let spender = "0x2b5AD5c4795c026514f8317c7a215E218dccD6cF";
        let data = encode_erc20_approve_max_value(spender).unwrap();
        let call = IERC20::approveCall::abi_decode(&data).unwrap();

        assert_eq!(call.spender, Address::from_str(spender).unwrap());
        assert_eq!(call.value, U256::MAX);
    }
}
