use chrono::{DateTime, Utc};

pub struct ParseContext<'a, Transaction, Metadata> {
    pub transaction: &'a Transaction,
    pub created_at: DateTime<Utc>,
    pub metadata: Metadata,
}

impl<'a, Transaction, Metadata> ParseContext<'a, Transaction, Metadata> {
    pub fn new(transaction: &'a Transaction, created_at: DateTime<Utc>, metadata: Metadata) -> Self {
        Self {
            transaction,
            created_at,
            metadata,
        }
    }
}

pub trait TransactionParser<Context, Output>: Send + Sync {
    fn matches(&self, context: &Context) -> bool;
    fn parse(&self, context: &Context) -> Option<Output>;
}

pub fn parse_transaction<'a, Context, Output, Parser>(context: &Context, parsers: impl IntoIterator<Item = &'a Parser>) -> Option<Output>
where
    Parser: TransactionParser<Context, Output> + ?Sized + 'a,
{
    parsers.into_iter().filter(|parser| parser.matches(context)).find_map(|parser| parser.parse(context))
}
