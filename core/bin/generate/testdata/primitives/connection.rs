#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Model)]
#[model(swift = "Equatable, Sendable")]
#[serde(rename_all = "camelCase")]
pub enum ConnectionState {
    Connected,
    Disconnected,
    NotReachable,
}
