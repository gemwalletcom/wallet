use crate::{
    ethereum_address_checksum,
    everstake::{EVERSTAKE_ACCOUNTING_ADDRESS, EVERSTAKE_POOL_ADDRESS},
    rpc::{balance_differ::BalanceDiffer, model::Log},
};
use primitives::{Chain, Transaction as PrimitivesTransaction, TransactionType};

use super::{EVENT_WORD_SIZE, ParseContext, ProtocolParser, ethereum_value_from_log_data, make_staking_transaction};

const EVENT_STAKED: &str = "0x7d194e8dc0f902cdc51bde00649039561dbd0b01574d671bad333436fdac7692";
const EVENT_UNSTAKED: &str = "0x0750a71dce555de583ab0225a108df42b9402d22123d7cc9cd95793e43e7db0e";
const EVENT_WITHDRAWN: &str = "0x262159451c4018521811107ecbe27e3de7d95a70a4a534f733aa59bc4346f03e";

pub struct EverstakeParser;

impl ProtocolParser for EverstakeParser {
    fn matches(&self, context: &ParseContext<'_>) -> bool {
        if *context.chain != Chain::Ethereum {
            return false;
        }

        context
            .transaction
            .to
            .as_ref()
            .is_some_and(|to| to.eq_ignore_ascii_case(EVERSTAKE_POOL_ADDRESS) || to.eq_ignore_ascii_case(EVERSTAKE_ACCOUNTING_ADDRESS))
    }

    fn parse(&self, context: &ParseContext<'_>) -> Option<PrimitivesTransaction> {
        context.receipt.logs.iter().find_map(|log| Self::parse_log(context, log))
    }
}

impl EverstakeParser {
    fn parse_log(context: &ParseContext<'_>, log: &Log) -> Option<PrimitivesTransaction> {
        if log.topics.len() != 2 {
            return None;
        }

        let value = ethereum_value_from_log_data(&log.data, 0, EVENT_WORD_SIZE)?;
        let pool_address = ethereum_address_checksum(EVERSTAKE_POOL_ADDRESS).ok()?;
        match log.topics.first()?.as_str() {
            EVENT_STAKED if log.address.eq_ignore_ascii_case(EVERSTAKE_POOL_ADDRESS) => make_staking_transaction(context, &pool_address, TransactionType::StakeDelegate, value),
            EVENT_UNSTAKED if log.address.eq_ignore_ascii_case(EVERSTAKE_POOL_ADDRESS) => make_staking_transaction(context, &pool_address, TransactionType::StakeUndelegate, value),
            EVENT_WITHDRAWN if log.address.eq_ignore_ascii_case(EVERSTAKE_ACCOUNTING_ADDRESS) => {
                let value = context
                    .trace
                    .and_then(|trace| BalanceDiffer::new(*context.chain).get_native_balance_change(trace, context.receipt, &context.transaction.from))
                    .unwrap_or(value);

                make_staking_transaction(context, &pool_address, TransactionType::StakeWithdraw, value)
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use primitives::{Chain, TransactionType, testkit::json_rpc::load_json_rpc_result};

    use crate::rpc::model::{Transaction, TransactionReceipt, TransactionReplayTrace};

    use super::super::{assert_staking_transaction, map_transaction};

    #[test]
    fn test_map_everstake_transactions() {
        let stake_transaction = load_json_rpc_result::<Transaction>(include_str!("../../../../testdata/everstake/transaction_stake.json"));
        let stake_receipt = load_json_rpc_result::<TransactionReceipt>(include_str!("../../../../testdata/everstake/transaction_stake_receipt.json"));
        let stake = map_transaction(&Chain::Ethereum, &stake_transaction, &stake_receipt, None);
        assert_staking_transaction(
            &stake,
            Chain::Ethereum,
            TransactionType::StakeDelegate,
            "0x0D9DAB1A248f63B0a48965bA8435e4de7497a3dC",
            "0xD523794C879D9eC028960a231F866758e405bE34",
            "0xD523794C879D9eC028960a231F866758e405bE34",
            "34800000000000000000",
        );

        let unstake_transaction = load_json_rpc_result::<Transaction>(include_str!("../../../../testdata/everstake/transaction_unstake.json"));
        let unstake_receipt = load_json_rpc_result::<TransactionReceipt>(include_str!("../../../../testdata/everstake/transaction_unstake_receipt.json"));
        let unstake = map_transaction(&Chain::Ethereum, &unstake_transaction, &unstake_receipt, None);
        assert_staking_transaction(
            &unstake,
            Chain::Ethereum,
            TransactionType::StakeUndelegate,
            "0x1085c5f70F7F7591D97da281A64688385455c2bD",
            "0xD523794C879D9eC028960a231F866758e405bE34",
            "0xD523794C879D9eC028960a231F866758e405bE34",
            "50000000000000000",
        );

        let withdraw_transaction = load_json_rpc_result::<Transaction>(include_str!("../../../../testdata/everstake/transaction_withdraw.json"));
        let withdraw_receipt = load_json_rpc_result::<TransactionReceipt>(include_str!("../../../../testdata/everstake/transaction_withdraw_receipt.json"));
        let withdraw_trace = load_json_rpc_result::<TransactionReplayTrace>(include_str!("../../../../testdata/everstake/transaction_withdraw_trace.json"));
        let withdraw = map_transaction(&Chain::Ethereum, &withdraw_transaction, &withdraw_receipt, Some(&withdraw_trace));
        assert_staking_transaction(
            &withdraw,
            Chain::Ethereum,
            TransactionType::StakeWithdraw,
            "0x1085c5f70F7F7591D97da281A64688385455c2bD",
            "0xD523794C879D9eC028960a231F866758e405bE34",
            "0x7a7f0b3c23C23a31cFcb0c44709be70d4D545c6e",
            "50000000000000000",
        );
    }
}
