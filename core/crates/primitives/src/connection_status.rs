use crate::ConnectionComponent;
use model_derive::Model;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Model)]
#[serde(rename_all = "camelCase")]
#[model(swift = "Equatable, Sendable")]
pub enum ConnectionStatus {
    Online,
    NoInternet,
    NoService,
}

impl ConnectionStatus {
    pub fn from_unhealthy_components(components: &[ConnectionComponent]) -> Self {
        components.iter().map(|component| component.failure_status()).max_by_key(|status| status.severity()).unwrap_or(Self::Online)
    }

    fn severity(&self) -> u8 {
        match self {
            Self::Online => 0,
            Self::NoService => 1,
            Self::NoInternet => 2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_unhealthy_components() {
        assert_eq!(ConnectionStatus::from_unhealthy_components(&[]), ConnectionStatus::Online);
        assert_eq!(ConnectionStatus::from_unhealthy_components(&[ConnectionComponent::Internet]), ConnectionStatus::NoInternet);
        assert_eq!(ConnectionStatus::from_unhealthy_components(&[ConnectionComponent::Stream]), ConnectionStatus::NoService);
        assert_eq!(ConnectionStatus::from_unhealthy_components(&[ConnectionComponent::Stream, ConnectionComponent::Internet]), ConnectionStatus::NoInternet);
    }
}
