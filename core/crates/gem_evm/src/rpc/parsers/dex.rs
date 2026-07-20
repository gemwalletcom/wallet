use std::str::FromStr;

use alloy_primitives::Address;
use primitives::Transaction as PrimitivesTransaction;

use super::{ParseContext, ProtocolParser};

pub struct DexSwapParser;

impl ProtocolParser for DexSwapParser {
    fn matches(&self, context: &ParseContext<'_>) -> bool {
        let Some(registry) = context.contract_registry else {
            return false;
        };
        context
            .transaction
            .to
            .as_ref()
            .is_some_and(|to| Address::from_str(to).ok().and_then(|addr| registry.get_by_address(&addr, *context.chain)).is_some())
    }

    fn parse(&self, context: &ParseContext<'_>) -> Option<PrimitivesTransaction> {
        let registry = context.contract_registry?;
        let to = Address::from_str(context.transaction.to.as_ref()?).ok()?;
        let entry = registry.get_by_address(&to, *context.chain)?;

        let metadata = context.try_map_balance_diff_swap(&context.transaction.from, Some(entry.provider.to_string()))?;
        context.make_swap_transaction(&context.transaction.from, &context.transaction.from, &metadata)
    }
}
