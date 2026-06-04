//! BLAKE2b-256 personalization tags for Zcash ZIP-244 transparent transaction signing.
//! Each tag is 16 bytes. See https://zips.z.cash/zip-0244.

// Per-component txid digests.
pub(super) const ZCASH_TXID_HEADERS_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxIdHeadersHash";
pub(super) const ZCASH_TXID_PREVOUTS_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxIdPrevoutHash";
pub(super) const ZCASH_TXID_SEQUENCES_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxIdSequencHash";
pub(super) const ZCASH_TXID_OUTPUTS_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxIdOutputsHash";
pub(super) const ZCASH_TXID_SAPLING_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxIdSaplingHash";
pub(super) const ZCASH_TXID_ORCHARD_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxIdOrchardHash";

// Transparent sighash digests.
pub(super) const ZCASH_TXID_TRANSPARENT_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxIdTranspaHash";
pub(super) const ZCASH_TRANSPARENT_AMOUNTS_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxTrAmountsHash";
pub(super) const ZCASH_TRANSPARENT_SCRIPTS_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxTrScriptsHash";
pub(super) const ZCASH_TXIN_HASH_PERSONALIZATION: &[u8; 16] = b"Zcash___TxInHash";

// 12-byte prefix; the final 4 bytes are the little-endian consensus branch id.
pub(super) const ZCASH_TX_HASH_PERSONALIZATION_PREFIX: &[u8; 12] = b"ZcashTxHash_";
