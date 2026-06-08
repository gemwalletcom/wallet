use primitives::{Account, WalletId, WalletType};

use crate::error::AccountDerivationError;

pub fn derive_wallet_id_from_account(account: &Account, wallet_type: WalletType) -> Result<WalletId, AccountDerivationError> {
    match wallet_type {
        WalletType::Multicoin => Ok(WalletId::Multicoin(account.address.clone())),
        WalletType::Single => Ok(WalletId::Single(account.chain, account.address.clone())),
        WalletType::PrivateKey => Ok(WalletId::PrivateKey(account.chain, account.address.clone())),
        WalletType::View => Err(AccountDerivationError::unsupported("view wallet id is app-only")),
    }
}

#[cfg(test)]
mod tests {
    use primitives::Chain;

    use super::*;

    #[test]
    fn test_wallet_id_from_account() {
        let account = Account {
            chain: Chain::Ethereum,
            address: "0x4ce31c0b2114abe61Ac123E1E6254E961C18D10B".to_string(),
            derivation_path: "m/44'/60'/0'/0/0".to_string(),
            extended_public_key: None,
        };

        assert_eq!(
            derive_wallet_id_from_account(&account, WalletType::Multicoin).unwrap().to_string(),
            "multicoin_0x4ce31c0b2114abe61Ac123E1E6254E961C18D10B"
        );
        assert_eq!(
            derive_wallet_id_from_account(&account, WalletType::Single).unwrap().to_string(),
            "single_ethereum_0x4ce31c0b2114abe61Ac123E1E6254E961C18D10B"
        );
        assert_eq!(
            derive_wallet_id_from_account(&account, WalletType::PrivateKey).unwrap().to_string(),
            "privateKey_ethereum_0x4ce31c0b2114abe61Ac123E1E6254E961C18D10B"
        );
        assert_eq!(
            derive_wallet_id_from_account(&account, WalletType::View).unwrap_err(),
            AccountDerivationError::unsupported("view wallet id is app-only")
        );
    }
}
