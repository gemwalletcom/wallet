use primitives::Chain;

use super::model::GemMemoWarning;
use crate::config::chain::is_memo_supported;

pub fn memo_warning(chain: Chain) -> GemMemoWarning {
    if !is_memo_supported(chain) {
        return GemMemoWarning::NotSupported;
    }
    match chain {
        Chain::Xrp => GemMemoWarning::DestinationTag,
        _ => GemMemoWarning::Memo,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memo_warning_names_the_field_each_chain_uses() {
        assert_eq!(memo_warning(Chain::Xrp), GemMemoWarning::DestinationTag);
        assert_eq!(memo_warning(Chain::Cosmos), GemMemoWarning::Memo);
        assert_eq!(memo_warning(Chain::Ton), GemMemoWarning::Memo);
        assert_eq!(memo_warning(Chain::Ethereum), GemMemoWarning::NotSupported);
        assert_eq!(memo_warning(Chain::Bitcoin), GemMemoWarning::NotSupported);
    }
}
