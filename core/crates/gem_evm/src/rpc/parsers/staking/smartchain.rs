use crate::{address::ethereum_address_from_topic, rpc::model::Log};
use gem_bsc::stake_hub;
use primitives::{Chain, Transaction as PrimitivesTransaction, TransactionType};

use super::{EVENT_WORD_SIZE, ParseContext, ProtocolParser, ethereum_value_from_log_data, make_staking_transaction};

const EVENT_DELEGATED: &str = "0x24d7bda8602b916d64417f0dbfe2e2e88ec9b1157bd9f596dfdb91ba26624e04";
const EVENT_UNDELEGATED: &str = "0x3aace7340547de7b9156593a7652dc07ee900cea3fd8f82cb6c9d38b40829802";
const EVENT_REDELEGATED: &str = "0xfdac6e81913996d95abcc289e90f2d8bd235487ce6fe6f821e7d21002a1915b4";
const EVENT_CLAIMED: &str = "0xf7a40077ff7a04c7e61f6f26fb13774259ddf1b6bce9ecf26a8276cdd3992683";

pub struct SmartChainStakingParser;

impl ProtocolParser for SmartChainStakingParser {
    fn matches(&self, context: &ParseContext<'_>) -> bool {
        if *context.chain != Chain::SmartChain {
            return false;
        }

        context.transaction.to.as_ref().is_some_and(|to| to.eq_ignore_ascii_case(stake_hub::STAKE_HUB_ADDRESS))
    }

    fn parse(&self, context: &ParseContext<'_>) -> Option<PrimitivesTransaction> {
        context.receipt.logs.iter().find_map(|log| Self::parse_log(context, log))
    }
}

impl SmartChainStakingParser {
    fn parse_log(context: &ParseContext<'_>, log: &Log) -> Option<PrimitivesTransaction> {
        if !log.address.eq_ignore_ascii_case(stake_hub::STAKE_HUB_ADDRESS) {
            return None;
        }

        match log.topics.first()?.as_str() {
            EVENT_DELEGATED => Self::parse_delegated_event(context, log),
            EVENT_UNDELEGATED => Self::parse_undelegated_event(context, log),
            EVENT_REDELEGATED => Self::parse_redelegated_event(context, log),
            EVENT_CLAIMED => Self::parse_claimed_event(context, log),
            _ => None,
        }
    }

    fn parse_delegated_event(context: &ParseContext<'_>, log: &Log) -> Option<PrimitivesTransaction> {
        if log.topics.len() != 3 {
            return None;
        }

        let operator_address = ethereum_address_from_topic(&log.topics[1])?;
        make_staking_transaction(
            context,
            &operator_address,
            TransactionType::StakeDelegate,
            ethereum_value_from_log_data(&log.data, EVENT_WORD_SIZE, EVENT_WORD_SIZE * 2)?,
        )
    }

    fn parse_undelegated_event(context: &ParseContext<'_>, log: &Log) -> Option<PrimitivesTransaction> {
        if log.topics.len() != 3 {
            return None;
        }

        let operator_address = ethereum_address_from_topic(&log.topics[1])?;
        make_staking_transaction(
            context,
            &operator_address,
            TransactionType::StakeUndelegate,
            ethereum_value_from_log_data(&log.data, EVENT_WORD_SIZE, EVENT_WORD_SIZE * 2)?,
        )
    }

    fn parse_redelegated_event(context: &ParseContext<'_>, log: &Log) -> Option<PrimitivesTransaction> {
        if log.topics.len() != 4 {
            return None;
        }

        let dst_validator = ethereum_address_from_topic(&log.topics[2])?;
        make_staking_transaction(
            context,
            &dst_validator,
            TransactionType::StakeRedelegate,
            ethereum_value_from_log_data(&log.data, EVENT_WORD_SIZE * 2, EVENT_WORD_SIZE * 3)?,
        )
    }

    fn parse_claimed_event(context: &ParseContext<'_>, log: &Log) -> Option<PrimitivesTransaction> {
        if log.topics.len() != 3 {
            return None;
        }

        let operator_address = ethereum_address_from_topic(&log.topics[1])?;
        make_staking_transaction(
            context,
            &operator_address,
            TransactionType::StakeRewards,
            ethereum_value_from_log_data(&log.data, 0, EVENT_WORD_SIZE)?,
        )
    }
}

#[cfg(test)]
mod tests {
    use chrono::DateTime;
    use primitives::{Chain, TransactionType, testkit::json_rpc::load_json_rpc_result};

    use crate::rpc::{
        model::{Transaction, TransactionReceipt},
        parsers::ProtocolParsers,
    };

    use super::super::{assert_staking_transaction, map_transaction};

    #[test]
    fn test_map_smartchain_staking_transactions() {
        let cases = [
            (
                include_str!("../../../../testdata/smartchain/transaction_staking_delegate.json"),
                include_str!("../../../../testdata/smartchain/transaction_staking_delegate_receipt.json"),
                TransactionType::StakeDelegate,
                "0x51eD60604637989d19D29e43c5D94B098A0d1Af7",
                "0xd34403249B2d82AAdDB14e778422c966265e5Fb5",
                "1000000000000000000",
            ),
            (
                include_str!("../../../../testdata/smartchain/transaction_staking_undelegate.json"),
                include_str!("../../../../testdata/smartchain/transaction_staking_undelegate_receipt.json"),
                TransactionType::StakeUndelegate,
                "0xa103B70852B1fE3eF3a0B60B818279F9D0D337d9",
                "0x5c38FF8Ca2b16099C086bF36546e99b13D152C4c",
                "1045889308410801049",
            ),
            (
                include_str!("../../../../testdata/smartchain/transaction_staking_redelegate.json"),
                include_str!("../../../../testdata/smartchain/transaction_staking_redelegate_receipt.json"),
                TransactionType::StakeRedelegate,
                "0xB5a0A71Be7B79F2A8Bd19B3A4D54d1b85fA2d50b",
                "0xB58ac55EB6B10e4f7918D77C92aA1cF5bB2DEd5e",
                "2370599727993109265",
            ),
            (
                include_str!("../../../../testdata/smartchain/transaction_staking_claim_rewards.json"),
                include_str!("../../../../testdata/smartchain/transaction_staking_claim_rewards_receipt.json"),
                TransactionType::StakeRewards,
                "0x47B47f2586089F68Ec17384a437F96800f499274",
                "0xB12e8137eF499a1d81552DB11664a9E617fd350A",
                "4001085336323661069",
            ),
        ];

        for (transaction, receipt, transaction_type, from, to, value) in cases {
            let transaction = load_json_rpc_result::<Transaction>(transaction);
            let receipt = load_json_rpc_result::<TransactionReceipt>(receipt);
            let staking_transaction = map_transaction(&Chain::SmartChain, &transaction, &receipt, None);

            assert_staking_transaction(
                &staking_transaction,
                Chain::SmartChain,
                transaction_type,
                from,
                to,
                "0x0000000000000000000000000000000000002002",
                value,
            );
        }

        let mut transaction = load_json_rpc_result::<Transaction>(include_str!("../../../../testdata/smartchain/transaction_staking_delegate.json"));
        let receipt = load_json_rpc_result::<TransactionReceipt>(include_str!("../../../../testdata/smartchain/transaction_staking_delegate_receipt.json"));
        transaction.to = Some("0x1234567890123456789012345678901234567890".to_string());

        assert!(ProtocolParsers::map_transaction(&Chain::SmartChain, &transaction, &receipt, None, None, DateTime::default()).is_none());
    }
}
