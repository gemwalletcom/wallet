use super::model::GemWalletConnectEvent;

#[uniffi::export(rust, foreign)]
pub trait GemWalletConnectListener: Send + Sync {
    fn on_event(&self, event: GemWalletConnectEvent);
}

#[uniffi::export(rust, foreign)]
pub trait GemSignatureVerifier: Send + Sync {
    fn is_valid_es256(&self, public_key: Vec<u8>, message: Vec<u8>, signature: Vec<u8>) -> bool;
}
