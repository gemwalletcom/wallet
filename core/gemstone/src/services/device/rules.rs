use primitives::{Device, Wallet};

pub fn subscriptions_signature(wallets: &[Wallet]) -> String {
    let mut entries: Vec<String> = wallets
        .iter()
        .flat_map(|wallet| {
            wallet
                .accounts
                .iter()
                .map(move |account| format!("{}/{}/{}", wallet.id.id(), account.chain.as_ref(), account.address))
        })
        .collect();
    entries.sort();
    entries.join(";")
}

pub fn device_changed(current: &Device, other: &Device) -> bool {
    current.id != other.id
        || current.token != other.token
        || current.locale != other.locale
        || current.version != other.version
        || current.currency != other.currency
        || current.is_push_enabled != other.is_push_enabled
        || current.is_price_alerts_enabled != other.is_price_alerts_enabled
        || current.subscriptions_version != other.subscriptions_version
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::currency::Currency;
    use primitives::{Account, Chain, WalletId, WalletSource};

    fn wallet(id: &str, accounts: Vec<Account>) -> Wallet {
        Wallet {
            id: WalletId::Multicoin(id.into()),
            name: "Test Wallet".into(),
            source: WalletSource::Create,
            ..Wallet::mock_with_accounts(accounts)
        }
    }

    #[test]
    fn test_signature_ignores_rename_and_pin_but_tracks_accounts_and_order() {
        let base = wallet("wallet1", vec![Account::mock(Chain::Ethereum, "0xabc")]);
        let renamed = Wallet {
            name: "Renamed".into(),
            is_pinned: true,
            ..base.clone()
        };
        let extended = Wallet {
            accounts: vec![Account::mock(Chain::Ethereum, "0xabc"), Account::mock(Chain::Bitcoin, "bc1xyz")],
            ..base.clone()
        };
        let other = wallet("wallet2", vec![Account::mock(Chain::Solana, "solana123")]);

        assert_eq!(subscriptions_signature(std::slice::from_ref(&base)), subscriptions_signature(&[renamed]));
        assert_ne!(subscriptions_signature(std::slice::from_ref(&base)), subscriptions_signature(&[extended]));
        assert_eq!(subscriptions_signature(&[base.clone(), other.clone()]), subscriptions_signature(&[other, base]));
    }

    #[test]
    fn test_device_changed_tracks_synced_fields_only() {
        let remote = Device::mock();

        assert!(!device_changed(&remote, &remote.clone()));
        assert!(device_changed(
            &remote,
            &Device {
                currency: Currency::EUR,
                ..remote.clone()
            }
        ));
        assert!(device_changed(
            &remote,
            &Device {
                subscriptions_version: 2,
                ..remote.clone()
            }
        ));
        assert!(!device_changed(
            &remote,
            &Device {
                model: "Other".into(),
                ..remote.clone()
            }
        ));
    }
}
