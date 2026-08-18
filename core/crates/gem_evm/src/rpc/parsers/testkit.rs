use chrono::DateTime;
use primitives::{Chain, Transaction as PrimitivesTransaction};

use super::{ProtocolParser, ProtocolParsers};
use crate::rpc::model::{Transaction, TransactionReceipt};

pub fn map_transaction(parser: Box<dyn ProtocolParser>, chain: &Chain, transaction: &Transaction, receipt: &TransactionReceipt) -> PrimitivesTransaction {
    ProtocolParsers::map_transaction(chain, transaction, receipt, DateTime::default(), &[parser]).unwrap()
}
