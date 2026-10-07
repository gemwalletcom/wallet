use alloy_primitives::{U256, hex};
use alloy_sol_types::SolValue;
use alloy_sol_types::sol;

sol! {
    interface IERC20 {
        function name() public view virtual returns (string memory);
        function symbol() public view virtual returns (string memory);
        function decimals() public view virtual returns (uint8);
        function allowance(address owner, address spender) external view returns (uint256);

        function transfer(address to, uint256 value) external returns (bool);
        function transferFrom(address from, address to, uint256 value) external returns (bool);
        function approve(address spender, uint256 value) external returns (bool);
    }
}

fn decode_abi_string(hex_data: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let bytes_data = hex::decode(hex_data)?;
    if bytes_data.is_empty() {
        return Ok(String::new());
    }
    String::abi_decode(&bytes_data).or_else(|_| String::from_utf8(bytes_data).map(|s| s.trim_matches('\0').to_string()).map_err(|utf8_error| utf8_error.to_string().into()))
}

fn decode_abi_uint8(hex_data: &str) -> Result<u8, Box<dyn std::error::Error + Send + Sync>> {
    if hex_data.is_empty() {
        return Ok(0);
    }

    let bytes_data = hex::decode(hex_data)?;
    let value_u256 = U256::abi_decode(&bytes_data)?;
    let value: u8 = value_u256.try_into()?;

    Ok(value)
}

pub fn decode_token_metadata(name_hex: &str, symbol_hex: &str, decimals_hex: &str) -> Result<(String, String, u8), Box<dyn std::error::Error + Send + Sync>> {
    let name = decode_abi_string(name_hex)?;
    let symbol = decode_abi_string(symbol_hex)?;
    let decimals = decode_abi_uint8(decimals_hex)?;
    if symbol.is_empty() {
        return Err("Invalid token metadata: symbol is empty".into());
    }
    let name = if name.is_empty() { symbol.clone() } else { name };
    Ok((name, symbol, decimals))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_abi_string() {
        let bytes32_data = "0x4d616b6572000000000000000000000000000000000000000000000000000000";
        let result = decode_abi_string(bytes32_data).unwrap();

        assert_eq!(result, "Maker");

        let string_data = "0x0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000a5465746865722055534400000000000000000000000000000000000000000000";
        let result = decode_abi_string(string_data).unwrap();

        assert_eq!(result, "Tether USD");
    }
}
