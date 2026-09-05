use super::{GemLocalizedText, GemLocalizer};

#[derive(Default)]
pub struct EnglishLocalizer;

impl GemLocalizer for EnglishLocalizer {
    fn text(&self, text: GemLocalizedText) -> String {
        match text {
            GemLocalizedText::WalletDefaultName { index } => format!("Wallet #{index}"),
            GemLocalizedText::WalletDefaultNameChain { chain, index } => format!("{} Wallet #{index}", chain.as_ref()),
        }
    }
}
