use std::collections::BTreeMap;

use crate::sign::types::{ProposalNamespace, Reason, SessionNamespace, SessionProposal};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NamespaceError {
    UnsupportedChains,
    UnsupportedAccounts,
    UnsupportedMethods,
    UnsupportedEvents,
}

pub struct Supported<'a> {
    pub chains: &'a [String],
    pub accounts: &'a [String],
    pub methods: &'a [String],
    pub events: &'a [String],
}

pub fn approved_namespaces(proposal: &SessionProposal, supported: &Supported) -> Result<BTreeMap<String, SessionNamespace>, NamespaceError> {
    let mut namespaces = BTreeMap::new();
    for (key, namespace) in &proposal.required_namespaces {
        let (name, chains) = proposal_chains(key, namespace);
        if chains.is_empty() || !chains.iter().all(|chain| supported.chains.contains(chain)) {
            return Err(NamespaceError::UnsupportedChains);
        }
        if !namespace.methods.iter().all(|method| supported.methods.contains(method)) {
            return Err(NamespaceError::UnsupportedMethods);
        }
        if !namespace.events.iter().all(|event| supported.events.contains(event)) {
            return Err(NamespaceError::UnsupportedEvents);
        }
        let accounts = accounts_on(supported.accounts, &chains);
        if accounts.is_empty() {
            return Err(NamespaceError::UnsupportedAccounts);
        }
        merge(&mut namespaces, name, chains, accounts, namespace.methods.clone(), namespace.events.clone());
    }
    for (key, namespace) in proposal.optional_namespaces.iter().flatten() {
        let (name, chains) = proposal_chains(key, namespace);
        let chains = intersection(&chains, supported.chains);
        let methods = intersection(&namespace.methods, supported.methods);
        let accounts = accounts_on(supported.accounts, &chains);
        if chains.is_empty() || methods.is_empty() || accounts.is_empty() {
            continue;
        }
        merge(&mut namespaces, name, chains, accounts, methods, intersection(&namespace.events, supported.events));
    }
    if namespaces.is_empty() {
        return Err(NamespaceError::UnsupportedAccounts);
    }
    Ok(namespaces)
}

pub fn adding_account(namespaces: &BTreeMap<String, SessionNamespace>, account: &str) -> BTreeMap<String, SessionNamespace> {
    let mut namespaces = namespaces.clone();
    let (Some((name, _)), Some((chain, _))) = (account.split_once(':'), account.rsplit_once(':')) else {
        return namespaces;
    };
    if let Some(namespace) = namespaces.get_mut(name) {
        if let Some(chains) = namespace.chains.as_mut()
            && !chains.iter().any(|existing| existing == chain)
        {
            chains.push(chain.to_string());
        }
        if !namespace.accounts.iter().any(|existing| existing == account) {
            namespace.accounts.push(account.to_string());
        }
    }
    namespaces
}

pub fn authorization(namespaces: &BTreeMap<String, SessionNamespace>, chain_id: &str, method: &str) -> Result<(), Reason> {
    let mut on_chain = namespaces.iter().filter(|(key, namespace)| namespace_chains(key, namespace).iter().any(|chain| chain == chain_id)).peekable();
    if on_chain.peek().is_none() {
        return Err(Reason::unauthorized_chain());
    }
    match on_chain.any(|(_, namespace)| namespace.methods.iter().any(|existing| existing == method)) {
        true => Ok(()),
        false => Err(Reason::unauthorized_method(method)),
    }
}

pub fn approved_chains(namespaces: &BTreeMap<String, SessionNamespace>) -> Vec<String> {
    sorted(namespaces.iter().flat_map(|(key, namespace)| namespace_chains(key, namespace)))
}

pub fn approved_methods(namespaces: &BTreeMap<String, SessionNamespace>) -> Vec<String> {
    sorted(namespaces.values().flat_map(|namespace| namespace.methods.clone()))
}

pub fn approved_events(namespaces: &BTreeMap<String, SessionNamespace>) -> Vec<String> {
    sorted(namespaces.values().flat_map(|namespace| namespace.events.clone()))
}

pub fn accounts(namespaces: &BTreeMap<String, SessionNamespace>) -> Vec<String> {
    namespaces.values().flat_map(|namespace| namespace.accounts.clone()).collect()
}

