use gem_bitcoin::BitcoinAddress;
use gem_cosmos::address::CosmosAddress;
use gem_hash::blake2::blake2b_256;
use gem_hash::keccak::keccak256;
use gem_hash::sha2::hash160;
use gem_hash::sha3::sha3_256;
use primitives::chain_cosmos::CosmosChain;
use primitives::{Address, BitcoinChain, Chain, ChainType, hex};
use signer::{Ed25519KeyPair, secp256k1_public_key, secp256k1_uncompressed_public_key};

use crate::AccountDerivationError;

const SECP256K1_UNCOMPRESSED_PUBLIC_KEY_PREFIX: u8 = 0x04;
const SECP256K1_COMPRESSED_PUBLIC_KEY_LEN: usize = 33;
const SECP256K1_UNCOMPRESSED_PUBLIC_KEY_LEN: usize = 65;
const ED25519_PUBLIC_KEY_LEN: usize = 32;
const EVM_ADDRESS_LEN: usize = 20;

pub(super) fn address_from_public_key(public_key: &[u8], chain: Chain) -> Result<String, AccountDerivationError> {
    match chain.chain_type() {
        ChainType::Ethereum | ChainType::HyperCore => Ok(gem_evm::EthereumAddress::from_bytes(secp256k1_keccak_address_hash(public_key)?).encode()),
        ChainType::Tron => Ok(gem_tron::address::TronAddress::from(secp256k1_keccak_address_hash(public_key)?).encode()),
        ChainType::Solana => Ok(gem_solana::address::SolanaAddress::from_bytes(ed25519_public_key(public_key)?).encode()),
        ChainType::Aptos => aptos_address_from_public_key(public_key),
        ChainType::Sui => sui_address_from_public_key(public_key),
        ChainType::Near => Ok(hex::encode(ed25519_public_key(public_key)?)),
        ChainType::Stellar => Ok(gem_stellar::StellarAddress::from_public_key(&ed25519_public_key(public_key)?)?.encode()),
        ChainType::Algorand => Ok(gem_algorand::AlgorandAddress::from_public_key(&ed25519_public_key(public_key)?)?.encode()),
        ChainType::Cosmos => cosmos_address_from_public_key(public_key, chain),
        ChainType::Ton => Ok(gem_ton::signer::WalletV4R2::new(ed25519_public_key(public_key)?)?.address.encode_non_bounceable()),
        ChainType::Xrp => Ok(gem_xrp::XrpAddress::from_public_key_hash(hash160(compressed_public_key(public_key)?)).encode()),
        ChainType::Polkadot => Ok(gem_polkadot::PolkadotAddress::from_public_key(ed25519_public_key(public_key)?).encode()),
        ChainType::Bitcoin | ChainType::Cardano => Err(AccountDerivationError::unsupported_chain(chain)),
    }
}

pub(super) fn address_from_private_key(private_key: &[u8], chain: Chain) -> Result<String, AccountDerivationError> {
    match chain.chain_type() {
        ChainType::Bitcoin => bitcoin_address_from_private_key(private_key, chain),
        ChainType::Cardano => crate::cardano::address_from_extended_private_key(private_key),
        _ => Err(AccountDerivationError::unsupported_chain(chain)),
    }
}

pub(super) fn public_key_from_private_key(private_key: &[u8], chain: Chain) -> Result<Option<Vec<u8>>, AccountDerivationError> {
    let public_key = match chain.chain_type() {
        ChainType::Ethereum | ChainType::HyperCore | ChainType::Tron => uncompressed_secp256k1_public_key(private_key)?,
        ChainType::Xrp => compressed_secp256k1_public_key(private_key)?,
        ChainType::Cosmos => match CosmosChain::from_chain(chain).ok_or_else(|| AccountDerivationError::unsupported_chain(chain))? {
            CosmosChain::Injective => uncompressed_secp256k1_public_key(private_key)?,
            CosmosChain::Cosmos | CosmosChain::Osmosis | CosmosChain::Celestia | CosmosChain::Thorchain | CosmosChain::Mayachain | CosmosChain::Sei | CosmosChain::Noble => {
                compressed_secp256k1_public_key(private_key)?
            }
        },
        ChainType::Solana | ChainType::Aptos | ChainType::Sui | ChainType::Near | ChainType::Stellar | ChainType::Algorand | ChainType::Ton | ChainType::Polkadot => {
            Ed25519KeyPair::from_private_key(private_key)
                .map_err(|_| AccountDerivationError::InvalidPrivateKey)?
                .public_key_bytes
                .to_vec()
        }
        // Bitcoin (account stores an xpub) and Cardano (extended key with separate payment/stake keys) have no single reusable public key.
        ChainType::Bitcoin | ChainType::Cardano => return Ok(None),
    };
    Ok(Some(public_key))
}

