mod relay;

use chain_traits::transaction_parser::{ParseContext as GenericParseContext, TransactionParser, parse_transaction};
use chrono::{DateTime, Utc};
use num_bigint::BigUint;

use crate::models::BlockTransaction;
use primitives::{AssetId, Chain, Transaction, TransactionState, TransactionType};

use self::relay::RelayParser;

struct ParseMetadata<'a> {
    memo: Option<&'a str>,
}

type ParseContext<'a> = GenericParseContext<'a, BlockTransaction, ParseMetadata<'a>>;
type ProtocolParser = dyn for<'a> TransactionParser<ParseContext<'a>, Transaction>;

trait ParseContextExt {
    fn make_swap_transaction(&self, from: String, program_id: &str, asset_id: AssetId, value: BigUint) -> Option<Transaction>;
}

impl ParseContextExt for ParseContext<'_> {
    fn make_swap_transaction(&self, from: String, program_id: &str, asset_id: AssetId, value: BigUint) -> Option<Transaction> {
        let state = if self.transaction.meta.has_error() {
            TransactionState::Reverted
        } else {
            TransactionState::Confirmed
        };

        Some(Transaction::new(
            self.transaction.transaction.signatures.first()?.clone(),
            asset_id,
            from,
            program_id.to_string(),
            Some(program_id.to_string()),
            TransactionType::Swap,
            state,
            self.transaction.fee(),
            Chain::Solana.as_asset_id(),
            value,
            self.metadata.memo.map(str::to_owned),
            None,
            self.created_at,
        ))
    }
}

pub(super) struct ProtocolParsers;

impl ProtocolParsers {
    fn default_parsers() -> [&'static ProtocolParser; 1] {
        [&RelayParser]
    }

    pub(super) fn map_transaction(transaction: &BlockTransaction, created_at: DateTime<Utc>, memo: Option<&str>) -> Option<Transaction> {
        let context = ParseContext::new(transaction, created_at, ParseMetadata { memo });

        parse_transaction(&context, Self::default_parsers())
    }
}
