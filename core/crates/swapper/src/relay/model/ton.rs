use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TonStepData {
    pub messages: Vec<TonMessage>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TonMessage {
    pub to: String,
    pub value: String,
    pub body: String,
}

#[cfg(test)]
mod tests {
    use crate::relay::model::RelayQuoteResponse;

    #[test]
    fn test_ton_step() {
        let response: RelayQuoteResponse = serde_json::from_str(include_str!("../testdata/quote_ton_to_base_usdc.json")).unwrap();
        let ton = response.get_ton_step().unwrap();

        assert_eq!(ton.messages.len(), 1);
        assert_eq!(ton.messages[0].to, "EQCrdGsDTqA2t6xRR4N6V4J705F7w_VQbUdHnofsh-8lVIPs");
        assert_eq!(ton.messages[0].value, "5000000000");
        assert_eq!(response.router_address().as_deref(), Some("EQCrdGsDTqA2t6xRR4N6V4J705F7w_VQbUdHnofsh-8lVIPs"));
        assert!(response.get_evm_step().is_none());
        assert!(response.get_solana_step().is_none());
    }
}
