use crate::GemstoneError;
use crate::alien::AlienError;
use crate::gateway::GatewayError;
use crate::payment::GemPaymentError;
use crate::services::error::GemServiceError;
use crate::services::node::model::GemAddNodeError;
use crate::services::wallet_connect::error::GemWalletConnectError;
use primitives::PaymentStatus;

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Enum)]
pub enum GemErrorText {
    Cancelled,
    NetworkOffline,
    NetworkMessage { text: String },
    NetworkStatus { status: u32 },
    InvalidNetworkId,
    InvalidUrl,
    NotSupported,
    UnsupportedChain,
    MaliciousOrigin,
    NoSupportedWallets,
    Payment { status: PaymentStatus },
    Message { text: String },
}

#[uniffi::export]
impl GemServiceError {
    pub fn text(&self) -> GemErrorText {
        match self {
            Self::Api { msg }
            | Self::Gateway { msg }
            | Self::Store { msg }
            | Self::Core { msg }
            | Self::Platform { msg }
            | Self::InvalidInput { msg }
            | Self::NotFound { msg }
            | Self::Unsupported { msg } => GemErrorText::Message { text: msg.clone() },
            Self::Cancelled => GemErrorText::Cancelled,
        }
    }
}

#[uniffi::export]
impl GemstoneError {
    pub fn text(&self) -> GemErrorText {
        match self {
            Self::AnyError { msg } | Self::SignerError { msg, .. } => GemErrorText::Message { text: msg.clone() },
            Self::Cancelled => GemErrorText::Cancelled,
        }
    }
}

#[uniffi::export]
impl GatewayError {
    pub fn text(&self) -> GemErrorText {
        match self {
            Self::Offline => GemErrorText::NetworkOffline,
            Self::NetworkError { msg } | Self::PlatformError { msg } => GemErrorText::Message { text: msg.clone() },
            Self::NetworkIdMismatch { .. } => GemErrorText::InvalidNetworkId,
        }
    }
}

#[uniffi::export]
impl GemAddNodeError {
    pub fn text(&self) -> GemErrorText {
        match self {
            Self::InvalidUrl => GemErrorText::InvalidUrl,
            Self::InvalidNetworkId => GemErrorText::InvalidNetworkId,
            Self::Gateway(error) => error.text(),
        }
    }
}

#[uniffi::export]
impl GemWalletConnectError {
    pub fn text(&self) -> GemErrorText {
        match self {
            Self::UnsupportedChains => GemErrorText::UnsupportedChain,
            Self::InvalidOrigin => GemErrorText::MaliciousOrigin,
            Self::UnsupportedWallets => GemErrorText::NoSupportedWallets,
            Self::Service { msg } => GemErrorText::Message { text: msg.clone() },
        }
    }
}

#[uniffi::export]
pub fn alien_error_text(error: AlienError) -> GemErrorText {
    match error {
        AlienError::RequestError { msg } | AlienError::ResponseError { msg } => GemErrorText::NetworkMessage { text: msg },
        AlienError::Http { status, .. } => GemErrorText::NetworkStatus { status: status as u32 },
        AlienError::Offline => GemErrorText::NetworkOffline,
    }
}

#[uniffi::export]
pub fn payment_error_text(error: GemPaymentError) -> GemErrorText {
    match error {
        GemPaymentError::NoPaymentOptions => GemErrorText::NotSupported,
        GemPaymentError::Status { status } => GemErrorText::Payment { status },
        GemPaymentError::InvalidRequest { reason } | GemPaymentError::Network { reason } => GemErrorText::Message { text: reason },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a_transport_message_is_named_apart_from_a_service_message() {
        assert_eq!(
            alien_error_text(AlienError::RequestError { msg: "timeout".into() }),
            GemErrorText::NetworkMessage { text: "timeout".into() },
            "a transport failure reads as a network error on both apps"
        );
        assert_eq!(alien_error_text(AlienError::Offline), GemErrorText::NetworkOffline);
        assert_eq!(alien_error_text(AlienError::Http { status: 503, len: 0 }), GemErrorText::NetworkStatus { status: 503 });
        assert_eq!(
            GatewayError::NetworkError { msg: "reverted".into() }.text(),
            GemErrorText::Message { text: "reverted".into() },
            "a message the gateway already phrased is shown as it is"
        );
    }

    #[test]
    fn test_a_carried_message_stays_the_message_and_a_decision_becomes_a_key() {
        assert_eq!(GemServiceError::Api { msg: "boom".into() }.text(), GemErrorText::Message { text: "boom".into() });
        assert_eq!(GemServiceError::Cancelled.text(), GemErrorText::Cancelled);
        assert_eq!(GatewayError::Offline.text(), GemErrorText::NetworkOffline);
        assert_eq!(
            GatewayError::NetworkIdMismatch {
                chain: "ethereum".into(),
                network_id: "1".into()
            }
            .text(),
            GemErrorText::InvalidNetworkId
        );
    }

    #[test]
    fn test_an_added_node_reports_the_gateway_answer_it_wraps() {
        assert_eq!(GemAddNodeError::InvalidUrl.text(), GemErrorText::InvalidUrl);
        assert_eq!(GemAddNodeError::Gateway(GatewayError::Offline).text(), GemErrorText::NetworkOffline);
        assert_eq!(alien_error_text(AlienError::Http { status: 503, len: 0 }), GemErrorText::NetworkStatus { status: 503 });
    }
}
