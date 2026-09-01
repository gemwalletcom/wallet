use gem_evm::rpc::model::Log;
use gem_evm::rpc::parsers::staking::make_staking_transaction;
use gem_evm::rpc::parsers::{EVENT_WORD_SIZE, ParseContext, ProtocolParser, ethereum_value_from_log_data};
use num_traits::ToPrimitive;
use primitives::{Chain, Transaction as PrimitivesTransaction, TransactionType};

use crate::constants::{EVENT_CLAIM_REWARDS, EVENT_DELEGATE, EVENT_UNDELEGATE, EVENT_WITHDRAW, STAKING_CONTRACT};

pub struct MonadParser;

impl ProtocolParser for MonadParser {
    fn matches(&self, context: &ParseContext<'_>) -> bool {
        *context.chain == Chain::Monad && context.is_to(STAKING_CONTRACT)
    }

    fn parse(&self, context: &ParseContext<'_>) -> Option<PrimitivesTransaction> {
        context.receipt.logs.iter().find_map(|log| Self::parse_log(context, log))
    }
}

impl MonadParser {
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
    use chrono::DateTime;
    use gem_evm::rpc::parsers::ProtocolParsers;
    use primitives::{AssetId, Chain, TransactionState, TransactionType, testkit::json_rpc::load_json_rpc_result};

    use super::MonadParser;
    use crate::constants::STAKING_CONTRACT;
    use crate::testkit::TEST_ADDRESS;

    #[test]
    fn test_map_monad_staking_transactions() {
        let cases = [
            (
                include_str!("../testdata/transaction_staking_delegate.json"),
                include_str!("../testdata/transaction_staking_delegate_receipt.json"),
                TransactionType::StakeDelegate,
                "5",
                "2000000000000000000",
            ),
            (
                include_str!("../testdata/transaction_staking_undelegate.json"),
                include_str!("../testdata/transaction_staking_undelegate_receipt.json"),
                TransactionType::StakeUndelegate,
                "10",
                "10000000000000000000",
            ),
            (
                include_str!("../testdata/transaction_staking_claim_rewards.json"),
                include_str!("../testdata/transaction_staking_claim_rewards_receipt.json"),
                TransactionType::StakeRewards,
                "10",
                "315193747607045635",
            ),
            (
                include_str!("../testdata/transaction_staking_withdraw.json"),
                include_str!("../testdata/transaction_staking_withdraw_receipt.json"),
                TransactionType::StakeWithdraw,
                "10",
                "10000521154972741508",
            ),
        ];

        for (transaction, receipt, transaction_type, to, value) in cases {
            let staking_transaction = ProtocolParsers::map_transaction_with_parsers(
                &Chain::Monad,
                &load_json_rpc_result(transaction),
                &load_json_rpc_result(receipt),
                DateTime::default(),
                &[&MonadParser],
            )
            .unwrap();
            assert_eq!(staking_transaction.transaction_type, transaction_type);
            assert_eq!(staking_transaction.state, TransactionState::Confirmed);
            assert_eq!(staking_transaction.asset_id, AssetId::from_chain(Chain::Monad));
            assert_eq!(staking_transaction.fee_asset_id, AssetId::from_chain(Chain::Monad));
            assert_eq!(staking_transaction.from, TEST_ADDRESS);
            assert_eq!(staking_transaction.to, to);
            assert_eq!(staking_transaction.contract.as_deref(), Some(STAKING_CONTRACT));
            assert_eq!(staking_transaction.value.to_string(), value);
            assert_eq!(staking_transaction.metadata, None);
        }
    }
}
