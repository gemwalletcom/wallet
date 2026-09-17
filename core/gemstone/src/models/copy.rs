use primitives::Chain;

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Enum)]
pub enum GemCopyKind {
    Address { chain: Chain },
    SecretPhrase,
    PrivateKey,
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct GemCopy {
    pub kind: GemCopyKind,
    pub value: String,
    pub display: String,
}
