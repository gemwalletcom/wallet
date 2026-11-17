use primitives::{Account, Chain, hex};
use signer::{decode_private_key, supports_private_key_import};
use zeroize::Zeroizing;

use crate::AccountDerivationError;

use super::{
    address::{address_from_private_key, address_from_public_key, public_key_from_private_key},
    path::default_derivation_path,
};

pub struct ImportedPrivateKeyAccount {
    pub account: Account,
    pub private_key: Zeroizing<Vec<u8>>,
}

pub fn import_account_from_private_key(value: &str, chain: Chain) -> Result<ImportedPrivateKeyAccount, AccountDerivationError> {
    let private_key = decode_supported_private_key(value, chain)?;
    let account = derive_account_from_private_key(&private_key, chain)?;
    Ok(ImportedPrivateKeyAccount { account, private_key })
}

pub fn derive_account_from_private_key_value(value: &str, chain: Chain) -> Result<Account, AccountDerivationError> {
    let private_key = decode_supported_private_key(value, chain)?;
    derive_account_from_private_key(&private_key, chain)
}

fn decode_supported_private_key(value: &str, chain: Chain) -> Result<Zeroizing<Vec<u8>>, AccountDerivationError> {
    if !supports_private_key_import(&chain) {
        return Err(AccountDerivationError::InvalidPrivateKey);
    }
    decode_private_key(&chain, value).map_err(|_| AccountDerivationError::InvalidPrivateKey)
}

pub fn derive_account_from_private_key(private_key: &[u8], chain: Chain) -> Result<Account, AccountDerivationError> {
    let public_key = public_key_from_private_key(private_key, chain)?;
    let address = match &public_key {
        Some(public_key) => address_from_public_key(public_key, chain)?,
        None => address_from_private_key(private_key, chain)?,
    };
    Ok(Account {
        chain,
        address,
        derivation_path: default_derivation_path(chain).to_string(),
        extended_public_key: public_key.map(hex::encode),
    })
}

#[cfg(test)]
mod tests {
    use gem_encoding::encode_base58;
    use primitives::hex::decode_hex;
    use signer::Ed25519KeyPair;

    use super::*;

    #[test]
    fn test_private_key_account_derivation_selected_chains() {
        let secp_private_key = decode_hex("30df0ffc2b43717f4653c2a1e827e9dfb3d9364e019cc60092496cd4997d5d6e").unwrap();
        assert_eq!(derive_account_from_private_key(&secp_private_key, Chain::Ethereum).unwrap().address, "0x4ce31c0b2114abe61Ac123E1E6254E961C18D10B");
        assert_eq!(derive_account_from_private_key(&secp_private_key, Chain::HyperCore).unwrap().address, "0x4ce31c0b2114abe61Ac123E1E6254E961C18D10B");
        assert_eq!(derive_account_from_private_key(&secp_private_key, Chain::Tron).unwrap().address, "TGykRV8uNxzjgbAKz9zWaaniwXT4nRcWaf");

        let ed_private_key = decode_hex("3d769e8a65b9002a470e9aecf2587ef848e2a0b483320e24c493a5913d594eb9").unwrap();
        assert_eq!(derive_account_from_private_key(&ed_private_key, Chain::Solana).unwrap().address, "QszdTXmSeM88WET7RqJbE51SWNWLG3975rVHT5bSyAP");
        assert_eq!(
            derive_account_from_private_key(&ed_private_key, Chain::Near).unwrap().address,
            "061e046a9b89bf2fd06e92b43dad4fc997cb1d0a2aab5d76c65687ddb0b22308"
        );
        assert_eq!(derive_account_from_private_key(&ed_private_key, Chain::Stellar).unwrap().address, "GADB4BDKTOE36L6QN2JLIPNNJ7EZPSY5BIVKWXLWYZLIPXNQWIRQQZKT");
        assert_eq!(derive_account_from_private_key(&ed_private_key, Chain::Algorand).unwrap().address, "AYPAI2U3RG7S7UDOSK2D3LKPZGL4WHIKFKVV25WGK2D53MFSEMEEEPTJWY");
        assert_eq!(
            derive_account_from_private_key(&ed_private_key, Chain::Aptos).unwrap().address,
            "0x82494016df078d6f9041319694f0973cab5e31cc9787d2b21a6303dd23fbb07e"
        );
        assert_eq!(
            derive_account_from_private_key(&ed_private_key, Chain::Sui).unwrap().address,
            "0xd1fd4baa9c58d54f561ba224506a572b02d0f8ed7fe2f7e70b5a93cc19c8e308"
        );

        let aptos_private_key = decode_hex("0000000000000000000000000000000000000000000000000000000000000022").unwrap();
        assert_eq!(
            derive_account_from_private_key(&aptos_private_key, Chain::Aptos).unwrap().address,
            "0x0dc4a9af7d0b8e5811bf326bf49c44f8685d4ee5b4fd3606e06987c5056f61bb"
        );
    }

    #[test]
    fn test_import_private_key_rejects_bitcoin_family() {
        assert_eq!(
            import_account_from_private_key("0x30df0ffc2b43717f4653c2a1e827e9dfb3d9364e019cc60092496cd4997d5d6e", Chain::Bitcoin).err().unwrap(),
            AccountDerivationError::InvalidPrivateKey
        );
    }

    #[test]
    fn test_import_near_prefixed_private_key() {
        let private_key = decode_hex("3d769e8a65b9002a470e9aecf2587ef848e2a0b483320e24c493a5913d594eb9").unwrap();
        let public_key = Ed25519KeyPair::from_private_key(&private_key).unwrap().public_key_bytes;
        let encoded = format!("ed25519:{}", encode_base58(private_key.iter().chain(public_key.iter()).copied().collect::<Vec<_>>()));

        assert_eq!(
            import_account_from_private_key(&encoded, Chain::Near).unwrap().account.address,
            "061e046a9b89bf2fd06e92b43dad4fc997cb1d0a2aab5d76c65687ddb0b22308"
        );
        assert_eq!(import_account_from_private_key(&encoded, Chain::Solana).err().unwrap(), AccountDerivationError::InvalidPrivateKey);
    }

    #[test]
    fn test_import_sui_and_aptos_private_keys() {
        let sui_address = "0x90f3e6d73b5730f16974f4df1d3441394ebae62186baf83608599f226455afa7";
        let aptos_address = "0x0dc4a9af7d0b8e5811bf326bf49c44f8685d4ee5b4fd3606e06987c5056f61bb";
        let values = [
            ("suiprivkey1qzwant3kaegmjy4qxex93s0jzvemekkjmyv3r2sjwgnv2y479pgsywhveae", Chain::Sui, sui_address),
            ("0x9dd9ae36ee51b912a0364c58c1f21333bcdad2d91911aa127226c512be285102", Chain::Sui, sui_address),
            ("ed25519-priv-0x0000000000000000000000000000000000000000000000000000000000000022", Chain::Aptos, aptos_address),
            ("0x0000000000000000000000000000000000000000000000000000000000000022", Chain::Aptos, aptos_address),
        ];
        for (value, chain, address) in values {
            assert_eq!(import_account_from_private_key(value, chain).unwrap().account.address, address);
        }
    }
}
