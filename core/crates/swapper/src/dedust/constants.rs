pub(super) const FACTORY: &str = "EQBfBWT7X2BHg9tXAxzhz2aKiNTU1tpt5NsiK0uSDW_YAJ67";
pub(super) const FACTORY_DEPLOY_CODE: &str =
    "te6ccgEBBAEAlgABFP8A9KQT9LzyyAsBAgJwAgMA9d4DoOmuQ/SAYEHaidqL2o8cMrcCAUDgsQAhkZYKA54sA/QFANeegZID9gHaz9rL2sji/9ojHHvaiaH0gaYOomWOC+XCBwgeCaY+AwQhNnVH9XQr5egHpn4CYammHgIDqEP2CEOh2j3apiCMIIsEAcpN2oex2oPb4gPl/wAJvyky+Dw=";
pub(super) const VAULT_CONTRACT_TYPE: u8 = 1;
pub(super) const POOL_CONTRACT_TYPE: u8 = 2;
pub(super) const UNINITIALIZED_EXIT_CODE: i32 = -13;

pub(super) const ESTIMATE_SWAP_OUT_METHOD: &str = "estimate_swap_out";

pub(super) const NATIVE_SWAP_OPCODE: u32 = 0xea06185d;
pub(super) const JETTON_SWAP_OPCODE: u32 = 0xe3a0d482;

pub(super) const SWAP_GAS: u64 = 250_000_000;
pub(super) const JETTON_SWAP_ATTACHMENT: u64 = 300_000_000;
pub(super) const FEE_JETTON_TRANSFER_ATTACHMENT: u64 = 100_000_000;
pub(super) const FEE_JETTON_FORWARD_AMOUNT: u64 = 1;
