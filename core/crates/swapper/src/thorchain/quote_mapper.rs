use primitives::Chain;

const OUTBOUND_DELAY_SECONDS: u32 = 60;

pub fn map_eta_in_seconds(destination_chain: Chain, total_swap_seconds: Option<u32>) -> u32 {
    destination_chain.block_time() / 1000 + OUTBOUND_DELAY_SECONDS + total_swap_seconds.unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_eta_in_seconds() {
        assert_eq!(map_eta_in_seconds(Chain::Bitcoin, None), 660);
        assert_eq!(map_eta_in_seconds(Chain::Bitcoin, Some(1200)), 1860);
        assert_eq!(map_eta_in_seconds(Chain::SmartChain, Some(648)), 709);
    }
}
