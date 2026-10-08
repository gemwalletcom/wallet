use std::fmt;

pub enum DeviceError {
    MissingHeader(&'static str),
    InvalidDeviceId,
    InvalidTimestamp,
    TimestampExpired,
    InvalidSignature,
    DeviceNotFound,
    WalletNotFound,
    MissingWalletId,
    InvalidAuthorizationFormat,
    ReplayedRequest,
    DatabaseError,
}

impl DeviceError {
    pub fn message(&self, device_id: Option<&str>, wallet_id: Option<&str>) -> String {
        let mut message = self.to_string();
        if let Some(id) = device_id {
            message.push_str(&format!(" device_id={id}"));
        }
        if let Some(id) = wallet_id {
            message.push_str(&format!(" wallet_id={id}"));
        }
        message
    }
}

impl fmt::Display for DeviceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingHeader(name) => write!(f, "Missing header: {}", name),
            Self::InvalidDeviceId => write!(f, "Invalid device ID"),
            Self::InvalidTimestamp => write!(f, "Invalid timestamp"),
            Self::TimestampExpired => write!(f, "Timestamp expired"),
            Self::InvalidSignature => write!(f, "Invalid signature"),
            Self::DeviceNotFound => write!(f, "Device not found"),
            Self::WalletNotFound => write!(f, "Wallet not found"),
            Self::MissingWalletId => write!(f, "Missing wallet ID"),
            Self::InvalidAuthorizationFormat => write!(f, "Invalid authorization format"),
            Self::ReplayedRequest => write!(f, "Replayed request"),
            Self::DatabaseError => write!(f, "Database error"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DeviceError;

    #[test]
    fn test_message_includes_wallet_id() {
        assert_eq!(DeviceError::WalletNotFound.message(Some("device_123"), Some("wallet_456")), "Wallet not found device_id=device_123 wallet_id=wallet_456");
    }
}
