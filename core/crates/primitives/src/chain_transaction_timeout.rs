use crate::{Chain, ChainType, DAY};

pub fn chain_transaction_timeout(chain: Chain) -> u32 {
    match chain.chain_type() {
        ChainType::Bitcoin => 1_209_600_000,
        ChainType::Solana => chain.block_time() * 150,
        ChainType::Ethereum => chain.block_time() * 120,
        ChainType::Cosmos
        | ChainType::Ton
        | ChainType::Tron
        | ChainType::Aptos
        | ChainType::Sui
        | ChainType::Xrp
        | ChainType::Near
        | ChainType::Stellar
        | ChainType::Algorand
        | ChainType::Polkadot
        | ChainType::Cardano
        | ChainType::HyperCore => chain.block_time() * 600,
    }
}

pub fn swap_transaction_timeout(source_chain: Chain, destination_chain: Chain) -> u64 {
    let source_timeout = u64::from(chain_transaction_timeout(source_chain));
    if source_chain == destination_chain {
        return source_timeout;
    }

    let destination_timeout = u64::from(chain_transaction_timeout(destination_chain));
    ((source_timeout + destination_timeout) * 3).max(DAY.as_millis() as u64)
}

#[cfg(test)]
mod tests {
    use super::{chain_transaction_timeout, swap_transaction_timeout};
    use crate::{Chain, DAY};

    #[test]
    fn test_chain_transaction_timeout() {
        assert_eq!(chain_transaction_timeout(Chain::Bitcoin), 1_209_600_000);
        assert_eq!(chain_transaction_timeout(Chain::Solana), Chain::Solana.block_time() * 150);
        assert_eq!(chain_transaction_timeout(Chain::Ethereum), Chain::Ethereum.block_time() * 120);
        assert_eq!(chain_transaction_timeout(Chain::Cosmos), Chain::Cosmos.block_time() * 600);
    }

    #[test]
    fn test_swap_transaction_timeout() {
        assert_eq!(
            swap_transaction_timeout(Chain::Ethereum, Chain::Ethereum),
            u64::from(chain_transaction_timeout(Chain::Ethereum))
        );
        assert_eq!(swap_transaction_timeout(Chain::Ethereum, Chain::Solana), DAY.as_millis() as u64);
        assert_eq!(
            swap_transaction_timeout(Chain::Bitcoin, Chain::Ethereum),
            (u64::from(chain_transaction_timeout(Chain::Bitcoin)) + u64::from(chain_transaction_timeout(Chain::Ethereum))) * 3
        );
    }
}
