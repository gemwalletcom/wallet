use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Response<T> {
    pub s: String,
    pub result: T,
}

#[derive(Debug, Deserialize)]
pub struct RecordResult {
    pub deserialized: String,
}
