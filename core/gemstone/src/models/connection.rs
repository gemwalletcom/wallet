use primitives::{ConnectionComponent, ConnectionStatus};

pub type GemConnectionStatus = ConnectionStatus;
pub type GemConnectionComponent = ConnectionComponent;

#[uniffi::export]
pub fn connection_status(unhealthy_components: Vec<GemConnectionComponent>) -> GemConnectionStatus {
    ConnectionStatus::from_unhealthy_components(&unhealthy_components)
}
