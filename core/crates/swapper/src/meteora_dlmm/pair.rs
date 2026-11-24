use gem_encoding::borsh::{BorshDeserialize, DecodeError, Reader};
use gem_solana::{
    Pubkey, SolanaDataSlice,
    anchor::AnchorAccount,
    instructions::program_ids::{token_2022_program, token_program},
    models::Clock,
};

use super::bin::{BinArray, array_index};
use crate::{
    clmm::{accumulated_volatility, decayed_volatility, fee_amount},
    solana_dex::{ActivationType, DexPool, PoolLayout},
};

pub(super) const LAYOUT: PoolLayout = PoolLayout {
    account_len: 904,
    mint_offsets: [88, 120],
    depth: SolanaDataSlice {
        offset: 152,
        length: 2 * size_of::<Pubkey>(),
    },
};
const BITMAP_ARRAYS: i64 = 512;
const ENABLED: u8 = 0;
const PERMISSION: u8 = 1;
const CUSTOMIZABLE_PERMISSIONLESS: u8 = 2;
const LIQUIDITY_MINING: u8 = 1;
const LIMIT_ORDER: u8 = 2;
const FEE_ONLY_Y: u8 = 1;
const TOKEN_2022: u8 = 1;
const FEE_PRECISION: u32 = 1_000_000_000;
const MAX_FEE_RATE: u128 = 100_000_000;
const VARIABLE_FEE_PRECISION: u128 = 100_000_000_000;

#[derive(Debug, Clone, PartialEq)]
struct StaticParameters {
    base_factor: u16,
    filter_period: u16,
    decay_period: u16,
    reduction_factor: u16,
    variable_fee_control: u32,
    max_volatility_accumulator: u32,
    base_fee_power_factor: u8,
    function_type: u8,
    collect_fee_mode: u8,
}

