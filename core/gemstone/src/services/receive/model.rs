#[derive(Clone, Copy, Debug, PartialEq, uniffi::Enum)]
pub enum GemMemoWarning {
    NotSupported,
    DestinationTag,
    Memo,
}
