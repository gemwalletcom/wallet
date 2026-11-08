use std::collections::BTreeMap;

use crate::ProtocolError;
use crate::authenticate::recap::recap_chains_and_methods;
use crate::authenticate::types::{AuthPayload, Cacao, CacaoHeader, CacaoPayload, CacaoSignature, Issuer};
use crate::sign::types::SessionNamespace;

const CACAO_TYPE: &str = "caip122";
const SIGNATURE_TYPE: &str = "eip191";
const SESSION_EVENTS: [&str; 2] = ["chainChanged", "accountsChanged"];
const EIP155: &str = "eip155";

pub fn message(payload: &AuthPayload, issuer: &Issuer) -> Result<String, ProtocolError> {
    if !payload.chains.contains(&issuer.chain_id) {
        return Err(ProtocolError::InvalidMessage("issuer chain is not requested".to_string()));
    }
    let mut lines = vec![format!("{} wants you to sign in with your Ethereum account:", payload.domain), issuer.address.clone(), String::new()];
    if let Some(statement) = payload.statement.as_ref().filter(|statement| !statement.is_empty()) {
        lines.push(statement.clone());
    }
    lines.extend([
        String::new(),
        format!("URI: {}", payload.aud),
        format!("Version: {}", payload.version),
        format!("Chain ID: {}", issuer.reference()),
        format!("Nonce: {}", payload.nonce),
        format!("Issued At: {}", payload.iat),
    ]);
    if let Some(exp) = &payload.exp {
        lines.push(format!("Expiration Time: {exp}"));
    }
    if let Some(nbf) = &payload.nbf {
        lines.push(format!("Not Before: {nbf}"));
    }
    if let Some(request_id) = &payload.request_id {
        lines.push(format!("Request ID: {request_id}"));
    }
    if let Some(resources) = &payload.resources {
        lines.push("Resources:".to_string());
        lines.extend(resources.iter().map(|resource| format!("- {resource}")));
    }
    Ok(lines.join("\n"))
}

pub fn cacao(payload: &AuthPayload, issuer: &Issuer, signature: String) -> Cacao {
    Cacao {
        h: CacaoHeader { t: CACAO_TYPE },
        p: CacaoPayload {
            iss: issuer.did(),
            domain: payload.domain.clone(),
            aud: payload.aud.clone(),
            version: payload.version.clone(),
            nonce: payload.nonce.clone(),
            iat: payload.iat.clone(),
            nbf: payload.nbf.clone(),
            exp: payload.exp.clone(),
            statement: payload.statement.clone(),
            request_id: payload.request_id.clone(),
            resources: payload.resources.clone(),
        },
        s: CacaoSignature { t: SIGNATURE_TYPE, s: signature },
    }
}

pub fn session_namespaces(payload: &AuthPayload, issuer: &Issuer) -> Option<BTreeMap<String, SessionNamespace>> {
    let (chains, methods) = recap_chains_and_methods(payload.resources.as_deref().unwrap_or_default())?;
    if methods.is_empty() {
        return None;
    }
    let chains = if chains.is_empty() { payload.chains.clone() } else { chains };
    let accounts = chains.iter().map(|chain| format!("{chain}:{}", issuer.address)).collect();
    Some(BTreeMap::from([(
        EIP155.to_string(),
        SessionNamespace {
            chains: Some(chains),
            accounts,
            methods,
            events: SESSION_EVENTS.iter().map(ToString::to_string).collect(),
        },
    )]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authenticate::recap::narrowed_payload;
    use crate::testkit::strings;

    #[test]
    fn test_message() {
        let issuer = Issuer::new("eip155:1", "0x1111111111111111111111111111111111111111");
        let payload = narrowed_payload(&AuthPayload::mock(), &strings(&["eip155:1"]), &strings(&["personal_sign"])).unwrap();

        assert_eq!(message(&payload, &issuer).unwrap(), include_str!("../../testdata/authenticate_message.txt").trim_end());
        assert_eq!(
            message(
                &AuthPayload {
                    statement: None,
                    resources: None,
                    ..payload.clone()
                },
                &issuer
            )
            .unwrap(),
            "example.com wants you to sign in with your Ethereum account:\n0x1111111111111111111111111111111111111111\n\n\nURI: https://example.com/login\nVersion: 1\nChain ID: 1\nNonce: 32891756\nIssued At: 2024-01-29T08:54:38Z\nExpiration Time: 2024-01-29T09:54:38Z"
        );
        assert_eq!(
            message(
                &AuthPayload {
                    statement: Some(String::new()),
                    resources: None,
                    ..payload.clone()
                },
                &issuer
            )
            .unwrap(),
            message(
                &AuthPayload {
                    statement: None,
                    resources: None,
                    ..payload.clone()
                },
                &issuer
            )
            .unwrap(),
            "an empty statement reads as none, as the dapp's verifier rebuilds it"
        );
        assert_eq!(
            message(&payload, &Issuer::new("eip155:56", "0x1111111111111111111111111111111111111111")),
            Err(ProtocolError::InvalidMessage("issuer chain is not requested".to_string()))
        );
    }

    #[test]
    fn test_cacao() {
        let issuer = Issuer::new("eip155:1", "0x1111111111111111111111111111111111111111");
        let payload = AuthPayload { resources: None, ..AuthPayload::mock() };

        assert_eq!(
            serde_json::to_string(&cacao(&payload, &issuer, "0xsignature".to_string())).unwrap(),
            r#"{"h":{"t":"caip122"},"p":{"iss":"did:pkh:eip155:1:0x1111111111111111111111111111111111111111","domain":"example.com","aud":"https://example.com/login","version":"1","nonce":"32891756","iat":"2024-01-29T08:54:38Z","exp":"2024-01-29T09:54:38Z","statement":"Sign in to Example."},"s":{"t":"eip191","s":"0xsignature"}}"#
        );
    }

    #[test]
    fn test_session_namespaces() {
        let issuer = Issuer::new("eip155:1", "0xabc");
        let payload = narrowed_payload(&AuthPayload::mock(), &strings(&["eip155:1"]), &strings(&["personal_sign"])).unwrap();

        assert_eq!(
            session_namespaces(&payload, &issuer),
            Some(BTreeMap::from([(
                "eip155".to_string(),
                SessionNamespace::mock(&["eip155:1"], &["eip155:1:0xabc"], &["personal_sign"], &["chainChanged", "accountsChanged"])
            )]))
        );
        assert_eq!(session_namespaces(&AuthPayload { resources: None, ..payload }, &issuer), None, "a sign-in without a recap opens no session");
    }
}