impl BorshDeserialize for StaticParameters {
    fn deserialize_reader(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        let value = Self {
            base_factor: reader.read()?,
            filter_period: reader.read()?,
            decay_period: reader.read()?,
            reduction_factor: reader.read()?,
            variable_fee_control: reader.read()?,
            max_volatility_accumulator: reader.read()?,
            base_fee_power_factor: reader.skip(10)?.read()?,
            function_type: reader.read()?,
            collect_fee_mode: reader.read()?,
        };
        reader.skip(3)?;
        Ok(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
struct VariableParameters {
    volatility_accumulator: u32,
    volatility_reference: u32,
    index_reference: i32,
    last_update_timestamp: i64,
}

impl BorshDeserialize for VariableParameters {
    fn deserialize_reader(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        let value = Self {
            volatility_accumulator: reader.read()?,
            volatility_reference: reader.read()?,
            index_reference: reader.read()?,
            last_update_timestamp: reader.skip(4)?.read()?,
        };
        reader.skip(8)?;
        Ok(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
struct RewardInfo {
    mint: Pubkey,
}

impl BorshDeserialize for RewardInfo {
    fn deserialize_reader(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        let value = Self { mint: reader.read()? };
        reader.skip(112)?;
        Ok(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Pair {
    parameters: StaticParameters,
    v_parameters: VariableParameters,
    pair_type: u8,
    pub active_id: i32,
    pub bin_step: u16,
    status: u8,
    activation_type: ActivationType,
    pub token_x_mint: Pubkey,
    pub token_y_mint: Pubkey,
    pub reserve_x: Pubkey,
    pub reserve_y: Pubkey,
    reward_infos: [RewardInfo; 2],
    pub oracle: Pubkey,
    bin_array_bitmap: [u64; 16],
    activation_point: u64,
    token_mint_x_program_flag: u8,
    token_mint_y_program_flag: u8,
}

impl BorshDeserialize for Pair {
    fn deserialize_reader(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            parameters: reader.read()?,
            v_parameters: reader.read()?,
            pair_type: reader.skip(3)?.read()?,
            active_id: reader.read()?,
            bin_step: reader.read()?,
            status: reader.read()?,
            activation_type: reader.skip(3)?.read()?,
            token_x_mint: reader.skip(1)?.read()?,
            token_y_mint: reader.read()?,
            reserve_x: reader.read()?,
            reserve_y: reader.read()?,
            reward_infos: [reader.skip(48)?.read()?, reader.read()?],
            oracle: reader.read()?,
            bin_array_bitmap: reader.read()?,
            activation_point: reader.skip(104)?.read()?,
            token_mint_x_program_flag: reader.skip(56)?.read()?,
            token_mint_y_program_flag: reader.read()?,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Volatility {
    reference: u32,
    index_reference: i32,
}

impl AnchorAccount for Pair {
    const NAME: &'static str = "LbPair";
}

impl DexPool for Pair {
    fn mints(&self) -> (Pubkey, Pubkey) {
        (self.token_x_mint, self.token_y_mint)
    }

    fn token_vaults(&self) -> (Pubkey, Pubkey) {
        (self.reserve_x, self.reserve_y)
    }
}

impl Pair {
    fn arrays_with_liquidity(&self) -> u32 {
        self.bin_array_bitmap.iter().map(|limb| limb.count_ones()).sum()
    }

    pub fn is_tradable(&self, clock: &Clock) -> bool {
        self.status == ENABLED && self.is_open(clock) && (-BITMAP_ARRAYS..BITMAP_ARRAYS).contains(&array_index(self.active_id)) && self.arrays_with_liquidity() > 0
    }

    fn is_open(&self, clock: &Clock) -> bool {
        self.activation_type.current_point(clock).is_some_and(|now| match self.pair_type {
            PERMISSION | CUSTOMIZABLE_PERMISSIONLESS => now >= self.activation_point,
            _ => true,
        })
    }

    fn has_liquidity(&self, index: i64) -> bool {
        let Ok(bit) = usize::try_from(index + BITMAP_ARRAYS) else {
            return false;
        };
        self.bin_array_bitmap.get(bit / 64).is_some_and(|limb| (limb >> (bit % 64)) & 1 == 1)
    }

    pub fn arrays_for_swap(&self, swap_for_y: bool, count: usize) -> Vec<i64> {
        let start = array_index(self.active_id);
        let indexes = if swap_for_y { (-BITMAP_ARRAYS..=start).rev().collect::<Vec<_>>() } else { (start..BITMAP_ARRAYS).collect() };
        indexes.into_iter().filter(|index| self.has_liquidity(*index)).take(count).collect()
    }

    pub fn token_programs(&self) -> (Pubkey, Pubkey) {
        let program = |flag: u8| if flag == TOKEN_2022 { token_2022_program() } else { token_program() };
        (program(self.token_mint_x_program_flag), program(self.token_mint_y_program_flag))
    }

    fn has_limit_orders(&self) -> bool {
        match self.parameters.function_type {
            LIMIT_ORDER => true,
            LIQUIDITY_MINING => false,
            _ => self.reward_infos.iter().all(|reward| reward.mint == Pubkey::new([0; 32])),
        }
    }

    fn fee_on_input(&self, swap_for_y: bool) -> bool {
        self.parameters.collect_fee_mode != FEE_ONLY_Y || !swap_for_y
    }

    fn volatility(&self, now: i64) -> Volatility {
        let elapsed = now.saturating_sub(self.v_parameters.last_update_timestamp);
        if elapsed < i64::from(self.parameters.filter_period) {
            return Volatility {
                reference: self.v_parameters.volatility_reference,
                index_reference: self.v_parameters.index_reference,
            };
        }
        let reference = match elapsed < i64::from(self.parameters.decay_period) {
            true => decayed_volatility(self.v_parameters.volatility_accumulator, self.parameters.reduction_factor),
            false => 0,
        };
        Volatility { reference, index_reference: self.active_id }
    }

    fn fee_rate(&self, volatility: Volatility, bin_id: i32) -> Option<u32> {
        let parameters = &self.parameters;
        let accumulator = accumulated_volatility(volatility.reference, volatility.index_reference, bin_id, parameters.max_volatility_accumulator);
        let base_fee = u128::from(parameters.base_factor) * u128::from(self.bin_step) * 10 * 10u128.checked_pow(u32::from(parameters.base_fee_power_factor))?;
        let volatility_step = u128::from(accumulator) * u128::from(self.bin_step);
        let variable_fee = (u128::from(parameters.variable_fee_control) * volatility_step.checked_mul(volatility_step)?).div_ceil(VARIABLE_FEE_PRECISION);
        Some(u32::try_from((base_fee + variable_fee).min(MAX_FEE_RATE)).unwrap_or_default())
    }

    pub fn quote(&self, arrays: &[BinArray], swap_for_y: bool, amount_in: u64, clock: &Clock) -> Option<u64> {
        if self.status != ENABLED || !self.is_open(clock) {
            return None;
        }
        let volatility = self.volatility(clock.unix_timestamp);
        let has_limit_orders = self.has_limit_orders();
        let fee_on_input = self.fee_on_input(swap_for_y);
        let bins = arrays.iter().flat_map(|array| array.bin_ids(self.active_id, swap_for_y).into_iter().map(move |bin_id| (array, bin_id)));
        let (left, output) = bins.into_iter().try_fold((amount_in, 0u64), |(left, output), (array, bin_id)| {
            let (bin, versioned) = array.bin(bin_id)?;
            let has_limit_orders = has_limit_orders && versioned;
            if left == 0 || bin.max_out(swap_for_y, has_limit_orders) == 0 {
                return Some((left, output));
            }
            let rate = self.fee_rate(volatility, bin_id)?;
            let fee = if fee_on_input { fee_amount(left, rate, FEE_PRECISION) } else { 0 };
            let (unfilled, filled) = bin.fill(left - fee, bin.price(bin_id, self.bin_step)?, swap_for_y, has_limit_orders)?;
            let used = match (unfilled, fee_on_input) {
                (0, _) => left,
                (unfilled, true) => (left - fee - unfilled) + fee_amount(left - fee - unfilled, rate, FEE_PRECISION - rate),
                (unfilled, false) => left - unfilled,
            };
            let received = if fee_on_input { filled } else { filled - fee_amount(filled, rate, FEE_PRECISION) };
            Some((left.checked_sub(used)?, output.checked_add(received)?))
        })?;
        (left == 0 && output > 0).then_some(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::meteora_dlmm::testkit::{TEST_SOL_USDC_POOL, mock_pool_snapshot};
    use gem_solana::{USDC_TOKEN_MINT, WSOL_TOKEN_ADDRESS};

    const TEST_TOKEN_MINT_X_PROGRAM_FLAG_OFFSET: usize = 880;
    const TEST_TOKEN_MINT_Y_PROGRAM_FLAG_OFFSET: usize = 881;

    #[test]
    fn test_decode() {
        let pair = mock_pool_snapshot(TEST_SOL_USDC_POOL).pool;

        assert_eq!(
            [pair.token_x_mint, pair.token_y_mint, pair.reserve_x, pair.reserve_y, pair.oracle].map(|key| key.to_base58()),
            [
                WSOL_TOKEN_ADDRESS,
                USDC_TOKEN_MINT,
                "H7j5NPopj3tQvDg4N8CxwtYciTn3e8AEV6wSVrxpyDUc",
                "HbYjRzx7teCxqW3unpXBEcNHhfVZvW2vW9MQ99TkizWt",
                "EgEYXef2FCoEYLHJJW74dMbom1atLXo6KwPuA6mSATYA",
            ]
        );
    }

    #[test]
    fn test_token_programs() {
        let snapshot = mock_pool_snapshot(TEST_SOL_USDC_POOL);
        let data = snapshot.accounts[&snapshot.address].as_ref().unwrap();
        let mut token_2022_x = data.clone();
        token_2022_x[TEST_TOKEN_MINT_X_PROGRAM_FLAG_OFFSET] = TOKEN_2022;
        let mut token_2022_y = data.clone();
        token_2022_y[TEST_TOKEN_MINT_Y_PROGRAM_FLAG_OFFSET] = TOKEN_2022;

        assert_eq!(snapshot.pool.token_programs(), (token_program(), token_program()));
        assert_eq!(Pair::decode(&token_2022_x).unwrap().token_programs(), (token_2022_program(), token_program()));
        assert_eq!(Pair::decode(&token_2022_y).unwrap().token_programs(), (token_program(), token_2022_program()));
    }
}
