use primitives::{Account, Chain};

// sui_getAccounts example from the WalletConnect Sui RPC reference.
pub const TEST_SUI_ADDRESS: &str = "0x3cd077f41680eebca0176baad3915b2ea26dbbdfd10161865234732bb1f2ac50";
pub const TEST_SUI_PUBLIC_KEY_HEX: &str = "2ebc3f9e960824c5d275045f7d7f5796f5c2a883d69bdf73c16a97c74f20f0c0";
pub const TEST_SUI_PUBLIC_KEY_BASE64: &str = "AC68P56WCCTF0nUEX31/V5b1wqiD1pvfc8Fql8dPIPDA";
pub const TEST_TON_ADDRESS: &str = "UQD-E9MxmtMJhnkRdi0tLoZ_TK0xr2WoxD5g2adbjE0G50Ud";
pub const TEST_TON_PUBLIC_KEY: &str = "d369452197c2a56481e5e2d3e8bf03de2349f67a63151956822208c2334adee2";

pub fn mock_sui_account() -> Account {
    Account {
        chain: Chain::Sui,
        address: TEST_SUI_ADDRESS.to_string(),
        derivation_path: "m/44'/784'/0'/0'/0'".to_string(),
        extended_public_key: Some(TEST_SUI_PUBLIC_KEY_HEX.to_string()),
    }
}

pub fn mock_ton_account() -> Account {
    Account {
        chain: Chain::Ton,
        address: TEST_TON_ADDRESS.to_string(),
        derivation_path: "m/44'/607'/0'".to_string(),
        extended_public_key: Some(TEST_TON_PUBLIC_KEY.to_string()),
    }
}
