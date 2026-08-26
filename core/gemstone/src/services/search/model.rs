#[derive(Debug, Clone, PartialEq, Eq, uniffi::Enum)]
pub enum GemSearchScope {
    All,
    List { id: String },
}
