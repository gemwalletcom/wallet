use gem_encoding::borsh::{BorshDeserialize, DecodeError, Reader};
use gem_solana::{Pubkey, SolanaDataSlice, anchor::AnchorAccount};

use crate::{
    clmm::{constant_product_output, fee_amount},
    solana_dex::{DexPool, PoolLayout},
};

pub(super) const LAYOUT: PoolLayout = PoolLayout {
    account_len: 637,
    mint_offsets: [168, 200],
    depth: SolanaDataSlice { offset: 333, length: size_of::<u64>() },
};
const FEE_RATE_DENOMINATOR: u32 = 1_000_000;
const SWAP_DISABLED: u8 = 1 << 2;
const LOCKED_LP_SUPPLY: u64 = 100;
const CREATOR_FEE_ON_BOTH_TOKENS: u8 = 0;
const CREATOR_FEE_ON_TOKEN_0: u8 = 1;
const CREATOR_FEE_ON_TOKEN_1: u8 = 2;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Pool {
    pub amm_config: Pubkey,
    pub token_0_vault: Pubkey,
    pub token_1_vault: Pubkey,
    pub token_0_mint: Pubkey,
    pub token_1_mint: Pubkey,
    pub observation_key: Pubkey,
    status: u8,
    pub lp_supply: u64,
    protocol_fees_token_0: u64,
    protocol_fees_token_1: u64,
    fund_fees_token_0: u64,
    fund_fees_token_1: u64,
    open_time: u64,
    creator_fee_on: u8,
    enable_creator_fee: u8,
    creator_fees_token_0: u64,
    creator_fees_token_1: u64,
}

impl BorshDeserialize for Pool {
    fn deserialize_reader(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            amm_config: reader.read()?,
            token_0_vault: reader.skip(32)?.read()?,
            token_1_vault: reader.read()?,
            token_0_mint: reader.skip(32)?.read()?,
            token_1_mint: reader.read()?,
            observation_key: reader.skip(64)?.read()?,
            status: reader.skip(1)?.read()?,
            lp_supply: reader.skip(3)?.read()?,
            protocol_fees_token_0: reader.read()?,
            protocol_fees_token_1: reader.read()?,
            fund_fees_token_0: reader.read()?,
            fund_fees_token_1: reader.read()?,
            open_time: reader.read()?,
            creator_fee_on: reader.skip(8)?.read()?,
            enable_creator_fee: reader.read()?,
            creator_fees_token_0: reader.skip(6)?.read()?,
            creator_fees_token_1: reader.read()?,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct AmmConfig {
    trade_fee_rate: u64,
    creator_fee_rate: u64,
}

impl BorshDeserialize for AmmConfig {
    fn deserialize_reader(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            trade_fee_rate: reader.skip(4)?.read()?,
            creator_fee_rate: reader.skip(88)?.read()?,
        })
    }
}

impl AnchorAccount for Pool {
    const NAME: &'static str = "PoolState";
}

impl AnchorAccount for AmmConfig {
    const NAME: &'static str = "AmmConfig";
}

impl DexPool for Pool {
    fn mints(&self) -> (Pubkey, Pubkey) {
        (self.token_0_mint, self.token_1_mint)
    }

    fn token_vaults(&self) -> (Pubkey, Pubkey) {
        (self.token_0_vault, self.token_1_vault)
    }
}

impl Pool {
    pub fn is_tradable(&self, timestamp: u64) -> bool {
        self.status & SWAP_DISABLED == 0 && timestamp >= self.open_time && self.lp_supply > LOCKED_LP_SUPPLY
    }

