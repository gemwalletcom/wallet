use gem_hash::sha2::sha256;
use primitives::{ChainSigner, SignerError, SignerInput};
use serde::{Deserialize, Serialize};
use signer::{SignatureScheme, Signer};

use crate::error::LightningError;
use crate::invoice::decode_invoice;
use crate::model::LightningTransfer;

const AUTHORIZATION_DOMAIN: &str = "gem-lightning-transfer-v1";

#[derive(Debug, Serialize, Deserialize)]
pub struct SignedLightningTransfer {
    pub transfer: LightningTransfer,
    #[serde(with = "serde_serializers::hex_bytes")]
    pub signature: Vec<u8>,
}

impl SignedLightningTransfer {
    pub fn decode(value: &str) -> Result<Self, LightningError> {
        serde_json::from_slice(value.as_bytes()).map_err(|_| LightningError::InvalidTransfer)
    }

    pub fn digest(&self) -> [u8; 32] {
        digest(&self.transfer)
    }
}

fn digest(transfer: &LightningTransfer) -> [u8; 32] {
    sha256(&[AUTHORIZATION_DOMAIN.as_bytes(), &serde_json::to_vec(transfer).unwrap_or_default()].concat())
}

fn sign(transfer: LightningTransfer, private_key: &[u8]) -> Result<String, SignerError> {
    let signature = Signer::sign_digest(SignatureScheme::Secp256k1, &digest(&transfer), private_key)?;
    Ok(serde_json::to_string(&SignedLightningTransfer { transfer, signature })?)
}

pub struct LightningChainSigner;

impl ChainSigner for LightningChainSigner {
    fn sign_transfer(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        let transfer = LightningTransfer {
            address: input.sender_address.clone(),
            invoice: decode_invoice(&input.destination_address).map_err(SignerError::from_display)?.invoice,
            amount_sats: input.value_as_u64()?,
            max_fee_sats: input.fee.fee()?,
        };
        sign(transfer, private_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::INVOICE_2000_SATS;

    #[test]
    fn test_signed_lightning_transfer() {
        let transfer = LightningTransfer {
            address: "spark1address".to_string(),
            invoice: INVOICE_2000_SATS.to_string(),
            amount_sats: 2_000,
            max_fee_sats: 7,
        };

        let signed = SignedLightningTransfer::decode(&sign(transfer.clone(), &[7u8; 32]).unwrap()).unwrap();

        assert_eq!(signed.transfer, transfer);
        assert_eq!(signed.digest(), digest(&transfer));
        assert_eq!(signed.signature.len(), 65);
        assert!(SignedLightningTransfer::decode("{}").is_err());
    }
}
