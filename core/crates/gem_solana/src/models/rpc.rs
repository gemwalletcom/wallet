use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueResult<T> {
    pub value: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueData<T> {
    pub data: T,
    pub owner: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parsed<T> {
    pub parsed: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Info<T> {
    pub info: T,
}

pub type AccountData = ValueData<Vec<String>>;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub ok: Option<String>,
}
