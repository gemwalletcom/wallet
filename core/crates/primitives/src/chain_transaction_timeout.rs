use std::time::Duration;

use crate::{Chain, ChainType, DurationExt, WEEK};

const MIN_TRANSACTION_TIMEOUT: Duration = Duration::from_mins(15);
const NONCE_HOLD_BLOCKS: u32 = 5;
const MIN_NONCE_HOLD: Duration = Duration::from_mins(1);

fn block_time(chain: Chain) -> Duration {
    Duration::from_millis(u64::from(chain.block_time()))
}

pub fn chain_transaction_timeout(chain: Chain) -> Duration {
    let block_time = block_time(chain);
    match chain.chain_type() {
        ChainType::Bitcoin => Duration::days(14),
        ChainType::Solana => block_time * 150,
        ChainType::Ethereum => (block_time * 120).max(MIN_TRANSACTION_TIMEOUT),
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
        | ChainType::HyperCore => (block_time * 600).max(MIN_TRANSACTION_TIMEOUT),
    }
}

pub fn chain_nonce_hold(chain: Chain) -> Duration {
    (block_time(chain) * NONCE_HOLD_BLOCKS).max(MIN_NONCE_HOLD)
}

pub fn swap_transaction_timeout(source_chain: Chain, destination_chain: Chain) -> Duration {
    ((chain_transaction_timeout(source_chain) + chain_transaction_timeout(destination_chain)) * 3).max(WEEK)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_transaction_timeout() {
        assert_eq!(chain_transaction_timeout(Chain::Bitcoin), Duration::days(14));
        assert_eq!(chain_transaction_timeout(Chain::Solana), block_time(Chain::Solana) * 150);
        assert_eq!(chain_transaction_timeout(Chain::Ethereum), block_time(Chain::Ethereum) * 120);
        assert_eq!(chain_transaction_timeout(Chain::Cosmos), block_time(Chain::Cosmos) * 600);
        assert_eq!(chain_transaction_timeout(Chain::Ton), MIN_TRANSACTION_TIMEOUT, "600 fast blocks are too short a wait for a send");
    }

    #[test]
    fn test_chain_nonce_hold() {
        assert_eq!(chain_nonce_hold(Chain::Ethereum), Duration::from_secs(60), "five Ethereum blocks");
        assert_eq!(chain_nonce_hold(Chain::Arbitrum), MIN_NONCE_HOLD, "five fast blocks would replace a send before it can land");
        assert_eq!(chain_nonce_hold(Chain::Bitcoin), block_time(Chain::Bitcoin) * NONCE_HOLD_BLOCKS, "slow blocks hold longer than the minimum");
    }

    #[test]
    fn test_swap_transaction_timeout() {
        assert_eq!(swap_transaction_timeout(Chain::Tron, Chain::Tron), WEEK, "a same-chain swap through a cross-chain provider waits as long");
        assert_eq!(swap_transaction_timeout(Chain::Ethereum, Chain::Solana), WEEK);
        assert_eq!(
            swap_transaction_timeout(Chain::Bitcoin, Chain::Ethereum),
            (chain_transaction_timeout(Chain::Bitcoin) + chain_transaction_timeout(Chain::Ethereum)) * 3
        );
    }
}
