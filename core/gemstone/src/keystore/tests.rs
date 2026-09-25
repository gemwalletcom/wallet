use gem_ton::Address as TonAddress;
use primitives::{Asset, Chain, SignerError, SignerInput, TransactionInputType, WalletType};
use tempfile::TempDir;
use zeroize::Zeroizing;

use super::testkit::mock_phrase_words;
use super::*;
use crate::GemstoneError;
use crate::auth::sign_auth_message_hash;
use crate::message::sign_type::{SignDigestType, SignMessage};
use crate::message::signer::MessageSigner;
use crate::models::transaction::GemSignerInput;
use gem_derivation::{derive_account_from_private_key, derive_legacy_solana_private_key_from_mnemonic};

#[test]
fn test_gem_keystore_rejects_unrelated_transaction_sender() {
    let dir = TempDir::new().unwrap();
    let keystore = GemKeystore::new(dir.path().to_string_lossy().to_string()).unwrap();
    let stored = keystore.create_store(GemImportType::mock_private_key(), b"password".to_vec()).unwrap();
    let mut input: GemSignerInput = SignerInput::mock_evm(TransactionInputType::Transfer { asset: Asset::mock() }, "1", 21000).into();
    input.input.sender_address = stored.accounts[0].address.to_lowercase();
    assert_eq!(keystore.sign(stored.keystore_id.clone(), Chain::Ethereum, input.clone(), b"password".to_vec()).unwrap().len(), 1);

    input.input.sender_address = input.input.destination_address.clone();
    assert_eq!(
        keystore.sign(stored.keystore_id, Chain::Ethereum, input, b"password".to_vec()).unwrap_err(),
        GemstoneError::from(SignerError::invalid_input("signing key does not match the approved account"))
    );
}

#[test]
fn test_signing_key_binds_mnemonic_accounts_and_accepts_equivalent_encodings() {
    let dir = TempDir::new().unwrap();
    let keystore = GemKeystore::new(dir.path().to_string_lossy().to_string()).unwrap();
    let chains = vec![
        Chain::Ethereum,
        Chain::Solana,
        Chain::Bitcoin,
        Chain::BitcoinCash,
        Chain::Cardano,
        Chain::Ton,
        Chain::Near,
        Chain::Tron,
        Chain::Stellar,
        Chain::Polkadot,
        Chain::Cosmos,
        Chain::Aptos,
        Chain::Sui,
        Chain::Xrp,
        Chain::Algorand,
    ];
    let stored = keystore.create_store(GemImportType::mock_multicoin_phrase(chains), b"password".to_vec()).unwrap();
    for account in &stored.accounts {
        assert!(keystore.signing_key(&stored.keystore_id, account.chain, &account.address, b"password".to_vec()).is_ok(), "{}", account.chain);
        assert_eq!(
            keystore.signing_key(&stored.keystore_id, account.chain, "unrelated", b"password".to_vec()).err().unwrap(),
            GemstoneError::from(SignerError::invalid_input("signing key does not match the approved account"))
        );
    }
    let ton = stored.accounts.iter().find(|account| account.chain == Chain::Ton).unwrap();
    let bounceable = TonAddress::parse(&ton.address).unwrap().encode_bounceable();
    assert!(keystore.signing_key(&stored.keystore_id, Chain::Ton, &bounceable, b"password".to_vec()).is_ok());
    let bitcoin_cash = stored.accounts.iter().find(|account| account.chain == Chain::BitcoinCash).unwrap();
    let prefixed = format!("bitcoincash:{}", bitcoin_cash.address);
    assert!(keystore.signing_key(&stored.keystore_id, Chain::BitcoinCash, &prefixed, b"password".to_vec()).is_ok());
}

