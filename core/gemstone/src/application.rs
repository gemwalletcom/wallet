use primitives::ApplicationMetadata;

#[derive(Default, uniffi::Object)]
pub struct GemApplicationMetadataService {}

#[uniffi::export]
impl GemApplicationMetadataService {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self {}
    }

    pub fn short_name(&self, metadata: ApplicationMetadata) -> String {
        metadata.short_name()
    }
}
