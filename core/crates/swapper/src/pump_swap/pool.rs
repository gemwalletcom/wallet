use borsh::BorshDeserialize;
use gem_solana::{Pubkey, anchor::AnchorAccount, find_program_address, instructions::program_ids::pump_amm_program, wrapped_sol::wrapped_sol_mint};
use primitives::decode_hex_literal;
use std::iter::repeat;

use super::fee_config::{FeeConfig, Fees};
use crate::{
    SwapperError,
    clmm::constant_product_output,
    solana_dex::{DexPool, USDC_MINT},
};

const PUMP_PROGRAM: Pubkey = Pubkey::new(decode_hex_literal("0156e0f693665acf44db1568bf175baa5189cb97f5d2ff3b655d2bb6fd6d18b0"));
const CANONICAL_POOL_INDEX: u16 = 0;
const POOL_ACCOUNT_LEN: usize = 287;
const MAYHEM_TOKEN_SUPPLY: u128 = 1_000_000_000_000_000;
const NO_COIN_CREATOR: Pubkey = Pubkey::new([0; 32]);

#[derive(Debug, Clone, PartialEq, BorshDeserialize)]
pub(crate) struct Pool {
    _bump_and_index: [u8; 3],
    _creator: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    _lp_mint: Pubkey,
    pub pool_base_token_account: Pubkey,
    pub pool_quote_token_account: Pubkey,
    pub lp_supply: u64,
    coin_creator: Pubkey,
    is_mayhem_mode: bool,
    pub is_cashback_coin: bool,
    virtual_quote_reserves: i128,
    _creator_fee_bps_and_flags: [u8; 10],
    protocol_fees: u64,
    creator_fees: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Reserves {
    base: u128,
    quote: u128,
    real_quote: u128,
}

impl AnchorAccount for Pool {
    const NAME: &'static str = "Pool";
}

impl DexPool for Pool {
    fn mints(&self) -> (Pubkey, Pubkey) {
        (self.base_mint, self.quote_mint)
    }

    fn token_vaults(&self) -> (Pubkey, Pubkey) {
        (self.pool_base_token_account, self.pool_quote_token_account)
    }
}

impl Pool {
    pub fn addresses(mint_a: &Pubkey, mint_b: &Pubkey) -> Result<Vec<Pubkey>, SwapperError> {
        let quote_mints = [wrapped_sol_mint(), USDC_MINT];
        [(mint_a, mint_b), (mint_b, mint_a)]
            .into_iter()
            .filter(|(base_mint, quote_mint)| quote_mints.contains(quote_mint) && !quote_mints.contains(base_mint))
            .map(|(base_mint, quote_mint)| Self::canonical_address(base_mint, quote_mint))
            .collect()
    }

    fn canonical_address(base_mint: &Pubkey, quote_mint: &Pubkey) -> Result<Pubkey, SwapperError> {
        let creator = find_program_address(&PUMP_PROGRAM, &[b"pool-authority".as_slice(), base_mint.as_bytes()])?.0;
        let seeds = [b"pool".as_slice(), &CANONICAL_POOL_INDEX.to_le_bytes(), creator.as_bytes(), base_mint.as_bytes(), quote_mint.as_bytes()];
        Ok(find_program_address(&pump_amm_program(), &seeds)?.0)
    }

    pub fn decode_padded(data: &[u8]) -> Result<Self, SwapperError> {
        Ok(Self::decode(&data.iter().copied().chain(repeat(0)).take(POOL_ACCOUNT_LEN).collect::<Vec<_>>())?)
    }

    pub(super) fn reserves(&self, base_vault: u64, quote_vault: u64) -> Option<Reserves> {
        Some(Reserves {
            base: u128::from(base_vault),
            quote: u128::try_from(i128::from(quote_vault).checked_add(self.virtual_quote_reserves)?).ok()?,
            real_quote: u128::from(quote_vault).checked_sub(u128::from(self.protocol_fees) + u128::from(self.creator_fees))?,
        })
    }

    pub(super) fn fees(&self, fee_config: &FeeConfig, base_supply: u64, reserves: &Reserves) -> Option<Fees> {
        let supply = if self.is_mayhem_mode { MAYHEM_TOKEN_SUPPLY } else { u128::from(base_supply) };
        let fees = fee_config.tier_fees(&self.quote_mint, reserves.quote.checked_mul(supply)?.checked_div(reserves.base)?)?;
        if self.coin_creator == NO_COIN_CREATOR { Some(Fees { creator_fee_bps: 0, ..fees }) } else { Some(fees) }
    }
}

impl Reserves {
    pub fn sell(&self, base_in: u64, fees: &Fees) -> Option<u64> {
        let base_in = u128::from(base_in);
        let quote_out = constant_product_output(self.base, self.quote, base_in)?;
        let without_lp_fee = quote_out.checked_sub(fees.lp_fee(quote_out)).filter(|value| *value <= self.real_quote)?;
        u64::try_from(without_lp_fee.checked_sub(fees.protocol_and_creator_fees(quote_out))?).ok()
    }

    pub fn buy(&self, quote_in: u64, fees: &Fees) -> Option<u64> {
        let input = fees.quote_after_fees(u128::from(quote_in))?.checked_sub(1)?;
        u64::try_from(constant_product_output(self.quote, self.base, input)?).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        pump_swap::testkit::{TEST_TOKEN_2022_MINT, TEST_USDC_POOL, mock_pool_snapshot},
        solana_dex::testkit::TEST_TOS_MINT,
    };

    #[test]
    fn test_addresses() {
        let key = |address: &str| Pubkey::from_base58(address).unwrap();
        let tos = key(TEST_TOS_MINT);
        let usdc = USDC_MINT;
        let sol_pool = key("9htNhHVtWrPiRdcSyBZoAhtLRZ4NKxxNE8qdkDCZx57f");

        assert_eq!(Pool::addresses(&tos, &wrapped_sol_mint()).unwrap(), vec![sol_pool]);
        assert_eq!(Pool::addresses(&wrapped_sol_mint(), &tos).unwrap(), vec![sol_pool]);
        assert_eq!(Pool::addresses(&key(TEST_TOKEN_2022_MINT), &usdc).unwrap(), vec![key("AKQyQgEifBBhQmANuKzrRuRGoKSkKBv8NVdccc87FrR8")]);
        assert_eq!(Pool::addresses(&wrapped_sol_mint(), &usdc).unwrap(), vec![]);
        assert_eq!(Pool::addresses(&tos, &key("DtcicPCKTKeAh7TDEzTym3hXeoHAoMiBdYJhMYxZpump")).unwrap(), vec![]);
    }

    #[test]
    fn test_decode_padded() {
        let snapshot = mock_pool_snapshot(TEST_USDC_POOL);
        let data = snapshot.accounts[&snapshot.address].as_ref().unwrap();

        assert_eq!(snapshot.pool.virtual_quote_reserves, -412_575);
        assert_eq!(
            Pool::decode_padded(&data[..243]).unwrap(),
            Pool {
                virtual_quote_reserves: 0,
                protocol_fees: 0,
                creator_fees: 0,
                ..snapshot.pool.clone()
            }
        );
        assert_eq!(Pool::decode_padded(&data[..211]).unwrap().coin_creator, NO_COIN_CREATOR);
    }
}