#[test]
fn test_signing_key_accepts_the_legacy_solana_derivation_only_for_its_address() {
    let dir = TempDir::new().unwrap();
    let keystore = GemKeystore::new(dir.path().to_string_lossy().to_string()).unwrap();
    let stored = keystore.create_store(GemImportType::mock_single_phrase(), b"password".to_vec()).unwrap();
    let legacy_key = derive_legacy_solana_private_key_from_mnemonic(&mock_phrase_words().join(" ")).unwrap();
    let legacy_address = derive_account_from_private_key(&legacy_key, Chain::Solana).unwrap().address;
    assert_eq!(legacy_address, "GjJyeC1r2RgkuoCWMyPYkCWSGSGLcz266EaAkLA27AhL");
    assert_ne!(legacy_address, stored.accounts[0].address);

    assert_eq!(*keystore.signing_key(&stored.keystore_id, Chain::Solana, &legacy_address, b"password".to_vec()).unwrap(), *legacy_key);
    assert_eq!(
        *keystore.signing_key(&stored.keystore_id, Chain::Solana, &stored.accounts[0].address, b"password".to_vec()).unwrap(),
        keystore.private_key(stored.keystore_id.clone(), Chain::Solana, b"password".to_vec()).unwrap()
    );
    assert_eq!(
        keystore.signing_key(&stored.keystore_id, Chain::Solana, "unrelated", b"password".to_vec()).unwrap_err(),
        GemstoneError::from(SignerError::invalid_input("signing key does not match the approved account"))
    );
}

#[test]
fn test_gem_keystore_private_key_create_export_delete() {
    let dir = TempDir::new().unwrap();
    let keystore = GemKeystore::new(dir.path().to_string_lossy().to_string()).unwrap();
    let stored = keystore.create_store(GemImportType::mock_private_key(), b"password".to_vec()).unwrap();

    assert_eq!(stored.wallet_type, WalletType::PrivateKey);
    assert_eq!(stored.accounts[0].address, "0x4ce31c0b2114abe61Ac123E1E6254E961C18D10B");
    assert_eq!(
        keystore.export_private_key(stored.keystore_id.clone(), Chain::Ethereum, b"password".to_vec()).unwrap(),
        "0x30df0ffc2b43717f4653c2a1e827e9dfb3d9364e019cc60092496cd4997d5d6e"
    );
    assert_eq!(
        keystore.add_accounts(stored.keystore_id.clone(), b"password".to_vec(), vec![Chain::Polygon]).unwrap_err().to_string(),
        "add_accounts does not support private-key wallets"
    );
    assert!(keystore.delete(stored.keystore_id.clone()).unwrap());
}

#[test]
fn test_gem_keystore_sign_with_keystore_matches_raw_key() {
    let dir = TempDir::new().unwrap();
    let keystore = GemKeystore::new(dir.path().to_string_lossy().to_string()).unwrap();
    let stored = keystore.create_store(GemImportType::mock_private_key(), b"password".to_vec()).unwrap();
    let raw_key = keystore.private_key(stored.keystore_id.clone(), Chain::Ethereum, b"password".to_vec()).unwrap();

    let signer = MessageSigner::new(SignMessage {
        chain: Chain::Ethereum,
        sign_type: SignDigestType::Eip191,
        data: b"hello world".to_vec(),
    });
    let expected = signer.sign(Zeroizing::new(raw_key)).unwrap();
    let actual = signer.sign_with_keystore(keystore.clone(), stored.keystore_id.clone(), &stored.accounts[0].address, b"password".to_vec()).unwrap();
    assert_eq!(actual, expected);
    assert_eq!(
        signer.sign_with_keystore(keystore, stored.keystore_id, "", b"password".to_vec()).unwrap_err(),
        GemstoneError::from(SignerError::invalid_input("signing key does not match the approved account"))
    );
}

