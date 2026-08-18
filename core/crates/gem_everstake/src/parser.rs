use gem_evm::ethereum_address_checksum;
use gem_evm::rpc::model::Log;
use primitives::{Chain, Transaction as PrimitivesTransaction, TransactionType};

use crate::constants::{EVERSTAKE_ACCOUNTING_ADDRESS, EVERSTAKE_POOL_ADDRESS};

use gem_evm::rpc::parsers::staking::make_staking_transaction;
use gem_evm::rpc::parsers::{EVENT_WORD_SIZE, ParseContext, ProtocolParser, ethereum_value_from_log_data};

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
                make_staking_transaction(context, &pool_address, TransactionType::StakeWithdraw, value)
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use gem_evm::rpc::model::{Transaction, TransactionReceipt};
    use gem_evm::rpc::parsers::testkit::map_transaction;
    use primitives::{Chain, TransactionType, testkit::json_rpc::load_json_rpc_result};

    use super::EverstakeParser;
    use crate::constants::{EVERSTAKE_ACCOUNTING_ADDRESS, EVERSTAKE_POOL_ADDRESS};

    #[test]
    fn test_map_everstake_transactions() {
        let stake_transaction = load_json_rpc_result::<Transaction>(include_str!("../testdata/everstake/transaction_stake.json"));
        let stake_receipt = load_json_rpc_result::<TransactionReceipt>(include_str!("../testdata/everstake/transaction_stake_receipt.json"));
        let stake = map_transaction(Box::new(EverstakeParser), &Chain::Ethereum, &stake_transaction, &stake_receipt);
        assert_eq!(stake.transaction_type, TransactionType::StakeDelegate);
        assert_eq!(stake.from, "0x0D9DAB1A248f63B0a48965bA8435e4de7497a3dC");
        assert_eq!(stake.to, EVERSTAKE_POOL_ADDRESS);
        assert_eq!(stake.contract.as_deref(), Some(EVERSTAKE_POOL_ADDRESS));
        assert_eq!(stake.value, "34800000000000000000");

        let unstake_transaction = load_json_rpc_result::<Transaction>(include_str!("../testdata/everstake/transaction_unstake.json"));
        let unstake_receipt = load_json_rpc_result::<TransactionReceipt>(include_str!("../testdata/everstake/transaction_unstake_receipt.json"));
        let unstake = map_transaction(Box::new(EverstakeParser), &Chain::Ethereum, &unstake_transaction, &unstake_receipt);
        assert_eq!(unstake.transaction_type, TransactionType::StakeUndelegate);
        assert_eq!(unstake.from, "0x1085c5f70F7F7591D97da281A64688385455c2bD");
        assert_eq!(unstake.to, EVERSTAKE_POOL_ADDRESS);
        assert_eq!(unstake.contract.as_deref(), Some(EVERSTAKE_POOL_ADDRESS));
        assert_eq!(unstake.value, "50000000000000000");

        let withdraw_transaction = load_json_rpc_result::<Transaction>(include_str!("../testdata/everstake/transaction_withdraw.json"));
        let withdraw_receipt = load_json_rpc_result::<TransactionReceipt>(include_str!("../testdata/everstake/transaction_withdraw_receipt.json"));
        let withdraw = map_transaction(Box::new(EverstakeParser), &Chain::Ethereum, &withdraw_transaction, &withdraw_receipt);
        assert_eq!(withdraw.transaction_type, TransactionType::StakeWithdraw);
        assert_eq!(withdraw.from, "0x1085c5f70F7F7591D97da281A64688385455c2bD");
        assert_eq!(withdraw.to, EVERSTAKE_POOL_ADDRESS);
        assert_eq!(withdraw.contract.as_deref(), Some(EVERSTAKE_ACCOUNTING_ADDRESS));
        assert_eq!(withdraw.value, "50000000000000000");
    }
}
