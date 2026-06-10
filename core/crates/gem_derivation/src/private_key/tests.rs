use primitives::{Chain, hex::decode_hex};

use super::*;
use crate::AccountDerivationError;

#[test]
fn test_private_key_account_derivation_selected_chains() {
    let secp_private_key = decode_hex("30df0ffc2b43717f4653c2a1e827e9dfb3d9364e019cc60092496cd4997d5d6e").unwrap();
    assert_eq!(
        derive_account_from_private_key(&secp_private_key, Chain::Ethereum).unwrap().address,
        "0x4ce31c0b2114abe61Ac123E1E6254E961C18D10B"
    );
    assert_eq!(
        derive_account_from_private_key(&secp_private_key, Chain::HyperCore).unwrap().address,
        "0x4ce31c0b2114abe61Ac123E1E6254E961C18D10B"
    );
    assert_eq!(
        derive_account_from_private_key(&secp_private_key, Chain::Tron).unwrap().address,
        "TGykRV8uNxzjgbAKz9zWaaniwXT4nRcWaf"
    );

    let ed_private_key = decode_hex("3d769e8a65b9002a470e9aecf2587ef848e2a0b483320e24c493a5913d594eb9").unwrap();
    assert_eq!(
        derive_account_from_private_key(&ed_private_key, Chain::Solana).unwrap().address,
        "QszdTXmSeM88WET7RqJbE51SWNWLG3975rVHT5bSyAP"
    );
    assert_eq!(
        derive_account_from_private_key(&ed_private_key, Chain::Near).unwrap().address,
        "061e046a9b89bf2fd06e92b43dad4fc997cb1d0a2aab5d76c65687ddb0b22308"
    );
    assert_eq!(
        derive_account_from_private_key(&ed_private_key, Chain::Stellar).unwrap().address,
        "GADB4BDKTOE36L6QN2JLIPNNJ7EZPSY5BIVKWXLWYZLIPXNQWIRQQZKT"
    );
    assert_eq!(
        derive_account_from_private_key(&ed_private_key, Chain::Algorand).unwrap().address,
        "AYPAI2U3RG7S7UDOSK2D3LKPZGL4WHIKFKVV25WGK2D53MFSEMEEEPTJWY"
    );
    assert_eq!(
        derive_account_from_private_key(&ed_private_key, Chain::Aptos).unwrap().address,
        "0x82494016df078d6f9041319694f0973cab5e31cc9787d2b21a6303dd23fbb07e"
    );
    assert_eq!(
        derive_account_from_private_key(&ed_private_key, Chain::Sui).unwrap().address,
        "0xd1fd4baa9c58d54f561ba224506a572b02d0f8ed7fe2f7e70b5a93cc19c8e308"
    );
}

#[test]
fn test_import_private_key_rejects_bitcoin_family() {
    assert_eq!(
        import_account_from_private_key("0x30df0ffc2b43717f4653c2a1e827e9dfb3d9364e019cc60092496cd4997d5d6e", Chain::Bitcoin)
            .err()
            .unwrap(),
        AccountDerivationError::InvalidPrivateKey
    );
}
