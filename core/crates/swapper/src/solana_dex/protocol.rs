use gem_encoding::borsh::{DecodeError, Reader};
use gem_solana::{
    DEFAULT_SWAP_GAS_LIMIT, Instruction, Pubkey, SolanaDataSlice, SolanaProgramAccountsFilter,
    models::{Clock, TokenAccount},
};

use super::token::{SOL_INTERMEDIATE, SwapToken};
use crate::{SwapperError, SwapperProvider};

const ACTIVATION_BY_SLOT: u8 = 0;
const ACTIVATION_BY_TIMESTAMP: u8 = 1;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum QuoteAccount {
    Owned { address: Pubkey, owner: Pubkey },
    Token(Pubkey),
}

impl QuoteAccount {
    pub fn address(&self) -> Pubkey {
        match self {
            Self::Owned { address, .. } | Self::Token(address) => *address,
        }
    }
}

pub(crate) fn account_data(account: &Option<Vec<u8>>) -> Result<&[u8], SwapperError> {
    account.as_deref().ok_or(SwapperError::NoQuoteAvailable)
}

pub(crate) fn token_amount(account: &Option<Vec<u8>>) -> Result<u64, SwapperError> {
    Ok(TokenAccount::decode(account_data(account)?)?.amount)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ActivationType(u8);

impl ActivationType {
    pub fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        reader.u8().map(Self)
    }

    pub fn current_point(&self, clock: &Clock) -> Option<u64> {
        match self.0 {
            ACTIVATION_BY_SLOT => Some(clock.slot),
            ACTIVATION_BY_TIMESTAMP => Some(clock.timestamp()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum PoolDepth {
    Value(u128),
    Reserves(Pubkey, Pubkey),
}

#[derive(Debug)]
pub(crate) struct PoolQuery {
    pub filters: Vec<Vec<SolanaProgramAccountsFilter>>,
    pub depth: SolanaDataSlice,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct PoolLayout {
    pub account_len: u64,
    pub mint_offsets: [usize; 2],
    pub depth: SolanaDataSlice,
}

impl PoolLayout {
    pub fn query(&self, mint_a: &Pubkey, mint_b: &Pubkey) -> PoolQuery {
        self.query_mints([(mint_a, mint_b), (mint_b, mint_a)])
    }

    pub fn sorted_query(&self, mint_a: &Pubkey, mint_b: &Pubkey) -> PoolQuery {
        self.query_mints([if mint_a.as_bytes() <= mint_b.as_bytes() { (mint_a, mint_b) } else { (mint_b, mint_a) }])
    }

    fn query_mints<'a>(&self, mints: impl IntoIterator<Item = (&'a Pubkey, &'a Pubkey)>) -> PoolQuery {
        let [offset_a, offset_b] = self.mint_offsets;
        let filters = mints.into_iter().map(|(mint_a, mint_b)| {
            vec![
                SolanaProgramAccountsFilter::DataSize(self.account_len),
                SolanaProgramAccountsFilter::Memcmp { offset: offset_a, bytes: mint_a.to_base58() },
                SolanaProgramAccountsFilter::Memcmp { offset: offset_b, bytes: mint_b.to_base58() },
            ]
        });
        PoolQuery {
            filters: filters.collect(),
            depth: self.depth,
        }
    }
}

#[derive(Debug)]
pub(crate) struct PoolSwap<'a> {
    pub authority: Pubkey,
    pub input: &'a SwapToken,
    pub intermediate: Option<&'a SwapToken>,
    pub output: &'a SwapToken,
    pub amount: u64,
    pub minimum_out: u64,
}

impl PoolSwap<'_> {
    pub fn amounts(&self) -> SwapAmounts {
        SwapAmounts {
            amount_in: self.amount,
            minimum_out: self.minimum_out,
        }
    }
}

pub(crate) struct SwapAmounts {
    pub amount_in: u64,
    pub minimum_out: u64,
}

impl SwapAmounts {
    pub fn encode(&self) -> Vec<u8> {
        [self.amount_in.to_le_bytes(), self.minimum_out.to_le_bytes()].concat()
    }
}

pub(crate) trait DexPool {
    fn mints(&self) -> (Pubkey, Pubkey);
    fn token_vaults(&self) -> (Pubkey, Pubkey);

    fn is_a_to_b(&self, input_mint: &Pubkey) -> Result<bool, SwapperError> {
        let (mint_a, mint_b) = self.mints();
        match (*input_mint == mint_a, *input_mint == mint_b) {
            (true, _) => Ok(true),
            (false, true) => Ok(false),
            (false, false) => Err(SwapperError::InvalidRoute),
        }
    }

    fn swap_direction(&self, input_mint: &Pubkey, output_mint: &Pubkey) -> Result<bool, SwapperError> {
        let a_to_b = self.is_a_to_b(input_mint)?;
        let (mint_a, mint_b) = self.mints();
        let other_mint = if a_to_b { mint_b } else { mint_a };
        if other_mint != *output_mint {
            return Err(SwapperError::InvalidRoute);
        }
        Ok(a_to_b)
    }

    fn vaults(&self, a_to_b: bool) -> (Pubkey, Pubkey) {
        let (vault_a, vault_b) = self.token_vaults();
        if a_to_b { (vault_a, vault_b) } else { (vault_b, vault_a) }
    }
}

pub(crate) trait NativeTwoHop<Pool> {
    fn liquidity(&self, pool: &Pool) -> u128;
    fn two_hop_swap_instruction(&self, first: &(Pubkey, Pool), second: &(Pubkey, Pool), swap: &PoolSwap, intermediate: &SwapToken) -> Result<Instruction, SwapperError>;
}

pub(crate) trait DexProtocol: Send + Sync {
    type Pool: Clone + PartialEq + Send + Sync;

    const PROVIDER: SwapperProvider;
    const PROGRAM_ID: Pubkey;
    const PROGRAM: &'static str;
    const INTERMEDIATE_TOKEN_ACCOUNT: bool = true;

    fn decode_pool(&self, data: &[u8]) -> Result<Self::Pool, SwapperError>;
    fn is_tradable(&self, pool: &Self::Pool, clock: &Clock) -> bool;

    fn pool_addresses(&self, _input_mint: &Pubkey, _output_mint: &Pubkey) -> Result<Vec<Pubkey>, SwapperError> {
        Ok(vec![])
    }

    fn constant_accounts(&self, _pool: &Self::Pool) -> Vec<Pubkey> {
        vec![]
    }

    fn compute_unit_limit(&self, _swap_instructions: usize) -> u32 {
        DEFAULT_SWAP_GAS_LIMIT
    }

    fn intermediate_tokens(&self) -> &'static [SwapToken] {
        &SOL_INTERMEDIATE
    }

    fn pool_query(&self, _input_mint: &Pubkey, _output_mint: &Pubkey) -> Option<PoolQuery> {
        None
    }

    fn pool_depth(&self, slice: &[u8]) -> PoolDepth {
        PoolDepth::Value(slice.iter().rev().fold(0, |depth, byte| depth << 8 | u128::from(*byte)))
    }

    fn quote_accounts(&self, address: &Pubkey, pool: &Self::Pool, input_mint: &Pubkey) -> Result<Vec<QuoteAccount>, SwapperError>;
    fn quote(&self, address: &Pubkey, pool: &Self::Pool, input_mint: &Pubkey, amount_in: u64, accounts: &[Option<Vec<u8>>], clock: &Clock) -> Result<u64, SwapperError>;
    fn swap_instruction(&self, address: &Pubkey, pool: &Self::Pool, swap: &PoolSwap) -> Result<Instruction, SwapperError>;

    fn native_two_hop(&self) -> Option<&dyn NativeTwoHop<Self::Pool>> {
        None
    }
}
