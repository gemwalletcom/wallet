#[derive(Clone, Copy, Debug, PartialEq, Eq, uniffi::Enum)]
pub enum GemReceiveWarning {
    AssetNetwork,
    NoDestinationTagRequired,
    NoMemoRequired,
}