#[test]
fn test_gem_keystore_sign_auth_matches_raw_key() {
    let dir = TempDir::new().unwrap();
    let keystore = GemKeystore::new(dir.path().to_string_lossy().to_string()).unwrap();
    let stored = keystore.create_store(GemImportType::mock_private_key(), b"password".to_vec()).unwrap();
    let raw_key = keystore.private_key(stored.keystore_id.clone(), Chain::Ethereum, b"password".to_vec()).unwrap();

    // Auth signing through the keystore must match signing the hash with the exported raw key.
    let hash = [7u8; 32];
    let expected = sign_auth_message_hash(hash, Zeroizing::new(raw_key)).unwrap();
    let actual = keystore.sign_auth(stored.keystore_id.clone(), Chain::Ethereum, &stored.accounts[0].address, hash, b"password".to_vec()).unwrap();
    assert_eq!(actual, expected);
    assert_eq!(
        keystore.sign_auth(stored.keystore_id, Chain::Ethereum, "", hash, b"password".to_vec()).unwrap_err(),
        GemstoneError::from(SignerError::invalid_input("signing key does not match the approved account"))
    );
}

#[test]
fn test_gem_keystore_mnemonic_import_create_export_add_accounts() {
    let dir = TempDir::new().unwrap();
    let keystore = GemKeystore::new(dir.path().to_string_lossy().to_string()).unwrap();

    let import = keystore.preview_import(GemImportType::mock_multicoin_phrase(vec![Chain::Ethereum, Chain::Solana, Chain::Bitcoin])).unwrap();
    assert_eq!(import.wallet_id, "multicoin_0x9858EfFD232B4033E47d90003D41EC34EcaEda94");
    assert_eq!(import.wallet_type, WalletType::Multicoin);
    assert_eq!(import.accounts[0].address, "0x9858EfFD232B4033E47d90003D41EC34EcaEda94");
    assert_eq!(import.accounts[1].address, "HAgk14JpMQLgt6rVgv7cBQFJWFto5Dqxi472uT3DKpqk");
    assert_eq!(import.accounts[1].derivation_path, "m/44'/501'/0'/0'");
    assert!(import.accounts[2].public_key.as_deref().is_some_and(|public_key| public_key.starts_with("zpub")));

    let stored = keystore.create_store(GemImportType::mock_multicoin_phrase(vec![Chain::Ethereum]), b"password".to_vec()).unwrap();
    assert_eq!(stored.wallet_id, "multicoin_0x9858EfFD232B4033E47d90003D41EC34EcaEda94");
    assert_eq!(stored.wallet_type, WalletType::Multicoin);
    assert_eq!(stored.accounts.len(), 1);
    assert_eq!(keystore.export_recovery_phrase(stored.keystore_id.clone(), b"password".to_vec()).unwrap(), mock_phrase_words());
    assert_eq!(
        keystore.export_private_key(stored.keystore_id.clone(), Chain::Ethereum, b"password".to_vec()).unwrap(),
        "0x1ab42cc412b618bdea3a599e3c9bae199ebf030895b039e9db1e30dafb12b727"
    );
    assert_eq!(
        primitives::hex::encode(keystore.private_key(stored.keystore_id.clone(), Chain::Ethereum, b"password".to_vec()).unwrap()),
        "1ab42cc412b618bdea3a599e3c9bae199ebf030895b039e9db1e30dafb12b727"
    );

    let added = keystore.add_accounts(stored.keystore_id.clone(), b"password".to_vec(), vec![Chain::Polygon, Chain::Tron]).unwrap();
    assert_eq!(added[0].chain, Chain::Polygon);
    assert_eq!(added[0].address, "0x9858EfFD232B4033E47d90003D41EC34EcaEda94");
    assert_eq!(added[1].chain, Chain::Tron);
    assert_eq!(added[1].address, "TUEZSdKsoDHQMeZwihtdoBiN46zxhGWYdH");
}

#[test]
fn test_gem_keystore_v4_password_is_opaque_bytes() {
    let dir = TempDir::new().unwrap();
    let keystore = GemKeystore::new(dir.path().to_string_lossy().to_string()).unwrap();
    let password = vec![0xde, 0xad, 0xbe, 0xef, 0x00, 0xff];
    let stored = keystore.create_store(GemImportType::mock_multicoin_phrase(vec![Chain::Ethereum]), password.clone()).unwrap();

    assert_eq!(keystore.export_recovery_phrase(stored.keystore_id.clone(), password).unwrap(), mock_phrase_words());
    assert!(keystore.export_recovery_phrase(stored.keystore_id, b"deadbeef00ff".to_vec()).is_err());
}