    pub fn quote(&self, config: &AmmConfig, zero_for_one: bool, amount_in: u64, vault_0: u64, vault_1: u64) -> Option<u64> {
        let reserve_0 = vault_0.checked_sub(self.protocol_fees_token_0.checked_add(self.fund_fees_token_0)?.checked_add(self.creator_fees_token_0)?)?;
        let reserve_1 = vault_1.checked_sub(self.protocol_fees_token_1.checked_add(self.fund_fees_token_1)?.checked_add(self.creator_fees_token_1)?)?;
        let (reserve_in, reserve_out) = if zero_for_one { (reserve_0, reserve_1) } else { (reserve_1, reserve_0) };
        let trade_fee_rate = u32::try_from(config.trade_fee_rate).ok()?;
        let creator_fee_rate = if self.enable_creator_fee == 0 { 0 } else { u32::try_from(config.creator_fee_rate).ok()? };
        let creator_fee_on_input = match self.creator_fee_on {
            CREATOR_FEE_ON_BOTH_TOKENS => true,
            CREATOR_FEE_ON_TOKEN_0 => zero_for_one,
            CREATOR_FEE_ON_TOKEN_1 => !zero_for_one,
            _ => return None,
        };
        let input_fee_rate = if creator_fee_on_input {
            trade_fee_rate.checked_add(creator_fee_rate).filter(|rate| *rate > 0)?
        } else {
            trade_fee_rate
        };
        let swap_in = u128::from(amount_in.checked_sub(fee_amount(amount_in, input_fee_rate, FEE_RATE_DENOMINATOR))?);
        let swapped = u64::try_from(constant_product_output(u128::from(reserve_in), u128::from(reserve_out), swap_in)?).ok()?;
        let output = if creator_fee_on_input {
            swapped
        } else {
            swapped.checked_sub(fee_amount(swapped, creator_fee_rate, FEE_RATE_DENOMINATOR))?
        };
        (output > 0).then_some(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::raydium_cpmm::testkit::{TEST_JUP_SOL_POOL, mock_pool_snapshot};
    use gem_solana::{models::TokenAccount, wrapped_sol::wrapped_sol_mint};

    const TEST_JUP_MINT: &str = "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN";

    fn config(snapshot: &crate::solana_dex::testkit::PoolSnapshot<Pool>) -> AmmConfig {
        AmmConfig::decode(snapshot.accounts[&snapshot.pool.amm_config].as_ref().unwrap()).unwrap()
    }

    #[test]
    fn test_layout_offsets() {
        let snapshot = mock_pool_snapshot(TEST_JUP_SOL_POOL);
        let data = snapshot.accounts[&snapshot.address].as_ref().unwrap();
        let at = |offset: usize, len: usize| &data[offset..offset + len];
        let config = config(&snapshot);

        assert_eq!(u64::try_from(data.len()).unwrap(), LAYOUT.account_len);
        assert_eq!(at(LAYOUT.mint_offsets[0], 32), Pubkey::from_base58(TEST_JUP_MINT).unwrap().as_bytes());
        assert_eq!(at(LAYOUT.mint_offsets[1], 32), wrapped_sol_mint().as_bytes());
        assert_eq!((snapshot.pool.token_0_mint, snapshot.pool.token_1_mint), (Pubkey::from_base58(TEST_JUP_MINT).unwrap(), wrapped_sol_mint()));
        assert_eq!(at(LAYOUT.depth.offset, LAYOUT.depth.length), snapshot.pool.lp_supply.to_le_bytes());
        assert_eq!((config.trade_fee_rate, config.creator_fee_rate), (2_500, 500));
    }

    #[test]
    fn test_quote_matches_the_program() {
        let snapshot = mock_pool_snapshot(TEST_JUP_SOL_POOL);
        let config = config(&snapshot);
        let pool = snapshot.pool.clone();
        let vault = |address: &Pubkey| TokenAccount::decode(snapshot.accounts[address].as_ref().unwrap()).unwrap().amount;
        let (vault_0, vault_1) = (vault(&pool.token_0_vault), vault(&pool.token_1_vault));
        let quote = |pool: &Pool, zero_for_one: bool, amount_in: u64| pool.quote(&config, zero_for_one, amount_in, vault_0, vault_1);
        let creator_fee_on = |creator_fee_on: u8| Pool {
            creator_fee_on,
            enable_creator_fee: 1,
            ..pool.clone()
        };

        assert_eq!(quote(&pool, false, 1_000_000_000), Some(342_176_339));
        assert_eq!(quote(&pool, true, 100_000_000), Some(275_611_957));
        assert_eq!(quote(&pool, false, 1_000_000_000_000), Some(8_176_440_009));
        assert_eq!(quote(&creator_fee_on(CREATOR_FEE_ON_BOTH_TOKENS), false, 1_000_000_000), Some(342_011_832));
        assert_eq!(quote(&creator_fee_on(CREATOR_FEE_ON_TOKEN_0), false, 1_000_000_000), Some(342_005_250));
        assert_eq!(quote(&creator_fee_on(CREATOR_FEE_ON_TOKEN_1), false, 1_000_000_000), Some(342_011_832));
        assert_eq!(quote(&pool, true, 0), None);
        assert_eq!(quote(&creator_fee_on(3), true, 100_000_000), None);
        assert_eq!(pool.quote(&config, true, 100_000_000, 0, vault_1), None);
    }

    #[test]
    fn test_is_tradable() {
        let pool = mock_pool_snapshot(TEST_JUP_SOL_POOL).pool;
        let open_time = pool.open_time;

        assert!(pool.is_tradable(open_time));
        assert!(!pool.is_tradable(open_time - 1));
        assert!(Pool { status: 1 | 2, ..pool }.is_tradable(open_time));
        assert!(!Pool { status: SWAP_DISABLED, ..pool }.is_tradable(open_time));
        assert!(!Pool { lp_supply: LOCKED_LP_SUPPLY, ..pool }.is_tradable(open_time));
    }
}
