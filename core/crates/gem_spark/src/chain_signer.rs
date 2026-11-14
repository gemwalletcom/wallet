use gem_hash::sha2::sha256;
use primitives::{ChainSigner, SignerError, SignerInput};
use serde::{Deserialize, Serialize};
use signer::{SignatureScheme, Signer};

use crate::error::LightningError;
use crate::invoice::decode_invoice;
use crate::model::LightningSendQuote;

const AUTHORIZATION_DOMAIN: &str = "gem-lightning-transfer-v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LightningAuthorization {
    pub digest: [u8; 32],
    pub signature: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LightningTransfer {
    pub address: String,
    pub invoice: String,
    pub amount_sats: u64,
    pub max_fee_sats: u64,
}

#[derive(Serialize, Deserialize)]
struct SignedLightningTransfer {
    transfer: LightningTransfer,
    #[serde(with = "serde_serializers::hex_bytes")]
    signature: Vec<u8>,
}

impl LightningTransfer {
    fn digest(&self) -> [u8; 32] {
        sha256(&[AUTHORIZATION_DOMAIN.as_bytes(), &serde_json::to_vec(self).unwrap_or_default()].concat())
    }

    fn sign(self, private_key: &[u8]) -> Result<String, SignerError> {
        let signature = Signer::sign_digest(SignatureScheme::Secp256k1, &self.digest(), private_key)?;
        Ok(serde_json::to_string(&SignedLightningTransfer { transfer: self, signature })?)
    }

    pub fn decode(value: &str) -> Result<(Self, LightningAuthorization), LightningError> {
        let signed: SignedLightningTransfer = serde_json::from_slice(value.as_bytes()).map_err(|_| LightningError::InvalidTransfer)?;
        let authorization = LightningAuthorization {
            digest: signed.transfer.digest(),
            signature: signed.signature,
        };
        Ok((signed.transfer, authorization))
    }

    pub(crate) fn quote(&self) -> Result<LightningSendQuote, LightningError> {
        Ok(LightningSendQuote {
            invoice: decode_invoice(&self.invoice)?,
            amount_sats: self.amount_sats,
            fee_sats: self.max_fee_sats,
        })
    }
}

pub struct LightningChainSigner;

impl ChainSigner for LightningChainSigner {
    fn sign_transfer(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        LightningTransfer {
            address: input.sender_address.clone(),
            invoice: decode_invoice(&input.destination_address).map_err(SignerError::from_display)?.invoice,
            amount_sats: input.value_as_u64()?,
            max_fee_sats: input.fee.fee()?,
        }
        .sign(private_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::INVOICE_2000_SATS;

    #[test]
    fn test_lightning_transfer_round_trip() {
        let transfer = LightningTransfer {
            address: "spark1address".to_string(),
            invoice: INVOICE_2000_SATS.to_string(),
            amount_sats: 2_000,
            max_fee_sats: 7,
        };

        let (decoded, authorization) = LightningTransfer::decode(&transfer.clone().sign(&[7u8; 32]).unwrap()).unwrap();

        assert_eq!(decoded, transfer);
        assert_eq!(authorization.digest, transfer.digest());
        assert_eq!(authorization.signature.len(), 65);
        assert_eq!(LightningTransfer::decode("{}"), Err(LightningError::InvalidTransfer));
    }
}
