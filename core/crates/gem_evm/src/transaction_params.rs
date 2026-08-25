use num_bigint::BigInt;

pub struct TransactionParams {
    pub to: String,
    pub data: Vec<u8>,
    pub value: BigInt,
}

impl TransactionParams {
    pub fn new(to: impl Into<String>, data: Vec<u8>, value: BigInt) -> Self {
        Self { to: to.into(), data, value }
    }

    pub fn new_approval(to: impl Into<String>, data: Vec<u8>) -> Self {
        Self {
            to: to.into(),
            data,
            value: BigInt::from(0),
        }
    }
}
