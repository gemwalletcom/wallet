use gem_ton::models::TonConnectMessage;
use primitives::Chain;

use crate::{Deployment, SwapperError};

pub(super) fn verify_ton_messages(messages: &str, contracts: &[&str]) -> Result<(), SwapperError> {
    let messages: Vec<TonConnectMessage> = serde_json::from_str(messages).map_err(SwapperError::transaction_error)?;
    if !messages.is_empty() && messages.iter().all(|message| Deployment::contains(Chain::Ton, contracts, message.address())) {
        Ok(())
    } else {
        Err(SwapperError::InvalidRoute)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::BigUint;

    #[test]
    fn test_verify_ton_messages() {
        let depository = "EQCrdGsDTqA2t6xRR4N6V4J705F7w_VQbUdHnofsh-8lVIPs";
        let messages = |addresses: &[&str]| serde_json::to_string(&addresses.iter().map(|address| TonConnectMessage::new(address.to_string(), &BigUint::from(1u64), None)).collect::<Vec<_>>()).unwrap();

        assert!(verify_ton_messages(&messages(&[depository]), &[depository]).is_ok());
        assert!(verify_ton_messages(&messages(&[depository, depository]), &[depository]).is_ok());
        assert_eq!(
            verify_ton_messages(&messages(&[depository, "EQBvW8Z5huBkMJYdnfAEM5JqTNkuWX3diqYENkWsIL0XggGG"]), &[depository]),
            Err(SwapperError::InvalidRoute),
            "every message must go to a shipped contract"
        );
        assert_eq!(verify_ton_messages("[]", &[depository]), Err(SwapperError::InvalidRoute));
    }
}
