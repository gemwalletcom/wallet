use crate::{
    instructions::{program_ids::compute_budget_program, system::is_advance_nonce_account},
    types::Instruction,
};

pub const SET_COMPUTE_UNIT_LIMIT_DISCRIMINANT: u8 = 2;
pub const SET_COMPUTE_UNIT_PRICE_DISCRIMINANT: u8 = 3;

pub fn set_compute_unit_price(micro_lamports: u64) -> Instruction {
    let mut data = Vec::with_capacity(9);
    data.push(SET_COMPUTE_UNIT_PRICE_DISCRIMINANT);
    data.extend_from_slice(&micro_lamports.to_le_bytes());
    Instruction {
        program_id: compute_budget_program(),
        accounts: vec![],
        data,
    }
}

pub fn set_compute_unit_limit(units: u32) -> Instruction {
    let mut data = Vec::with_capacity(5);
    data.push(SET_COMPUTE_UNIT_LIMIT_DISCRIMINANT);
    data.extend_from_slice(&units.to_le_bytes());
    Instruction {
        program_id: compute_budget_program(),
        accounts: vec![],
        data,
    }
}

pub fn parse_compute_unit_limit_data(data: &[u8]) -> Option<u32> {
    let bytes = data.strip_prefix(&[SET_COMPUTE_UNIT_LIMIT_DISCRIMINANT])?;
    Some(u32::from_le_bytes(<[u8; 4]>::try_from(bytes.get(..4)?).ok()?))
}

pub fn parse_compute_unit_price_data(data: &[u8]) -> Option<u64> {
    let bytes = data.strip_prefix(&[SET_COMPUTE_UNIT_PRICE_DISCRIMINANT])?;
    Some(u64::from_le_bytes(<[u8; 8]>::try_from(bytes.get(..8)?).ok()?))
}

pub(crate) fn find_unique_compute_unit_limit<'a>(instruction_data: impl Iterator<Item = &'a [u8]>) -> Option<u32> {
    let mut limits = instruction_data.filter_map(parse_compute_unit_limit_data);
    let limit = limits.next()?;
    limits.next().is_none().then_some(limit)
}

pub(crate) fn find_unique_compute_unit_price<'a>(instruction_data: impl Iterator<Item = &'a [u8]>) -> Option<u64> {
    let mut prices = instruction_data.filter_map(parse_compute_unit_price_data);
    let price = prices.next()?;
    prices.next().is_none().then_some(price)
}

pub fn get_compute_unit_limit(instructions: &[Instruction]) -> Option<u32> {
    find_unique_compute_unit_limit(instructions.iter().filter(|instruction| instruction.program_id == compute_budget_program()).map(|instruction| instruction.data.as_slice()))
}

pub fn with_compute_unit_price(instructions: Vec<Instruction>, micro_lamports: u64) -> Vec<Instruction> {
    let has_price = instructions
        .iter()
        .any(|instruction| instruction.program_id == compute_budget_program() && parse_compute_unit_price_data(&instruction.data).is_some());
    if has_price {
        return instructions;
    }
    let (nonce, rest) = instructions.split_at(usize::from(instructions.first().is_some_and(is_advance_nonce_account)));
    [nonce, &[set_compute_unit_price(micro_lamports)], rest].concat()
}

#[cfg(test)]
mod tests {
    use primitives::hex;

    use super::*;
    use crate::Pubkey;
    use crate::instructions::program_ids::system_program;
    use crate::instructions::system::{ADVANCE_NONCE_ACCOUNT_DISCRIMINANT, transfer};

    #[test]
    fn test_compute_budget_wire_formats_and_parsing() {
        let limit = set_compute_unit_limit(200_000);
        assert_eq!(limit.data, hex::decode("02400d0300").unwrap());
        assert_eq!(parse_compute_unit_limit_data(&limit.data), Some(200_000));
        assert_eq!(parse_compute_unit_limit_data(&hex::decode("02400d0300ff").unwrap()), Some(200_000));
        assert_eq!(parse_compute_unit_limit_data(&[2, 1]), None);

        let price = set_compute_unit_price(1_000);
        assert_eq!(price.data, hex::decode("03e803000000000000").unwrap());
        assert_eq!(parse_compute_unit_price_data(&price.data), Some(1_000));
        assert_eq!(parse_compute_unit_price_data(&hex::decode("03e803000000000000ff").unwrap()), Some(1_000));
        assert_eq!(parse_compute_unit_price_data(&[3, 1]), None);
    }

    #[test]
    fn test_get_compute_unit_limit() {
        let limit = set_compute_unit_limit(200_000);
        let price = set_compute_unit_price(1_000);

        assert_eq!(get_compute_unit_limit(&[price.clone(), limit.clone()]), Some(200_000));
        assert_eq!(get_compute_unit_limit(&[limit.clone(), set_compute_unit_limit(300_000)]), None);
        assert_eq!(get_compute_unit_limit(&[price]), None);
        assert_eq!(
            get_compute_unit_limit(&[Instruction {
                program_id: system_program(),
                accounts: vec![],
                data: limit.data,
            }]),
            None
        );
    }

    #[test]
    fn test_compute_unit_price_insertion_preserves_nonce_order() {
        let payer = Pubkey::new([1; 32]);
        let recipient = Pubkey::new([2; 32]);
        let transfer = transfer(&payer, &recipient, 10);
        let priced = with_compute_unit_price(vec![transfer.clone()], 5_000);

        assert_eq!(priced, vec![set_compute_unit_price(5_000), transfer]);
        assert_eq!(with_compute_unit_price(priced.clone(), 9_999), priced);

        let nonce = Instruction {
            program_id: system_program(),
            accounts: vec![],
            data: ADVANCE_NONCE_ACCOUNT_DISCRIMINANT.to_vec(),
        };
        assert_eq!(with_compute_unit_price(vec![nonce.clone()], 5_000), vec![nonce, set_compute_unit_price(5_000)]);
    }
}
