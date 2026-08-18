use num_traits::ToPrimitive;

use gem_evm::rpc::model::Log;

use crate::constants::{EVENT_CLAIM_REWARDS, EVENT_DELEGATE, EVENT_UNDELEGATE, EVENT_WITHDRAW, STAKING_CONTRACT};
use primitives::{Chain, Transaction as PrimitivesTransaction, TransactionType};

use gem_evm::rpc::parsers::staking::make_staking_transaction;
use gem_evm::rpc::parsers::{EVENT_WORD_SIZE, ParseContext, ProtocolParser, ethereum_value_from_log_data};

pub struct MonadStakingParser;

impl ProtocolParser for MonadStakingParser {
    fn matches(&self, context: &ParseContext<'_>) -> bool {
        if *context.chain != Chain::Monad {
            return false;
        }

        context.transaction.to.as_ref().is_some_and(|to| to.eq_ignore_ascii_case(STAKING_CONTRACT))
    }

    fn parse(&self, context: &ParseContext<'_>) -> Option<PrimitivesTransaction> {
        context.receipt.logs.iter().find_map(|log| Self::parse_log(context, log))
    }
}

impl MonadStakingParser {
    fn parse_log(context: &ParseContext<'_>, log: &Log) -> Option<PrimitivesTransaction> {
        if !log.address.eq_ignore_ascii_case(STAKING_CONTRACT) || log.topics.len() != 3 {
            return None;
        }

        let validator_id = ethereum_value_from_log_data(log.topics.get(1)?, 0, EVENT_WORD_SIZE)?.to_u64()?.to_string();

        match log.topics.first()?.as_str() {
            EVENT_DELEGATE => make_staking_transaction(
                context,
                &validator_id,
                TransactionType::StakeDelegate,
                ethereum_value_from_log_data(&log.data, 0, EVENT_WORD_SIZE)?,
            ),
            EVENT_UNDELEGATE => make_staking_transaction(
                context,
                &validator_id,
                TransactionType::StakeUndelegate,
                ethereum_value_from_log_data(&log.data, EVENT_WORD_SIZE, EVENT_WORD_SIZE * 2)?,
            ),
            EVENT_WITHDRAW => make_staking_transaction(
                context,
                &validator_id,
                TransactionType::StakeWithdraw,
                ethereum_value_from_log_data(&log.data, EVENT_WORD_SIZE, EVENT_WORD_SIZE * 2)?,
            ),
            EVENT_CLAIM_REWARDS => make_staking_transaction(
                context,
                &validator_id,
                TransactionType::StakeRewards,
                ethereum_value_from_log_data(&log.data, 0, EVENT_WORD_SIZE)?,
            ),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use gem_evm::rpc::model::{Transaction, TransactionReceipt};
    use gem_evm::rpc::parsers::testkit::map_transaction;
    use primitives::{Chain, TransactionType, testkit::json_rpc::load_json_rpc_result};

    use super::MonadStakingParser;
    use crate::constants::STAKING_CONTRACT;
    use crate::testkit::TEST_MONAD_ADDRESS;

    #[test]
    fn test_map_monad_staking_transactions() {
        let delegate_transaction = load_json_rpc_result::<Transaction>(include_str!("../testdata/monad/transaction_staking_delegate.json"));
        let delegate_receipt = load_json_rpc_result::<TransactionReceipt>(include_str!("../testdata/monad/transaction_staking_delegate_receipt.json"));
        let delegate = map_transaction(Box::new(MonadStakingParser), &Chain::Monad, &delegate_transaction, &delegate_receipt);
        assert_eq!(delegate.transaction_type, TransactionType::StakeDelegate);
        assert_eq!(delegate.from, TEST_MONAD_ADDRESS);
        assert_eq!(delegate.to, "5");
        assert_eq!(delegate.contract.as_deref(), Some(STAKING_CONTRACT));
        assert_eq!(delegate.value, "2000000000000000000");

        let undelegate_transaction = load_json_rpc_result::<Transaction>(include_str!("../testdata/monad/transaction_staking_undelegate.json"));
        let undelegate_receipt = load_json_rpc_result::<TransactionReceipt>(include_str!("../testdata/monad/transaction_staking_undelegate_receipt.json"));
        let undelegate = map_transaction(Box::new(MonadStakingParser), &Chain::Monad, &undelegate_transaction, &undelegate_receipt);
        assert_eq!(undelegate.transaction_type, TransactionType::StakeUndelegate);
        assert_eq!(undelegate.from, TEST_MONAD_ADDRESS);
        assert_eq!(undelegate.to, "10");
        assert_eq!(undelegate.contract.as_deref(), Some(STAKING_CONTRACT));
        assert_eq!(undelegate.value, "10000000000000000000");

        let claim_transaction = load_json_rpc_result::<Transaction>(include_str!("../testdata/monad/transaction_staking_claim_rewards.json"));
        let claim_receipt = load_json_rpc_result::<TransactionReceipt>(include_str!("../testdata/monad/transaction_staking_claim_rewards_receipt.json"));
        let claim = map_transaction(Box::new(MonadStakingParser), &Chain::Monad, &claim_transaction, &claim_receipt);
        assert_eq!(claim.transaction_type, TransactionType::StakeRewards);
        assert_eq!(claim.from, TEST_MONAD_ADDRESS);
        assert_eq!(claim.to, "10");
        assert_eq!(claim.contract.as_deref(), Some(STAKING_CONTRACT));
        assert_eq!(claim.value, "315193747607045635");

        let withdraw_transaction = load_json_rpc_result::<Transaction>(include_str!("../testdata/monad/transaction_staking_withdraw.json"));
        let withdraw_receipt = load_json_rpc_result::<TransactionReceipt>(include_str!("../testdata/monad/transaction_staking_withdraw_receipt.json"));
        let withdraw = map_transaction(Box::new(MonadStakingParser), &Chain::Monad, &withdraw_transaction, &withdraw_receipt);
        assert_eq!(withdraw.transaction_type, TransactionType::StakeWithdraw);
        assert_eq!(withdraw.from, TEST_MONAD_ADDRESS);
        assert_eq!(withdraw.to, "10");
        assert_eq!(withdraw.contract.as_deref(), Some(STAKING_CONTRACT));
        assert_eq!(withdraw.value, "10000521154972741508");
    }
}
