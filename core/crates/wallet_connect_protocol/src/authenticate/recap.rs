use gem_encoding::{decode_base64_no_pad, decode_base64_url, encode_base64};
use serde_json::{Map, Value};

use crate::ProtocolError;
use crate::authenticate::types::AuthPayload;

const RECAP_PREFIX: &str = "urn:recap:";
const RECAP_STATEMENT: &str = "I further authorize the stated URI to perform the following actions on my behalf";
const EIP155: &str = "eip155";
const REQUEST_ACTION: &str = "request/";

pub fn narrowed_payload(payload: &AuthPayload, supported_chains: &[String], supported_methods: &[String]) -> Result<AuthPayload, ProtocolError> {
    let resources = payload.resources.clone().unwrap_or_default();
    let Some(mut recap) = recap(&resources) else {
        return Ok(payload.clone());
    };
    let Some(actions) = recap.get("att").and_then(|att| att.get(EIP155)).and_then(Value::as_object) else {
        return Ok(payload.clone());
    };
    let chains: Vec<String> = payload.chains.iter().filter(|chain| supported_chains.contains(chain)).cloned().collect();
    let methods: Vec<String> = request_methods(actions).into_iter().filter(|method| supported_methods.contains(method)).collect();
    if chains.is_empty() {
        return Err(ProtocolError::InvalidMessage("unsupported chains".to_string()));
    }
    if methods.is_empty() {
        return Err(ProtocolError::InvalidMessage("unsupported methods".to_string()));
    }
    let narrowed: Map<String, Value> = methods
        .iter()
        .map(|method| (format!("{REQUEST_ACTION}{method}"), Value::Array(vec![Value::Object(Map::from_iter([("chains".to_string(), Value::from(chains.clone()))]))])))
        .collect();
    if let Some(att) = recap.get_mut("att").and_then(Value::as_object_mut) {
        att.insert(EIP155.to_string(), Value::Object(narrowed));
    }
    let resources = [&resources[..resources.len() - 1], &[encode(&recap)]].concat();
    Ok(AuthPayload {
        chains,
        statement: statement(payload.statement.as_deref(), &resources),
        resources: Some(resources),
        ..payload.clone()
    })
}

pub fn recap_chains_and_methods(resources: &[String]) -> Option<(Vec<String>, Vec<String>)> {
    let recap = recap(resources)?;
    let actions = recap.get("att")?.get(EIP155)?.as_object()?;
    let chains = actions
        .values()
        .next()
        .and_then(|values| values.get(0))
        .and_then(|value| value.get("chains"))
        .and_then(Value::as_array)
        .map(|chains| chains.iter().filter_map(Value::as_str).map(str::to_string).collect())
        .unwrap_or_default();
    Some((chains, request_methods(actions)))
}

fn statement(statement: Option<&str>, resources: &[String]) -> Option<String> {
    if let Some(statement) = statement
        && statement.contains(RECAP_STATEMENT)
    {
        return Some(statement.to_string());
    }
    let recap = recap(resources).and_then(|recap| actions_statement(&recap));
    match (statement.filter(|statement| !statement.is_empty()), recap) {
        (Some(statement), Some(recap)) => Some(format!("{statement} {recap}")),
        (None, Some(recap)) => Some(recap),
        (statement, None) => statement.map(str::to_string),
    }
}

