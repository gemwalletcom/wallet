use crate::{WalletConfiguration, WalletConfigurationResult, WalletId};

impl WalletConfigurationResult {
    pub fn mock(wallet_id: WalletId) -> Self {
        Self {
            wallet_id,
            configuration: WalletConfiguration {
                multi_signature_accounts: vec![],
                externally_controlled_accounts: vec![],
            },
        }
    }
}
