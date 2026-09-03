use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SmartQueryResponse<T> {
    pub data: T,
}
