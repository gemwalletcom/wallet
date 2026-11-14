use crate::{Chain, ChainType, DAY, MINUTE, WEEK};

const MIN_EVM_TRANSACTION_TIMEOUT: u32 = (MINUTE.as_millis() * 15) as u32;
const LIGHTNING_TRANSACTION_TIMEOUT: u32 = (DAY.as_millis() * 16) as u32;

pub fn chain_transaction_timeout(chain: Chain) -> u32 {
    match chain.chain_type() {
        ChainType::Bitcoin => 1_209_600_000,
        ChainType::Solana => chain.block_time() * 150,
        ChainType::Ethereum => (chain.block_time() * 120).max(MIN_EVM_TRANSACTION_TIMEOUT),
        ChainType::Lightning => LIGHTNING_TRANSACTION_TIMEOUT,
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
    let destination_timeout = u64::from(chain_transaction_timeout(destination_chain));
    ((source_timeout + destination_timeout) * 3).max(WEEK.as_millis() as u64)
}

#[cfg(test)]
mod tests {
    use super::{chain_transaction_timeout, swap_transaction_timeout};
    use crate::{Chain, WEEK};

    #[test]
    fn test_chain_transaction_timeout() {
        assert_eq!(chain_transaction_timeout(Chain::Bitcoin), 1_209_600_000);
        assert_eq!(chain_transaction_timeout(Chain::Solana), Chain::Solana.block_time() * 150);
        assert_eq!(chain_transaction_timeout(Chain::Ethereum), Chain::Ethereum.block_time() * 120);
        assert_eq!(chain_transaction_timeout(Chain::Cosmos), Chain::Cosmos.block_time() * 600);
    }

    #[test]
    fn test_swap_transaction_timeout() {
        assert_eq!(swap_transaction_timeout(Chain::Tron, Chain::Tron), WEEK.as_millis() as u64, "a same-chain swap through a cross-chain provider waits as long");
        assert_eq!(swap_transaction_timeout(Chain::Ethereum, Chain::Solana), WEEK.as_millis() as u64);
        assert_eq!(
            swap_transaction_timeout(Chain::Bitcoin, Chain::Ethereum),
            (u64::from(chain_transaction_timeout(Chain::Bitcoin)) + u64::from(chain_transaction_timeout(Chain::Ethereum))) * 3
        );
    }
}