fn proposal_chains(key: &str, namespace: &ProposalNamespace) -> (String, Vec<String>) {
    match (&namespace.chains, key.split_once(':')) {
        (Some(chains), _) => (key.to_string(), chains.clone()),
        (None, Some((name, _))) => (name.to_string(), vec![key.to_string()]),
        (None, None) => (key.to_string(), vec![]),
    }
}

fn namespace_chains(key: &str, namespace: &SessionNamespace) -> Vec<String> {
    let account_chains = namespace.accounts.iter().filter_map(|account| account.rsplit_once(':').map(|(chain, _)| chain.to_string()));
    let key_chain = key.contains(':').then(|| key.to_string());
    union(namespace.chains.iter().flatten().cloned().chain(account_chains).chain(key_chain))
}

fn accounts_on(accounts: &[String], chains: &[String]) -> Vec<String> {
    accounts
        .iter()
        .filter(|account| account.rsplit_once(':').is_some_and(|(chain, _)| chains.iter().any(|existing| existing == chain)))
        .cloned()
        .collect()
}

fn merge(namespaces: &mut BTreeMap<String, SessionNamespace>, name: String, chains: Vec<String>, accounts: Vec<String>, methods: Vec<String>, events: Vec<String>) {
    let namespace = namespaces.entry(name).or_insert_with(|| SessionNamespace {
        chains: Some(vec![]),
        accounts: vec![],
        methods: vec![],
        events: vec![],
    });
    namespace.chains = Some(union(namespace.chains.iter().flatten().cloned().chain(chains)));
    namespace.accounts = union(namespace.accounts.iter().cloned().chain(accounts));
    namespace.methods = union(namespace.methods.iter().cloned().chain(methods));
    namespace.events = union(namespace.events.iter().cloned().chain(events));
}

fn intersection(values: &[String], supported: &[String]) -> Vec<String> {
    union(values.iter().filter(|value| supported.contains(value)).cloned())
}

fn union(values: impl Iterator<Item = String>) -> Vec<String> {
    let mut unique: Vec<String> = Vec::new();
    for value in values {
        if !unique.contains(&value) {
            unique.push(value);
        }
    }
    unique
}

