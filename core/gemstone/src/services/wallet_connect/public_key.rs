use gem_wallet_connect::requires_public_key;
use primitives::{Account, Chain};

use super::GemWalletConnectService;

impl GemWalletConnectService {
    pub(super) async fn with_public_keys(&self, accounts: Vec<Account>, chains: &[Chain]) -> Vec<Account> {
        let mut session_accounts = Vec::with_capacity(accounts.len());
        for account in accounts {
            session_accounts.push(self.with_public_key(account, chains).await);
        }
        session_accounts
    }

    async fn with_public_key(&self, account: Account, chains: &[Chain]) -> Account {
        if !needs_public_key(&account, chains) {
            return account;
        }
        match self.gateway.get_public_key(account.chain, account.address.clone()).await {
            Ok(Some(public_key)) => Account {
                extended_public_key: Some(public_key),
                ..account
            },
            Ok(None) | Err(_) => account,
        }
    }
}

fn needs_public_key(account: &Account, chains: &[Chain]) -> bool {
    requires_public_key(account.chain) && chains.contains(&account.chain) && account.extended_public_key.as_deref().is_none_or(str::is_empty)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use futures::executor::block_on;
    use gem_wallet_connect::testkit::{TEST_TON_PUBLIC_KEY, mock_ton_account};
    use primitives::{Wallet, WalletConnectionVerificationStatus};

    use super::*;
    use crate::services::wallet_connect::testkit::TestWalletConnectSigner;
    use crate::testkit::TestAlienProvider;

    #[test]
    fn test_needs_public_key_only_for_a_session_account_without_a_required_key() {
        let account = |chain: Chain, public_key: Option<&str>| Account {
            extended_public_key: public_key.map(str::to_string),
            ..Account::mock(chain, "address")
        };

        assert!(needs_public_key(&account(Chain::Ton, None), &[Chain::Ton]));
        assert!(needs_public_key(&account(Chain::Ton, Some("")), &[Chain::Ton]));
        assert!(!needs_public_key(&account(Chain::Ton, Some("key")), &[Chain::Ton]));
        assert!(!needs_public_key(&account(Chain::Ton, None), &[Chain::Ethereum]));
        assert!(needs_public_key(&account(Chain::Sui, None), &[Chain::Sui]));
        assert!(!needs_public_key(&account(Chain::Ethereum, None), &[Chain::Ethereum]));
    }

    fn ton_connect_proposal_chain_ids(namespaces: &str) -> Vec<String> {
        let proposal: serde_json::Value = serde_json::from_str(include_str!("../../../../crates/gem_wallet_connect/testdata/ton_connect_session_proposal.json")).unwrap();
        proposal[namespaces]
            .as_object()
            .unwrap()
            .values()
            .flat_map(|namespace| namespace["chains"].as_array().unwrap().iter().map(|chain| chain.as_str().unwrap().to_string()))
            .collect()
    }

    async fn ton_connect_session_properties(stored_public_key: Option<&str>, chain_public_key: &str) -> (HashMap<String, String>, Arc<TestAlienProvider>) {
        let account = Account {
            extended_public_key: stored_public_key.map(str::to_string),
            ..mock_ton_account()
        };
        let wallet = Wallet::mock_with_accounts(vec![account]);
        let body = format!(r#"{{"gas_used":1021,"exit_code":0,"stack":[{{"type":"num","value":"0x{chain_public_key}"}}]}}"#);
        let provider = Arc::new(TestAlienProvider::with_json_by_path(200, &[("runGetMethod", body.as_str())]));
        let service = GemWalletConnectService::mock_with_signer(Arc::new(TestWalletConnectSigner::new(Ok(String::new()))), wallet.clone(), provider.clone()).await;
        let metadata = service.application_metadata("Demo DApp".to_string(), "Demo DApp".to_string(), "https://demo.ton-connect.io".to_string(), vec![]);

        let prepared = service
            .prepare_session_proposal(
                ton_connect_proposal_chain_ids("requiredNamespaces"),
                ton_connect_proposal_chain_ids("optionalNamespaces"),
                metadata,
                Some("https://demo.ton-connect.io".to_string()),
                WalletConnectionVerificationStatus::Verified,
            )
            .await
            .unwrap();
        assert_eq!(prepared.proposal.default_wallet.id, wallet.id);

        let approval = service.session_approval(prepared.proposal.default_wallet);
        let properties = service.config_session_properties(HashMap::new(), vec!["ton:-239".to_string()], approval.accounts).await;
        (properties, provider)
    }

    #[test]
    fn test_ton_connect_session_reads_a_missing_public_key_from_chain() {
        block_on(async {
            let (properties, provider) = ton_connect_session_properties(None, TEST_TON_PUBLIC_KEY).await;

            assert_eq!(properties.get("ton_getPublicKey").map(String::as_str), Some(TEST_TON_PUBLIC_KEY));
            assert!(properties.contains_key("ton_getStateInit"), "TON Connect drops a session without the wallet state init");
            assert_eq!(provider.requested_paths().len(), 1);
        });
    }

    #[test]
    fn test_ton_connect_session_rejects_a_chain_public_key_for_another_wallet() {
        block_on(async {
            let (properties, _) = ton_connect_session_properties(Some(""), "7952e94118f34607c75e23258dd9220d66ccac5a3ee074125c25068e8107bfbf").await;

            assert!(properties.is_empty(), "a key that does not derive the account address is never sent");
        });
    }

    #[test]
    fn test_ton_connect_session_uses_the_stored_public_key() {
        block_on(async {
            let (properties, provider) = ton_connect_session_properties(Some(TEST_TON_PUBLIC_KEY), TEST_TON_PUBLIC_KEY).await;

            assert_eq!(properties.get("ton_getPublicKey").map(String::as_str), Some(TEST_TON_PUBLIC_KEY));
            assert!(provider.requested_paths().is_empty());
        });
    }
}
