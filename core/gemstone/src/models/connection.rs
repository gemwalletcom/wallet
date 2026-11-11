use primitives::{ConnectionComponent, ConnectionStatus};

#[uniffi::export]
pub fn connection_status(unhealthy_components: Vec<ConnectionComponent>) -> ConnectionStatus {
    ConnectionStatus::from_unhealthy_components(&unhealthy_components)
}