fn actions_statement(recap: &Value) -> Option<String> {
    let att = recap.get("att")?.as_object()?;
    let mut resources: Vec<(&String, &Value)> = att.iter().collect();
    resources.sort_by(|left, right| left.0.cmp(right.0));
    let mut counter = 0;
    let mut parts = Vec::new();
    for (resource, actions) in resources {
        let Some(actions) = actions.as_object() else { continue };
        let mut grouped: Vec<(String, Vec<String>)> = Vec::new();
        for key in actions.keys() {
            let Some((ability, action)) = key.split_once('/') else { continue };
            match grouped.iter_mut().find(|(existing, _)| existing == ability) {
                Some((_, actions)) => actions.push(action.to_string()),
                None => grouped.push((ability.to_string(), vec![action.to_string()])),
            }
        }
        grouped.sort_by(|left, right| left.0.cmp(&right.0));
        for (ability, mut actions) in grouped {
            actions.sort();
            counter += 1;
            let actions = actions.iter().map(|action| format!("'{action}'")).collect::<Vec<_>>().join(", ");
            parts.push(format!("({counter}) '{ability}': {actions} for '{resource}'"));
        }
    }
    (!parts.is_empty()).then(|| format!("{RECAP_STATEMENT}: {}.", parts.join(". ")))
}

fn request_methods(actions: &Map<String, Value>) -> Vec<String> {
    actions.keys().filter_map(|key| key.strip_prefix(REQUEST_ACTION)).map(str::to_string).collect()
}

fn recap(resources: &[String]) -> Option<Value> {
    resources.last().and_then(|resource| decode(resource))
}

fn decode(resource: &str) -> Option<Value> {
    let encoded = resource.strip_prefix(RECAP_PREFIX)?;
    let bytes = decode_base64_no_pad(encoded.trim_end_matches('=')).or_else(|_| decode_base64_url(encoded.trim_end_matches('='))).ok()?;
    serde_json::from_slice(&bytes).ok().filter(Value::is_object)
}

fn encode(recap: &Value) -> String {
    format!("{RECAP_PREFIX}{}", encode_base64(&serde_json::to_vec(recap).unwrap_or_default()).trim_end_matches('='))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::strings;

    #[test]
    fn test_narrowed_payload() {
        let payload = AuthPayload::mock();

        let narrowed = narrowed_payload(&payload, &strings(&["eip155:1"]), &strings(&["personal_sign", "eth_sendTransaction"])).unwrap();

        assert_eq!(narrowed.chains, strings(&["eip155:1"]));
        assert_eq!(
            narrowed.statement.as_deref(),
            Some("Sign in to Example. I further authorize the stated URI to perform the following actions on my behalf: (1) 'request': 'eth_sendTransaction', 'personal_sign' for 'eip155'.")
        );
        let resources = narrowed.resources.clone().unwrap();
        assert_eq!(resources[0], "ipfs://bafybeiemxf5abjwjbikoz4mc3a3dla6ual3jsgpdr4cjr3oz3evfyavhwq/");
        assert_eq!(
            decode(&resources[1]).unwrap(),
            serde_json::from_str::<Value>(r#"{"att":{"eip155":{"request/eth_sendTransaction":[{"chains":["eip155:1"]}],"request/personal_sign":[{"chains":["eip155:1"]}]}}}"#).unwrap(),
            "only the methods and chains the wallet supports stay authorized"
        );
        assert_eq!(recap_chains_and_methods(&resources), Some((strings(&["eip155:1"]), strings(&["personal_sign", "eth_sendTransaction"]))));
        assert_eq!(
            narrowed_payload(&narrowed, &strings(&["eip155:1"]), &strings(&["personal_sign", "eth_sendTransaction"])).unwrap().statement,
            narrowed.statement,
            "a statement that already names the recap is kept"
        );

        assert_eq!(
            narrowed_payload(&payload, &strings(&["eip155:56"]), &strings(&["personal_sign"])),
            Err(ProtocolError::InvalidMessage("unsupported chains".to_string()))
        );
        assert_eq!(
            narrowed_payload(&payload, &strings(&["eip155:1"]), &strings(&["eth_signTypedData_v4"])),
            Err(ProtocolError::InvalidMessage("unsupported methods".to_string()))
        );

        let without_recap = AuthPayload { resources: None, ..payload };
        assert_eq!(narrowed_payload(&without_recap, &strings(&["eip155:1"]), &strings(&["personal_sign"])).unwrap(), without_recap);
    }
}
