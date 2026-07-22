use crate::ConnectionComponent;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[typeshare(swift = "Equatable, Sendable")]
pub enum ConnectionStatus {
    Online,
    NoInternet,
    NoService,
}

impl ConnectionStatus {
    pub fn from_unhealthy_components(components: &[ConnectionComponent]) -> Self {
        components
            .iter()
            .map(|component| component.failure_status())
            .max_by_key(|status| status.severity())
            .unwrap_or(Self::Online)
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
        assert_eq!(ConnectionStatus::from_unhealthy_components(&[ConnectionComponent::Api]), ConnectionStatus::NoService);
        assert_eq!(
            ConnectionStatus::from_unhealthy_components(&[ConnectionComponent::Nodes, ConnectionComponent::Stream]),
            ConnectionStatus::NoService
        );
        assert_eq!(
            ConnectionStatus::from_unhealthy_components(&[ConnectionComponent::Api, ConnectionComponent::Internet]),
            ConnectionStatus::NoInternet
        );
    }
}
