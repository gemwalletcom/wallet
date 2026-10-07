use crate::GemstoneError;
use primitives::Chain;
use zeroize::Zeroizing;

pub fn decode_private_key(chain: Chain, value: String) -> Result<Vec<u8>, GemstoneError> {
    let value = Zeroizing::new(value);
    let mut private_key = signer::decode_private_key(&chain, &value)?;
    Ok(std::mem::take(private_key.as_mut()))
}
