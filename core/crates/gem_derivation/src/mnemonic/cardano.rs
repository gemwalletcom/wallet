use gem_crypto::{hash::hmac_sha512, pbkdf::pbkdf2_hmac_sha512};
use zeroize::{Zeroize, Zeroizing};

use crate::{AccountDerivationError, read_array};

use super::path::{HARDENED_OFFSET, component_number, parse_derivation_path};

const ICARUS_PBKDF2_ROUNDS: usize = 4096;
const SECRET_LEN: usize = 32;
const EXTENDED_SECRET_LEN: usize = 96;
const PRIVATE_KEY_LEN: usize = EXTENDED_SECRET_LEN * 2;
const PAYMENT_PATH: &str = "m/1852'/1815'/0'/0/0";
const STAKE_PATH: &str = "m/1852'/1815'/0'/2/0";

#[derive(Clone)]
struct CardanoNode {
    secret: [u8; SECRET_LEN],
    extension: [u8; SECRET_LEN],
    chain_code: [u8; SECRET_LEN],
}

impl Drop for CardanoNode {
    fn drop(&mut self) {
        self.secret.zeroize();
        self.extension.zeroize();
        self.chain_code.zeroize();
    }
}

pub(super) fn derive_private_key(entropy: &[u8]) -> Result<Zeroizing<Vec<u8>>, AccountDerivationError> {
    let root = root_from_entropy(entropy)?;
    let payment = derive_path(root.clone(), PAYMENT_PATH)?;
    let stake = derive_path(root, STAKE_PATH)?;

    let mut private_key = Vec::with_capacity(PRIVATE_KEY_LEN);
    extend_node(&mut private_key, &payment);
    extend_node(&mut private_key, &stake);
    Ok(Zeroizing::new(private_key))
}

fn root_from_entropy(entropy: &[u8]) -> Result<CardanoNode, AccountDerivationError> {
    let mut secret = pbkdf2_hmac_sha512(b"", entropy, ICARUS_PBKDF2_ROUNDS, EXTENDED_SECRET_LEN)?;
    // Clamp the root scalar (BIP32-Ed25519 / Cardano Icarus): clear the low 3 bits and the top 2 bits, set bit 6.
    secret[0] &= 0xf8;
    secret[SECRET_LEN - 1] &= 0x1f;
    secret[SECRET_LEN - 1] |= 0x40;

    let node = CardanoNode {
        secret: read_array(&secret, 0),
        extension: read_array(&secret, 32),
        chain_code: read_array(&secret, 64),
    };
    secret.zeroize();
    Ok(node)
}

fn derive_path(mut node: CardanoNode, path: &str) -> Result<CardanoNode, AccountDerivationError> {
    let components = parse_derivation_path(path).ok_or_else(|| AccountDerivationError::invalid_input(format!("invalid derivation path: {path}")))?;
    for component in components {
        node = derive_child(node, component_number(component))?;
    }
    Ok(node)
}

fn derive_child(node: CardanoNode, index: u32) -> Result<CardanoNode, AccountDerivationError> {
    let hardened = index & HARDENED_OFFSET != 0;
    let key_size = if hardened { 64 } else { 32 };
    let mut data = Zeroizing::new(vec![0u8; 1 + key_size + 4]);
    data[1 + key_size..].copy_from_slice(&index.to_le_bytes());

    if hardened {
        data[0] = 0;
        data[1..33].copy_from_slice(&node.secret);
        data[33..65].copy_from_slice(&node.extension);
    } else {
        data[0] = 2;
        data[1..33].copy_from_slice(&gem_cardano::public_key_from_extended_secret(node.secret));
    }

    let z = hmac_sha512(&node.chain_code, data.as_slice())?;
    let mut zl8 = [0u8; SECRET_LEN];
    scalar_multiply8(&z[..28], &mut zl8);

    let mut secret = [0u8; SECRET_LEN];
    scalar_add_256bits(&zl8, &node.secret, &mut secret);

    let mut extension = [0u8; SECRET_LEN];
    scalar_add_256bits(&z[32..64], &node.extension, &mut extension);

    data[0] = if hardened { 1 } else { 3 };
    let z = hmac_sha512(&node.chain_code, data.as_slice())?;
    let chain_code = read_array(z.as_slice(), 32);
    zl8.zeroize();

    Ok(CardanoNode { secret, extension, chain_code })
}

fn scalar_multiply8(src: &[u8], dst: &mut [u8; SECRET_LEN]) {
    let mut previous = 0u8;
    for (index, value) in src.iter().enumerate() {
        dst[index] = (value << 3) + (previous & 0x07);
        previous = value >> 5;
    }
    dst[src.len()] = src[src.len() - 1] >> 5;
}

fn scalar_add_256bits(left: &[u8], right: &[u8], output: &mut [u8; SECRET_LEN]) {
    let mut carry = 0u16;
    for index in 0..SECRET_LEN {
        carry += u16::from(left[index]) + u16::from(right[index]);
        output[index] = (carry & 0xff) as u8;
        carry >>= 8;
    }
}

fn extend_node(bytes: &mut Vec<u8>, node: &CardanoNode) {
    bytes.extend_from_slice(&node.secret);
    bytes.extend_from_slice(&node.extension);
    bytes.extend_from_slice(&node.chain_code);
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{decode_hex, hex};

    #[test]
    fn test_cardano_private_key_vector() {
        let entropy = decode_hex("30a6f50aeb58ff7699b822d63e0ef27aeff17d9f").unwrap();
        let private_key = derive_private_key(&entropy).unwrap();

        assert_eq!(
            hex::encode(private_key.as_slice()),
            "e8c8c5b2df13f3abed4e6b1609c808e08ff959d7e6fc3d849e3f2880550b5744\
             37aa559095324d78459b9bb2da069da32337e1cc5da78f48e1bd084670107f31\
             10f3245ddf9132ecef98c670272ef39c03a232107733d4a1d28cb53318df26fa\
             e0d152bb611cb9ff34e945e4ff627e6fba81da687a601a879759cd76530b5744\
             424db69a75edd4780a5fbc05d1a3c84ac4166ff8e424808481dd8e77627ce5f5\
             bf2eea84515a4e16c4ff06c92381822d910b5cbf9e9c144e1fb76a6291af7276"
                .replace(char::is_whitespace, "")
        );
    }
}
