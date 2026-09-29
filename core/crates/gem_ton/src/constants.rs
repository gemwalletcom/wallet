pub const JETTON_TRANSFER_OPCODE: u32 = 0x0f8a7ea5;
#[cfg(feature = "signer")]
pub(crate) const NFT_TRANSFER_OPCODE: u32 = 0x5fcc3d14;

#[cfg(feature = "signer")]
pub(crate) const NFT_TRANSFER_FORWARD_AMOUNT: u64 = 10_000_000;
#[cfg(any(test, feature = "rpc"))]
pub(crate) const NFT_TRANSFER_ATTACHMENT: u64 = 50_000_000;

pub const TON_PROXY_JETTON_ADDRESS: &str = "EQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAM9c";
#[cfg(feature = "rpc")]
pub(crate) const STONFI_PTON_ADDRESSES: &[&str] = &["EQCM3B12QK1e4yZSf8GtBRT0aLMNyEsBc_DhVfRRtOEffLez", "EQBnGWMCf3-FZZq1W4IWcWiGAc3PHuZ0_H-7sad2oY00o83S"];

pub const FAILED_OPERATION_OPCODES: &[&str] = &["0x93be2305", "0xd6182fce", "0x77d0fee6", "0x98ce9044"];
