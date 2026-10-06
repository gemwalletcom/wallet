pub(crate) mod hypercore;
pub(crate) mod sui;

use super::SwapType;
use primitives::Chain;

pub(crate) fn swap_type(from_chain: Chain) -> SwapType {
    match from_chain {
        Chain::HyperCore => SwapType::ExactInput,
        _ => SwapType::FlexInput,
    }
}

pub(crate) fn supports_destination(to_chain: Chain) -> bool {
    to_chain != Chain::HyperCore
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SwapAmountMode;

    #[test]
    fn test_swap_type() {
        assert_eq!(swap_type(Chain::HyperCore).amount_mode(), SwapAmountMode::Fixed);
        assert_eq!(swap_type(Chain::Ethereum).amount_mode(), SwapAmountMode::Flexible);
    }

    #[test]
    fn test_supports_destination() {
        assert!(!supports_destination(Chain::HyperCore));
        assert!(supports_destination(Chain::Ethereum));
    }
}
