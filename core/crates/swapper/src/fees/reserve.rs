use num_bigint::BigUint;
use primitives::{Chain, ChainType};

pub fn reserved_transaction_fees(chain: Chain) -> Option<BigUint> {
    let reserve: u64 = match chain {
        Chain::Ethereum => 1_000_000_000_000_000,
        Chain::Arbitrum => 300_000_000_000_000,
        Chain::Base => 300_000_000_000_000,
        Chain::Optimism => 500_000_000_000_000,
        Chain::AvalancheC => 3_000_000_000_000_000,
        Chain::SmartChain => 2_000_000_000_000_000,
        Chain::Polygon => 20_000_000_000_000_000,
        Chain::Gnosis => 5_000_000_000_000_000,
        Chain::Berachain => 5_000_000_000_000_000,
        Chain::Sui => 50_000_000,
        Chain::Solana => 5_000_000,
        Chain::Tron => 2_000_000,
        Chain::Ton => 20_000_000,
        Chain::Aptos => 20_000_000,
        Chain::Stellar => 5_500_000,
        Chain::Monad => 5_000_000_000_000_000,
        Chain::XLayer => 5_000_000_000_000_000,
        Chain::Plasma => 5_000_000_000_000_000,
        Chain::OpBNB => 100_000_000_000_000,
        Chain::Mantle => 50_000_000_000_000_000,
        Chain::Linea => 300_000_000_000_000,
        Chain::Celo => 10_000_000_000_000_000,
        Chain::Sonic => 50_000_000_000_000_000,
        Chain::Robinhood => 100_000_000_000_000,
        Chain::Hyperliquid => 1_000_000_000_000_000,
        Chain::Stable => 10_000_000_000_000_000,
        Chain::Arc => 50_000_000_000_000_000,
        _ => return None,
    };
    Some(BigUint::from(reserve))
}

pub fn max_amount_spends_all_but_fee(chain: Chain) -> bool {
    match chain.chain_type() {
        ChainType::Ethereum => true,
        ChainType::Bitcoin
        | ChainType::Solana
        | ChainType::Cosmos
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
        | ChainType::HyperCore => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solana_reserve_covers_rent_and_priority_fees() {
        let reserve = reserved_transaction_fees(Chain::Solana).unwrap();
        assert!(reserve >= BigUint::from(2_500_000u64), "Solana reserve {reserve} too small to cover wSOL ATA rent plus priority fees");
    }

    #[test]
    fn stellar_reserve_covers_a_new_trustline_and_the_swap_fee() {
        let reserve = reserved_transaction_fees(Chain::Stellar).unwrap();
        assert!(
            reserve >= BigUint::from(5_000_000u64 + 3 * 2 * 18_888),
            "Stellar reserve {reserve} too small for a trustline's 0.5 XLM base reserve plus three operations at twice the p99 fee"
        );
    }

    #[test]
    fn tron_reserve_covers_native_chainflip_memo_transfer() {
        let reserve = reserved_transaction_fees(Chain::Tron).unwrap();
        assert!(reserve >= BigUint::from(1_398_000u64 * 120 / 100));
    }
}
