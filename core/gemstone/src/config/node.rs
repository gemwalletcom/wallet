use primitives::node_config::NodeRegion;

// Sources:
// https://chainlist.org

#[uniffi::remote(Enum)]
pub enum NodeRegion {
    Us,
    Eu,
    Asia,
}
