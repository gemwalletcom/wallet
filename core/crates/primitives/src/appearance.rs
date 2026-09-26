use model_derive::Model;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Model)]
#[model(swift = "Equatable, Hashable, Sendable, CaseIterable")]
#[serde(rename_all = "lowercase")]
pub enum Appearance {
    System,
    Light,
    Dark,
}
