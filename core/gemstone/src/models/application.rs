use primitives::{ApplicationMetadata, ApplicationMetadataSource};

pub type GemApplicationMetadata = ApplicationMetadata;
pub type GemApplicationMetadataSource = ApplicationMetadataSource;

#[uniffi::remote(Enum)]
pub enum GemApplicationMetadataSource {
    WalletConnect,
    Payment,
}

#[uniffi::remote(Record)]
pub struct GemApplicationMetadata {
    pub name: String,
    pub description: String,
    pub url: String,
    pub icon: String,
    pub source: GemApplicationMetadataSource,
}
