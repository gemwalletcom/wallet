use gem_evm::ethereum_address_checksum;
use gem_evm::rpc::model::Log;
use gem_evm::rpc::parsers::staking::make_staking_transaction;
use gem_evm::rpc::parsers::{EVENT_WORD_SIZE, ParseContext, ProtocolParser, ethereum_value_from_log_data};
use primitives::{Chain, Transaction as PrimitivesTransaction, TransactionType};

use crate::constants::{EVERSTAKE_ACCOUNTING_ADDRESS, EVERSTAKE_POOL_ADDRESS};

const EVENT_STAKED: &str = "0x7d194e8dc0f902cdc51bde00649039561dbd0b01574d671bad333436fdac7692";
const EVENT_UNSTAKED: &str = "0x0750a71dce555de583ab0225a108df42b9402d22123d7cc9cd95793e43e7db0e";
const EVENT_WITHDRAWN: &str = "0x262159451c4018521811107ecbe27e3de7d95a70a4a534f733aa59bc4346f03e";

pub struct EverstakeParser;

impl ProtocolParser for EverstakeParser {
    fn matches(&self, context: &ParseContext<'_>) -> bool {
        *context.chain == Chain::Ethereum && context.is_to_any(&[EVERSTAKE_POOL_ADDRESS, EVERSTAKE_ACCOUNTING_ADDRESS])
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
    use chrono::DateTime;
    use gem_evm::rpc::parsers::ProtocolParsers;
    use primitives::{Chain, TransactionType, testkit::json_rpc::load_json_rpc_result};

    use super::EverstakeParser;
    use crate::constants::{EVERSTAKE_ACCOUNTING_ADDRESS, EVERSTAKE_POOL_ADDRESS};

    #[test]
    fn test_map_everstake_transactions() {
        let cases = [
            (
                include_str!("../testdata/transaction_stake.json"),
                include_str!("../testdata/transaction_stake_receipt.json"),
                TransactionType::StakeDelegate,
                "0x0D9DAB1A248f63B0a48965bA8435e4de7497a3dC",
                EVERSTAKE_POOL_ADDRESS,
                EVERSTAKE_POOL_ADDRESS,
                "34800000000000000000",
            ),
            (
                include_str!("../testdata/transaction_unstake.json"),
                include_str!("../testdata/transaction_unstake_receipt.json"),
                TransactionType::StakeUndelegate,
                "0x1085c5f70F7F7591D97da281A64688385455c2bD",
                EVERSTAKE_POOL_ADDRESS,
                EVERSTAKE_POOL_ADDRESS,
                "50000000000000000",
            ),
            (
                include_str!("../testdata/transaction_withdraw.json"),
                include_str!("../testdata/transaction_withdraw_receipt.json"),
                TransactionType::StakeWithdraw,
                "0x1085c5f70F7F7591D97da281A64688385455c2bD",
                EVERSTAKE_POOL_ADDRESS,
                EVERSTAKE_ACCOUNTING_ADDRESS,
                "50000000000000000",
            ),
        ];

        for (transaction, receipt, transaction_type, from, to, contract, value) in cases {
            let staking_transaction = ProtocolParsers::map_transaction_with_parsers(
                &Chain::Ethereum,
                &load_json_rpc_result(transaction),
                &load_json_rpc_result(receipt),
                DateTime::default(),
                &[&EverstakeParser],
            )
            .unwrap();
            assert_eq!(staking_transaction.transaction_type, transaction_type);
            assert_eq!(staking_transaction.from, from);
            assert_eq!(staking_transaction.to, to);
            assert_eq!(staking_transaction.contract.as_deref(), Some(contract));
            assert_eq!(staking_transaction.value.to_string(), value);
        }
    }
}
