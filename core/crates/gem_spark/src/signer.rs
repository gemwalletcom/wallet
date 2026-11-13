use std::sync::{Arc, PoisonError, RwLock};

use async_trait::async_trait;
use bitcoin::bip32::DerivationPath;
use bitcoin::secp256k1::ecdsa::Signature;
use bitcoin::secp256k1::{PublicKey, SecretKey, schnorr};
use bitcoin::taproot::TapNodeHash;
use frost_secp256k1_tr::round2::SignatureShare;
use spark_wallet::{DefaultSigner, EncryptedSecret, FrostSigningCommitmentsWithNonces, SecretSource, SecretToSplit, SignFrostRequest, Signer, SignerError, VerifiableSecretShare};

use crate::error::LightningError;
use crate::keys::AccountKey;

pub struct LockableSigner {
    signer: RwLock<Option<Arc<DefaultSigner>>>,
}

impl LockableSigner {
    pub fn new(account: &AccountKey) -> Self {
        Self {
            signer: RwLock::new(Some(Arc::new(DefaultSigner::from_master(account.xpriv())))),
        }
    }

    pub fn lock(&self) {
        *self.signer.write().unwrap_or_else(PoisonError::into_inner) = None;
    }

    #[cfg(test)]
    pub(crate) fn is_locked(&self) -> bool {
        self.signer.read().unwrap_or_else(PoisonError::into_inner).is_none()
    }

    fn signer(&self) -> Result<Arc<DefaultSigner>, SignerError> {
        self.signer.read().unwrap_or_else(PoisonError::into_inner).clone().ok_or_else(|| SignerError::Generic(LightningError::Locked.to_string()))
    }
}

#[async_trait]
impl Signer for LockableSigner {
    async fn derive_public_key(&self, path: &DerivationPath) -> Result<PublicKey, SignerError> {
        self.signer()?.derive_public_key(path).await
    }

    async fn secret_key(&self, path: &DerivationPath) -> Result<SecretKey, SignerError> {
        self.signer()?.secret_key(path).await
    }

    async fn sign_message_ecdsa(&self, path: &DerivationPath, message: &[u8]) -> Result<Signature, SignerError> {
        self.signer()?.sign_message_ecdsa(path, message).await
    }

    async fn sign_hash_schnorr(&self, path: &DerivationPath, hash: &[u8]) -> Result<schnorr::Signature, SignerError> {
        self.signer()?.sign_hash_schnorr(path, hash).await
    }

    async fn sign_hash_schnorr_with_tweak(&self, secret: &SecretSource, hash: &[u8], tap_tweak: Option<TapNodeHash>) -> Result<schnorr::Signature, SignerError> {
        self.signer()?.sign_hash_schnorr_with_tweak(secret, hash, tap_tweak).await
    }

    async fn generate_random_signing_commitment(&self) -> Result<FrostSigningCommitmentsWithNonces, SignerError> {
        self.signer()?.generate_random_signing_commitment().await
    }

    async fn generate_random_secret(&self) -> Result<EncryptedSecret, SignerError> {
        self.signer()?.generate_random_secret().await
    }

    async fn subtract_secrets(&self, signing_key: &SecretSource, new_signing_key: &SecretSource) -> Result<SecretSource, SignerError> {
        self.signer()?.subtract_secrets(signing_key, new_signing_key).await
    }

    async fn split_secret_with_proofs(&self, secret: &SecretToSplit, threshold: u32, num_shares: usize) -> Result<Vec<VerifiableSecretShare>, SignerError> {
        self.signer()?.split_secret_with_proofs(secret, threshold, num_shares).await
    }

    async fn encrypt_secret_for_receiver(&self, secret: &SecretSource, receiver_public_key: &PublicKey) -> Result<Vec<u8>, SignerError> {
        self.signer()?.encrypt_secret_for_receiver(secret, receiver_public_key).await
    }

    async fn public_key_from_secret(&self, private_key: &SecretSource) -> Result<PublicKey, SignerError> {
        self.signer()?.public_key_from_secret(private_key).await
    }

    async fn sign_frost<'a>(&self, request: SignFrostRequest<'a>) -> Result<SignatureShare, SignerError> {
        self.signer()?.sign_frost(request).await
    }

    async fn split_signing_key(&self, parent_key: &SecretSource, n: usize) -> Result<Vec<(SecretSource, PublicKey)>, SignerError> {
        self.signer()?.split_signing_key(parent_key, n).await
    }

    async fn combine_signing_keys(&self, keys: &[SecretSource]) -> Result<(SecretSource, PublicKey), SignerError> {
        self.signer()?.combine_signing_keys(keys).await
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use primitives::hex::decode_hex;

    use super::*;
    use crate::keys::account_key;
    use crate::testkit::{ABANDON_SEED_HEX, IDENTITY_PUBLIC_KEY};

    #[tokio::test]
    async fn test_lock() {
        let signer = LockableSigner::new(&account_key(&decode_hex(ABANDON_SEED_HEX).unwrap()).unwrap());
        let identity = DerivationPath::from_str("m/0'").unwrap();

        assert_eq!(signer.derive_public_key(&identity).await.unwrap().to_string(), IDENTITY_PUBLIC_KEY);

        signer.lock();

        assert_eq!(signer.derive_public_key(&identity).await.err().map(|error| error.to_string()), Some(format!("generic error: {}", LightningError::Locked)));
        assert!(signer.sign_message_ecdsa(&identity, b"gem").await.is_err());
    }
}
