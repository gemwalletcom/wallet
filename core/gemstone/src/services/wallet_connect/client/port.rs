use super::model::GemWalletConnectEvent;

#[uniffi::export(rust, foreign)]
pub trait GemWalletConnectListener: Send + Sync {
    fn on_event(&self, event: GemWalletConnectEvent);
}