fn sorted(values: impl Iterator<Item = String>) -> Vec<String> {
    let mut values = union(values);
    values.sort();
    values
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::strings;

    #[test]
    fn test_approved_namespaces() {
        let chains = strings(&["eip155:1", "eip155:137", "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp"]);
        let accounts = strings(&["eip155:1:0xabc", "eip155:137:0xabc", "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp:sol"]);
        let methods = strings(&["eth_sendTransaction", "personal_sign", "solana_signMessage"]);
        let events = strings(&["chainChanged", "accountsChanged"]);
        let supported = Supported {
            chains: &chains,
            accounts: &accounts,
            methods: &methods,
            events: &events,
        };

        assert_eq!(
            approved_namespaces(&SessionProposal::mock(), &supported).unwrap(),
            BTreeMap::from([
                (
                    "eip155".to_string(),
                    SessionNamespace::mock(
                        &["eip155:1", "eip155:137"],
                        &["eip155:1:0xabc", "eip155:137:0xabc"],
                        &["eth_sendTransaction", "personal_sign"],
                        &["chainChanged", "accountsChanged"]
                    )
                ),
                (
                    "solana".to_string(),
                    SessionNamespace::mock(&["solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp"], &["solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp:sol"], &["solana_signMessage"], &[])
                ),
            ]),
            "the required chain plus every supported optional chain, each with the requested methods the wallet supports"
        );

        let ethereum = strings(&["eip155:1"]);
        let ethereum_account = strings(&["eip155:1:0xabc"]);
        let only_ethereum = Supported {
            chains: &ethereum,
            accounts: &ethereum_account,
            methods: &methods,
            events: &events,
        };
        assert_eq!(
            approved_namespaces(&SessionProposal::mock_with_required("eip155", &["eip155:56"], &["personal_sign"], &[]), &only_ethereum),
            Err(NamespaceError::UnsupportedChains)
        );
        assert_eq!(
            approved_namespaces(&SessionProposal::mock_with_required("eip155", &["eip155:1"], &["eth_sign"], &[]), &only_ethereum),
            Err(NamespaceError::UnsupportedMethods)
        );
        assert_eq!(
            approved_namespaces(&SessionProposal::mock_with_required("eip155", &["eip155:1"], &["personal_sign"], &["message"]), &only_ethereum),
            Err(NamespaceError::UnsupportedEvents)
        );
        assert_eq!(
            approved_namespaces(&SessionProposal::mock_with_required("eip155:1", &[], &["personal_sign"], &[]), &only_ethereum).unwrap(),
            BTreeMap::from([("eip155".to_string(), SessionNamespace::mock(&["eip155:1"], &["eip155:1:0xabc"], &["personal_sign"], &[]))]),
            "a CAIP-2 key names its chain"
        );
        let no_accounts = Supported {
            chains: &ethereum,
            accounts: &[],
            methods: &methods,
            events: &events,
        };
        assert_eq!(
            approved_namespaces(&SessionProposal::mock_with_required("eip155", &["eip155:1"], &["personal_sign"], &[]), &no_accounts),
            Err(NamespaceError::UnsupportedAccounts)
        );
        assert_eq!(
            approved_namespaces(&SessionProposal::mock_with_optional("cosmos", &["cosmos:cosmoshub-4"], &["cosmos_signDirect"]), &supported),
            Err(NamespaceError::UnsupportedAccounts),
            "nothing to approve"
        );
    }

    #[test]
    fn test_approved_namespaces_keep_the_requested_tron_reference() {
        let chains = strings(&["tron:0x2b6653dc", "tron:728126428"]);
        let accounts = strings(&["tron:0x2b6653dc:address", "tron:728126428:address"]);
        let methods = strings(&["tron_signMessage"]);
        let supported = Supported {
            chains: &chains,
            accounts: &accounts,
            methods: &methods,
            events: &[],
        };

        for chain in ["tron:728126428", "tron:0x2b6653dc"] {
            let required = approved_namespaces(&SessionProposal::mock_with_required("tron", &[chain], &["tron_signMessage"], &[]), &supported).unwrap();
            let optional = approved_namespaces(&SessionProposal::mock_with_optional("tron", &[chain], &["tron_signMessage"]), &supported).unwrap();
            let expected = SessionNamespace::mock(&[chain], &[&format!("{chain}:address")], &["tron_signMessage"], &[]);

            assert_eq!(required["tron"], expected);
            assert_eq!(optional["tron"], expected);
        }
    }

    #[test]
    fn test_adding_account() {
        let namespaces = BTreeMap::from([
            ("eip155".to_string(), SessionNamespace::mock(&["eip155:1"], &["eip155:1:0xabc"], &["eth_sendTransaction"], &["chainChanged"])),
            (
                "solana".to_string(),
                SessionNamespace::mock(&["solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp"], &["solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp:sol"], &["solana_signMessage"], &[]),
            ),
        ]);

        let updated = adding_account(&namespaces, "eip155:81457:0xabc");

        assert_eq!(
            updated["eip155"],
            SessionNamespace::mock(&["eip155:1", "eip155:81457"], &["eip155:1:0xabc", "eip155:81457:0xabc"], &["eth_sendTransaction"], &["chainChanged"])
        );
        assert_eq!(updated["solana"], namespaces["solana"]);
        assert_eq!(adding_account(&updated, "eip155:81457:0xabc"), updated, "an account already in the session is not added twice");
        assert_eq!(adding_account(&namespaces, "cosmos:cosmoshub-4:addr"), namespaces);
    }

    #[test]
    fn test_authorization() {
        let namespaces = BTreeMap::from([("eip155".to_string(), SessionNamespace::mock(&["eip155:1"], &["eip155:1:0xabc", "eip155:10:0xabc"], &["personal_sign"], &[]))]);

        assert_eq!(authorization(&namespaces, "eip155:1", "personal_sign"), Ok(()));
        assert_eq!(authorization(&namespaces, "eip155:10", "personal_sign"), Ok(()), "an account's chain is part of the session");
        assert_eq!(authorization(&namespaces, "eip155:56", "personal_sign"), Err(Reason::unauthorized_chain()));
        assert_eq!(authorization(&namespaces, "eip155:1", "eth_sendTransaction"), Err(Reason::unauthorized_method("eth_sendTransaction")));
        assert_eq!(approved_chains(&namespaces), strings(&["eip155:1", "eip155:10"]));
        assert_eq!(approved_methods(&namespaces), strings(&["personal_sign"]));
    }
}