fn aptos_address_from_public_key(public_key: &[u8]) -> Result<String, AccountDerivationError> {
    let public_key = ed25519_public_key(public_key)?;
    let mut input = Vec::with_capacity(ED25519_PUBLIC_KEY_LEN + 1);
    input.extend_from_slice(&public_key);
    input.push(0x00);
    Ok(gem_aptos::AccountAddress::from_bytes(&sha3_256(&input))?.encode())
}

fn sui_address_from_public_key(public_key: &[u8]) -> Result<String, AccountDerivationError> {
    let public_key = ed25519_public_key(public_key)?;
    let mut input = Vec::with_capacity(ED25519_PUBLIC_KEY_LEN + 1);
    input.push(0x00);
    input.extend_from_slice(&public_key);
    Ok(gem_sui::address::SuiAddress::from_bytes(&blake2b_256(&input))
        .map_err(|error| AccountDerivationError::invalid_input(error.to_string()))?
        .encode())
}

fn cosmos_address_from_public_key(public_key: &[u8], chain: Chain) -> Result<String, AccountDerivationError> {
    let cosmos_chain = CosmosChain::from_chain(chain).ok_or_else(|| AccountDerivationError::unsupported_chain(chain))?;
    let public_key_hash = match cosmos_chain {
        CosmosChain::Injective => secp256k1_keccak_address_hash(public_key)?,
        CosmosChain::Cosmos | CosmosChain::Osmosis | CosmosChain::Celestia | CosmosChain::Thorchain | CosmosChain::Mayachain | CosmosChain::Sei | CosmosChain::Noble => {
            hash160(compressed_public_key(public_key)?)
        }
    };
    Ok(CosmosAddress::from_public_key_hash(chain, public_key_hash)
        .ok_or_else(|| AccountDerivationError::unsupported_chain(chain))?
        .encode())
}

fn bitcoin_address_from_private_key(private_key: &[u8], chain: Chain) -> Result<String, AccountDerivationError> {
    let bitcoin_chain = BitcoinChain::from_chain(chain).ok_or_else(|| AccountDerivationError::unsupported_chain(chain))?;
    let public_key = compressed_secp256k1_public_key(private_key)?;
    Ok(BitcoinAddress::from_public_key(bitcoin_chain, &public_key)?.encode())
}

fn uncompressed_secp256k1_public_key(private_key: &[u8]) -> Result<Vec<u8>, AccountDerivationError> {
    let public_key = secp256k1_uncompressed_public_key(private_key).map_err(|_| AccountDerivationError::InvalidPrivateKey)?;
    uncompressed_public_key(&public_key).map_err(|_| AccountDerivationError::InvalidPrivateKey)?;
    Ok(public_key)
}

fn compressed_secp256k1_public_key(private_key: &[u8]) -> Result<Vec<u8>, AccountDerivationError> {
    let public_key = secp256k1_public_key(private_key).map_err(|_| AccountDerivationError::InvalidPrivateKey)?;
    compressed_public_key(&public_key).map_err(|_| AccountDerivationError::InvalidPrivateKey)?;
    Ok(public_key)
}

fn uncompressed_public_key(public_key: &[u8]) -> Result<&[u8], AccountDerivationError> {
    if public_key.len() != SECP256K1_UNCOMPRESSED_PUBLIC_KEY_LEN || public_key.first() != Some(&SECP256K1_UNCOMPRESSED_PUBLIC_KEY_PREFIX) {
        return Err(AccountDerivationError::invalid_input("invalid uncompressed public key"));
    }
    Ok(public_key)
}

fn compressed_public_key(public_key: &[u8]) -> Result<&[u8], AccountDerivationError> {
    if public_key.len() != SECP256K1_COMPRESSED_PUBLIC_KEY_LEN {
        return Err(AccountDerivationError::invalid_input("invalid compressed public key"));
    }
    Ok(public_key)
}

fn ed25519_public_key(public_key: &[u8]) -> Result<[u8; ED25519_PUBLIC_KEY_LEN], AccountDerivationError> {
    public_key.try_into().map_err(|_| AccountDerivationError::invalid_input("invalid ed25519 public key"))
}

fn secp256k1_keccak_address_hash(public_key: &[u8]) -> Result<[u8; EVM_ADDRESS_LEN], AccountDerivationError> {
    let public_key = uncompressed_public_key(public_key)?;
    let hash = keccak256(&public_key[1..]);
    hash[hash.len() - EVM_ADDRESS_LEN..]
        .try_into()
        .map_err(|_| AccountDerivationError::invalid_input("invalid public key hash length"))
}
