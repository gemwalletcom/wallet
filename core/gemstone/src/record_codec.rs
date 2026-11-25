use primitives::PaymentLink;

use crate::GemstoneError;
use crate::payment::GemPaymentRecipient;
use crate::services::perpetual::GemPerpetualPositionAction;
use crate::services::stake::model::GemStakeAmountInput;
use crate::services::transfer::{GemRecipient, GemTransferData};

fn encode<T: uniffi::Lower<crate::UniFfiTag>>(value: T) -> Vec<u8> {
    let mut buffer = Vec::new();
    T::write(value, &mut buffer);
    buffer
}

fn decode<T: uniffi::Lift<crate::UniFfiTag>>(bytes: Vec<u8>) -> Result<T, GemstoneError> {
    let mut buffer = bytes.as_slice();
    let value = T::try_read(&mut buffer).map_err(|error| GemstoneError::from(error.to_string()))?;
    if !buffer.is_empty() {
        return Err("record payload has trailing bytes".into());
    }
    Ok(value)
}

#[uniffi::export]
pub fn encode_gem_recipient(value: GemRecipient) -> Vec<u8> {
    encode(value)
}

#[uniffi::export]
pub fn decode_gem_recipient(bytes: Vec<u8>) -> Result<GemRecipient, GemstoneError> {
    decode(bytes)
}

#[uniffi::export]
pub fn encode_gem_payment_recipient(value: GemPaymentRecipient) -> Vec<u8> {
    encode(value)
}

#[uniffi::export]
pub fn decode_gem_payment_recipient(bytes: Vec<u8>) -> Result<GemPaymentRecipient, GemstoneError> {
    decode(bytes)
}

#[uniffi::export]
pub fn encode_gem_perpetual_position_action(value: GemPerpetualPositionAction) -> Vec<u8> {
    encode(value)
}

#[uniffi::export]
pub fn decode_gem_perpetual_position_action(bytes: Vec<u8>) -> Result<GemPerpetualPositionAction, GemstoneError> {
    decode(bytes)
}

#[uniffi::export]
pub fn encode_gem_transfer_data(value: GemTransferData) -> Vec<u8> {
    encode(value)
}

#[uniffi::export]
pub fn decode_gem_transfer_data(bytes: Vec<u8>) -> Result<GemTransferData, GemstoneError> {
    decode(bytes)
}

#[uniffi::export]
pub fn encode_gem_stake_amount_input(value: GemStakeAmountInput) -> Vec<u8> {
    encode(value)
}

#[uniffi::export]
pub fn decode_gem_stake_amount_input(bytes: Vec<u8>) -> Result<GemStakeAmountInput, GemstoneError> {
    decode(bytes)
}

#[uniffi::export]
pub fn encode_payment_link(value: PaymentLink) -> Vec<u8> {
    encode(value)
}

#[uniffi::export]
pub fn decode_payment_link(bytes: Vec<u8>) -> Result<PaymentLink, GemstoneError> {
    decode(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a_recipient_round_trips_and_trailing_bytes_are_rejected() {
        let recipient = GemRecipient {
            address: "0x1".to_string(),
            name: Some("Gem".to_string()),
            memo: None,
            references: vec!["ref".to_string()],
        };
        let mut bytes = encode_gem_recipient(recipient.clone());

        assert_eq!(decode_gem_recipient(bytes.clone()).unwrap(), recipient);

        bytes.push(0);
        assert!(decode_gem_recipient(bytes).is_err());
        assert!(decode_gem_recipient(vec![1, 2, 3]).is_err());
    }
}
