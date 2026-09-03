use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct Data<T> {
    pub data: T,
}

#[derive(Debug, Deserialize)]
pub struct Records {
    pub records: Vec<Record>,
}

#[derive(Debug, Deserialize)]
pub struct Record {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Serialize)]
pub struct AccountRequest<'a> {
    pub account: &'a str,
}
