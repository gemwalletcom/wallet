use primitives::{Chain, WalletType, hex};

use crate::{AccountDerivationError, derive_wallet_id_from_account};

use super::super::derivation::{derive_accounts_from_mnemonic, derive_private_key_from_mnemonic};
use super::{PHRASE, TEST_PHRASE, bitcoin_family_v3_vectors, expected_derivation};

#[test]
fn test_derive_accounts_from_mnemonic_selected_chains() {
    let accounts = derive_accounts_from_mnemonic(
        PHRASE,
        vec![
            Chain::Ethereum,
            Chain::Tron,
            Chain::Solana,
            Chain::Aptos,
            Chain::Sui,
            Chain::Near,
            Chain::Stellar,
            Chain::Algorand,
        ],
    )
    .unwrap();

    assert_eq!(accounts[0].chain, Chain::Ethereum);
    assert_eq!(accounts[0].address, "0x9858EfFD232B4033E47d90003D41EC34EcaEda94");
    assert_eq!(accounts[0].derivation_path, "m/44'/60'/0'/0/0");
    assert_eq!(accounts[1].address, "TUEZSdKsoDHQMeZwihtdoBiN46zxhGWYdH");
    assert_eq!(accounts[2].address, "HAgk14JpMQLgt6rVgv7cBQFJWFto5Dqxi472uT3DKpqk");
    assert_eq!(accounts[2].derivation_path, "m/44'/501'/0'/0'");
    assert_eq!(accounts[3].address, "0xeb663b681209e7087d681c5d3eed12aaa8e1915e7c87794542c3f96e94b3d3bf");
    assert_eq!(accounts[4].address, "0x5e93a736d04fbb25737aa40bee40171ef79f65fae833749e3c089fe7cc2161f1");
    assert_eq!(accounts[5].address, "5510e2b44cae6eb807e3e0e45d579dda058c274abcba15e5cb84636f5d1ee412");
    assert_eq!(accounts[6].address, "GB3JDWCQJCWMJ3IILWIGDTQJJC5567PGVEVXSCVPEQOTDN64VJBDQBYX");
    assert_eq!(accounts[7].address, "EP2D7TV7IAFANZHK3B6QLKB53N5UTD7RARVXZTWCPCRQQBKYVGM2XIMT2Q");
}

#[test]
fn test_derive_accounts_from_mnemonic_deduplicates_and_rejects_empty_chains() {
    let accounts = derive_accounts_from_mnemonic(PHRASE, vec![Chain::Ethereum, Chain::Ethereum]).unwrap();
    assert_eq!(accounts.len(), 1);

    assert_eq!(derive_accounts_from_mnemonic(PHRASE, vec![Chain::Cardano]).unwrap().len(), 1);
    assert_eq!(
        derive_accounts_from_mnemonic(PHRASE, Vec::new()).unwrap_err(),
        AccountDerivationError::invalid_input("mnemonic derivation requires at least one chain")
    );
}

#[test]
fn test_derive_accounts_from_mnemonic_bitcoin_family_v3_vectors() {
    for vector in bitcoin_family_v3_vectors() {
        let account = derive_accounts_from_mnemonic(&vector.phrase, vec![vector.chain]).unwrap().remove(0);
        assert_eq!(account.address, vector.address);
        assert_eq!(account.extended_public_key.as_deref(), Some(vector.extended_public_key.as_str()));
    }

    let bitcoincash = derive_accounts_from_mnemonic(TEST_PHRASE, vec![Chain::BitcoinCash]).unwrap().remove(0);
    assert_eq!(bitcoincash.address, "qq29xrkkd68alnrca375qlfyhwdqdkevsvmgkq9cmw");
    assert_eq!(
        bitcoincash.extended_public_key.as_deref(),
        Some("xpub6Cd3LU6iyrbbhxPRYZpE5hGUdmrQVpQ79i9RYNLrs2iVrtYkKRv6swMWeTpPfomebgisrRGPrFvt1qaFiZLLuQdSFRVBWdbKD4HWnMrFsjR")
    );
}

#[test]
fn test_derive_accounts_public_key() {
    let accounts = derive_accounts_from_mnemonic(TEST_PHRASE, vec![Chain::Ethereum, Chain::Solana, Chain::Cosmos, Chain::Bitcoin, Chain::Cardano]).unwrap();

    assert_eq!(
        accounts[0].extended_public_key.as_deref(),
        Some("045515e0ac635b35f12639f7df11f4488ba2f3dfa3ba4e11e286cfb59c45af60d8455161a6f2294f567ac4d8bc70fb26abd78338cbb5f9f238bdb5a875b390eaa2")
    );
    assert_eq!(
        accounts[1].extended_public_key.as_deref(),
        Some("34bee4f639dc054e05c01a0d196aed6db69d56e4ea920b3c022d2f0d5bce73a9")
    );
    assert_eq!(
        accounts[2].extended_public_key.as_deref(),
        Some("032c95c627ec384f7b96fff253371673d7846063e52742d79f97235859d6061b04")
    );
    assert_eq!(
        accounts[3].extended_public_key.as_deref(),
        Some("zpub6qQxTLc1Xa8mbCLj7CUrdLogszRJeCcVx8cKYs5HuH9UURf5nB7kmBBJbL5nvxZcYF7EXNU9zP8pWbqYxCLdW26bqq6tDuxxWVkjmn8Fks9")
    );
    assert_eq!(accounts[4].extended_public_key, None);
}

#[test]
fn test_mnemonic_derivation_coverage_for_every_chain() {
    for chain in Chain::all() {
        let expected = expected_derivation(chain);
        let accounts = derive_accounts_from_mnemonic(PHRASE, vec![chain]).unwrap();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].chain, chain);
        assert_eq!(accounts[0].address, expected.address);
        assert_eq!(accounts[0].derivation_path, expected.path);
    }
}

#[test]
fn test_mnemonic_wallet_id() {
    let ethereum = derive_accounts_from_mnemonic(PHRASE, vec![Chain::Ethereum]).unwrap().remove(0);
    let solana = derive_accounts_from_mnemonic(PHRASE, vec![Chain::Solana]).unwrap().remove(0);

    assert_eq!(
        derive_wallet_id_from_account(&ethereum, WalletType::Multicoin).unwrap().to_string(),
        "multicoin_0x9858EfFD232B4033E47d90003D41EC34EcaEda94"
    );
    assert_eq!(
        derive_wallet_id_from_account(&solana, WalletType::Single).unwrap().to_string(),
        "single_solana_HAgk14JpMQLgt6rVgv7cBQFJWFto5Dqxi472uT3DKpqk"
    );
}

#[test]
fn test_derive_private_key_from_mnemonic() {
    let private_key = derive_private_key_from_mnemonic(PHRASE, Chain::Ethereum).unwrap();

    assert_eq!(private_key.len(), 32);
    assert_eq!(hex::encode(private_key.as_slice()), "1ab42cc412b618bdea3a599e3c9bae199ebf030895b039e9db1e30dafb12b727");
}