#[test]
fn test_gem_keystore_has_stored_wallets() {
    let dir = TempDir::new().unwrap();
    let keystore = GemKeystore::new(dir.path().to_string_lossy().to_string()).unwrap();
    assert!(!keystore.has_stored_wallets().unwrap());

    let stored = keystore.create_store(GemImportType::mock_multicoin_phrase(vec![Chain::Ethereum]), b"password".to_vec()).unwrap();
    assert!(keystore.has_stored_wallets().unwrap());

    keystore.delete(stored.keystore_id).unwrap();
    assert!(!keystore.has_stored_wallets().unwrap());
}

#[test]
fn test_gem_keystore_single_phrase_import_create() {
    let dir = TempDir::new().unwrap();
    let keystore = GemKeystore::new(dir.path().to_string_lossy().to_string()).unwrap();

    let import = keystore.preview_import(GemImportType::mock_single_phrase()).unwrap();
    assert_eq!(import.wallet_id, "single_solana_HAgk14JpMQLgt6rVgv7cBQFJWFto5Dqxi472uT3DKpqk");
    assert_eq!(import.wallet_type, WalletType::Single);
    assert_eq!(import.accounts.len(), 1);
    assert_eq!(import.accounts[0].chain, Chain::Solana);
    assert_eq!(import.accounts[0].derivation_path, "m/44'/501'/0'/0'");

    let stored = keystore.create_store(GemImportType::mock_single_phrase(), b"password".to_vec()).unwrap();
    assert_eq!(stored.wallet_id, "single_solana_HAgk14JpMQLgt6rVgv7cBQFJWFto5Dqxi472uT3DKpqk");
    assert_eq!(stored.wallet_type, WalletType::Single);
    assert_eq!(stored.accounts.len(), 1);
    assert_eq!(stored.accounts[0].chain, Chain::Solana);
    assert_eq!(stored.accounts[0].derivation_path, "m/44'/501'/0'/0'");
}

#[test]
fn test_gem_keystore_survives_concurrent_create_read_and_delete() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().to_string_lossy().to_string();
    let password = || b"password".to_vec();
    let threads = 8;

    let created: Vec<_> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|_| {
                let path = path.clone();
                scope.spawn(move || GemKeystore::new(path).unwrap().create_store(GemImportType::mock_single_phrase(), password()).unwrap())
            })
            .collect();
        handles.into_iter().map(|handle| handle.join().unwrap()).collect()
    });

    let keystore_id = created[0].keystore_id.clone();
    assert!(created.iter().all(|stored| stored.keystore_id == keystore_id), "concurrent creates of one wallet must agree on its id");
    let files: Vec<_> = std::fs::read_dir(dir.path())
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().is_some_and(|extension| extension == "json"))
        .collect();
    assert_eq!(files.len(), 1, "the race must leave one keystore file, not a duplicate per thread");

    let phrases: Vec<_> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|_| {
                let (path, keystore_id) = (path.clone(), keystore_id.clone());
                scope.spawn(move || GemKeystore::new(path).unwrap().export_recovery_phrase(keystore_id, password()).unwrap().join(" "))
            })
            .collect();
        handles.into_iter().map(|handle| handle.join().unwrap()).collect()
    });
    assert!(phrases.iter().all(|phrase| *phrase == mock_phrase_words().join(" ")), "every concurrent read must return the stored phrase");

    std::thread::scope(|scope| {
        for _ in 0..threads {
            let (path, keystore_id) = (path.clone(), keystore_id.clone());
            scope.spawn(move || GemKeystore::new(path).unwrap().delete(keystore_id));
        }
    });
    assert!(!dir.path().join(format!("{keystore_id}.json")).exists(), "the file must be gone after a concurrent delete");
}
