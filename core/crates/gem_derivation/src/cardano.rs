use gem_cardano::public_key_from_extended_secret;

use crate::{AccountDerivationError, read_array};

const KEY_LENGTH: usize = 32;
const STAKE_SECRET_OFFSET: usize = 96;

pub(crate) fn address_from_extended_private_key(private_key: &[u8]) -> Result<String, AccountDerivationError> {
    let (payment_public_key, stake_public_key) = public_keys_from_extended_private_key(private_key)?;
    Ok(gem_cardano::address_from_public_keys(payment_public_key, stake_public_key)?)
}

fn public_keys_from_extended_private_key(private_key: &[u8]) -> Result<([u8; KEY_LENGTH], [u8; KEY_LENGTH]), AccountDerivationError> {
    if private_key.len() != signer::CARDANO_EXTENDED_PRIVATE_KEY_LENGTH {
        return Err(AccountDerivationError::invalid_input("invalid Cardano private key length"));
    }

    Ok((
        public_key_from_extended_secret(read_array(private_key, 0)),
        public_key_from_extended_secret(read_array(private_key, STAKE_SECRET_OFFSET)),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{decode_hex, hex};

    #[test]
    fn test_cardano_address_from_extended_private_key() {
        let private_key = decode_hex(
            "e8c8c5b2df13f3abed4e6b1609c808e08ff959d7e6fc3d849e3f2880550b5744\
             37aa559095324d78459b9bb2da069da32337e1cc5da78f48e1bd084670107f31\
             10f3245ddf9132ecef98c670272ef39c03a232107733d4a1d28cb53318df26fa\
             e0d152bb611cb9ff34e945e4ff627e6fba81da687a601a879759cd76530b5744\
             424db69a75edd4780a5fbc05d1a3c84ac4166ff8e424808481dd8e77627ce5f5\
             bf2eea84515a4e16c4ff06c92381822d910b5cbf9e9c144e1fb76a6291af7276",
        )
        .unwrap();

        assert_eq!(
            address_from_extended_private_key(&private_key).unwrap(),
            "addr1qxxe304qg9py8hyyqu8evfj4wln7dnms943wsugpdzzsxnkvvjljtzuwxvx0pnwelkcruy95ujkq3aw6rl0vvg32x35qc92xkq"
        );
    }

    #[test]
    fn test_cardano_public_key_from_extended_secret() {
        let secret = decode_hex("e8c8c5b2df13f3abed4e6b1609c808e08ff959d7e6fc3d849e3f2880550b5744").unwrap().try_into().unwrap();

        assert_eq!(
            hex::encode(public_key_from_extended_secret(secret)),
            "fafa7eb4146220db67156a03a5f7a79c666df83eb31abbfbe77c85e06d40da31"
        );
    }
}
