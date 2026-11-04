use crate::Pubkey;
pub use primitives::contract_constants::{SOLANA_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID, SOLANA_COMPUTE_BUDGET_PROGRAM_ID, SOLANA_MEMO_PROGRAM_ID, SOLANA_SYSTEM_PROGRAM_ID, SOLANA_TOKEN_2022_PROGRAM_ID, SOLANA_TOKEN_PROGRAM_ID};

macro_rules! program_id {
    ($name:ident, $hex:literal) => {
        pub const fn $name() -> Pubkey {
            Pubkey::new(hex_lit::hex!($hex))
        }
    };
}

program_id!(system_program, "0000000000000000000000000000000000000000000000000000000000000000");
program_id!(token_program, "06ddf6e1d765a193d9cbe146ceeb79ac1cb485ed5f5b37913a8cf5857eff00a9");
program_id!(token_2022_program, "06ddf6e1ee758fde18425dbce46ccddab61afc4d83b90d27febdf928d8a18bfc");
program_id!(associated_token_program, "8c97258f4e2489f1bb3d1029148e0d830b5a1399daff1084048e7bd8dbe9f859");
program_id!(memo_program, "054a535a992921064d24e87160da387c7c35b5ddbc92bb81e41fa8404105448d");
program_id!(compute_budget_program, "0306466fe5211732ffecadba72c39be7bc8ce5bbc5f7126b2c439b3a40000000");
program_id!(sysvar_program, "06a7d5171875f729c73d93408f216120067ed88c76e08c287fc1946000000000");
program_id!(raydium_clmm_program, "a5d5ca9e04cf5db590b714ba2fe32cb159133fc1c192b72257fd07d39cb0401e");
program_id!(pump_amm_program, "0c14defc825ec67694250818bb654065f4298d3156d571b4d4f8090c18e9a863");
program_id!(meteora_damm_v2_program, "092d2135657a159c2b87d4b66a70db8e9752389ff76aaf206ced063a38f95aed");
program_id!(meteora_dlmm_program, "04e9e12fbc84e826c932cce9e2640cce15590c1c6273b0925708ba3b8520b0bc");
program_id!(raydium_amm_program, "4bd949c43602c33f207790ed16a3524ca1b9975cf121a2a90cffec7df8b68acd");
program_id!(raydium_cpmm_program, "a92a5a8b4f295952842550aa93fd5b95b5ace6a8eb920c93942e43690c20ec73");
program_id!(orca_whirlpool_program, "0e03685f8e909053e458121c66f5a76aedc7706aa11c82f8aa952a8f2b7879a9");
program_id!(sysvar_clock, "06a7d51718c774c928566398691d5eb68b5eb8a39b4b6d5c73555b2100000000");

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{METEORA_DAMM_V2_PROGRAM_ID, METEORA_DLMM_PROGRAM_ID, ORCA_WHIRLPOOL_PROGRAM_ID, PUMP_AMM_PROGRAM_ID, RAYDIUM_AMM_PROGRAM_ID, RAYDIUM_CLMM_PROGRAM_ID, RAYDIUM_CPMM_PROGRAM_ID};

    #[test]
    fn test_pool_program_ids_match_their_addresses() {
        assert_eq!(meteora_dlmm_program().to_base58(), METEORA_DLMM_PROGRAM_ID);
        assert_eq!(raydium_amm_program().to_base58(), RAYDIUM_AMM_PROGRAM_ID);
        assert_eq!(raydium_cpmm_program().to_base58(), RAYDIUM_CPMM_PROGRAM_ID);
        assert_eq!(raydium_clmm_program().to_base58(), RAYDIUM_CLMM_PROGRAM_ID);
        assert_eq!(pump_amm_program().to_base58(), PUMP_AMM_PROGRAM_ID);
        assert_eq!(meteora_damm_v2_program().to_base58(), METEORA_DAMM_V2_PROGRAM_ID);
        assert_eq!(orca_whirlpool_program().to_base58(), ORCA_WHIRLPOOL_PROGRAM_ID);
    }
}
