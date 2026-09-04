use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Data<T> {
    pub data: T,
}

#[derive(Debug, Deserialize)]
pub struct Record {
    pub username: Option<Username>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Username {
    pub linked_to: Option<String>,
}
