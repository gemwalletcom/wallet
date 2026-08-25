use primitives::ApplicationMetadata;

#[uniffi::export]
pub fn application_metadata_short_name(metadata: ApplicationMetadata) -> String {
    metadata.short_name()
}
