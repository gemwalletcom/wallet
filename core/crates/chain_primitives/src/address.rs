use primitives::{Chain, ChainType};

pub fn checksum_address(address: &str, chain: Chain) -> String {
    let address = address.trim();
    match chain.chain_type() {
        ChainType::Ethereum | ChainType::HyperCore => gem_evm::ethereum_address_checksum(address).unwrap_or_else(|_| address.to_string()),
        _ => address.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checksum_address() {
        assert_eq!(checksum_address("0x5615e8ab93b9d695b6d4d6545f7792aa59e1069a", Chain::Ethereum), "0x5615E8AB93b9d695b6d4d6545f7792aA59e1069a");
        assert_eq!(checksum_address(" \n0x5615e8ab93b9d695b6d4d6545f7792aa59e1069a\r ", Chain::Ethereum), "0x5615E8AB93b9d695b6d4d6545f7792aA59e1069a");
        assert_eq!(checksum_address(" \ngemcoder.eth\r ", Chain::Ethereum), "gemcoder.eth");
        assert_eq!(checksum_address("invalid", Chain::Ethereum), "invalid");
        assert_eq!(checksum_address(" \nGvhwZwtV32kYUXUw965CUM3KGPdtBsDwPVpi92brY5R2\r ", Chain::Solana), "GvhwZwtV32kYUXUw965CUM3KGPdtBsDwPVpi92brY5R2");
    }
}
