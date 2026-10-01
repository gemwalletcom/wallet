use primitives::node_config::NodeRegion;

#[uniffi::remote(Enum)]
pub enum NodeRegion {
    Us,
    Eu,
    Asia,
}
