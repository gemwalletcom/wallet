use num_traits::ToPrimitive;

use crate::{
    monad::{EVENT_CLAIM_REWARDS, EVENT_DELEGATE, EVENT_UNDELEGATE, EVENT_WITHDRAW, STAKING_CONTRACT},
    rpc::model::Log,
};
use primitives::{Chain, Transaction as PrimitivesTransaction, TransactionType};

use super::{EVENT_WORD_SIZE, ParseContext, ProtocolParser, ethereum_value_from_log_data, make_staking_transaction};

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
    use primitives::{Chain, TransactionType, testkit::json_rpc::load_json_rpc_result};

    use crate::rpc::model::{Transaction, TransactionReciept};

    use super::super::{assert_staking_transaction, map_transaction};

    #[test]
    fn test_map_monad_staking_transactions() {
        let delegate_transaction = load_json_rpc_result::<Transaction>(include_str!("../../../../testdata/monad/transaction_staking_delegate.json"));
        let delegate_receipt = load_json_rpc_result::<TransactionReciept>(include_str!("../../../../testdata/monad/transaction_staking_delegate_receipt.json"));
        let delegate = map_transaction(&Chain::Monad, &delegate_transaction, &delegate_receipt, None);
        assert_staking_transaction(
            &delegate,
            Chain::Monad,
            TransactionType::StakeDelegate,
            "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7",
            "5",
            "0x0000000000000000000000000000000000001000",
            "2000000000000000000",
        );

        let undelegate_transaction = load_json_rpc_result::<Transaction>(include_str!("../../../../testdata/monad/transaction_staking_undelegate.json"));
        let undelegate_receipt = load_json_rpc_result::<TransactionReciept>(include_str!("../../../../testdata/monad/transaction_staking_undelegate_receipt.json"));
        let undelegate = map_transaction(&Chain::Monad, &undelegate_transaction, &undelegate_receipt, None);
        assert_staking_transaction(
            &undelegate,
            Chain::Monad,
            TransactionType::StakeUndelegate,
            "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7",
            "10",
            "0x0000000000000000000000000000000000001000",
            "10000000000000000000",
        );

        let claim_transaction = load_json_rpc_result::<Transaction>(include_str!("../../../../testdata/monad/transaction_staking_claim_rewards.json"));
        let claim_receipt = load_json_rpc_result::<TransactionReciept>(include_str!("../../../../testdata/monad/transaction_staking_claim_rewards_receipt.json"));
        let claim = map_transaction(&Chain::Monad, &claim_transaction, &claim_receipt, None);
        assert_staking_transaction(
            &claim,
            Chain::Monad,
            TransactionType::StakeRewards,
            "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7",
            "10",
            "0x0000000000000000000000000000000000001000",
            "315193747607045635",
        );

        let withdraw_transaction = load_json_rpc_result::<Transaction>(include_str!("../../../../testdata/monad/transaction_staking_withdraw.json"));
        let withdraw_receipt = load_json_rpc_result::<TransactionReciept>(include_str!("../../../../testdata/monad/transaction_staking_withdraw_receipt.json"));
        let withdraw = map_transaction(&Chain::Monad, &withdraw_transaction, &withdraw_receipt, None);
        assert_staking_transaction(
            &withdraw,
            Chain::Monad,
            TransactionType::StakeWithdraw,
            "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7",
            "10",
            "0x0000000000000000000000000000000000001000",
            "10000521154972741508",
        );
    }
}
