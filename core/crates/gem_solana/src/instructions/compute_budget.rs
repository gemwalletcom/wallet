use hex_lit::hex;

use crate::{
    instructions::program_ids::{compute_budget_program, system_program},
    types::Instruction,
};

pub const SET_COMPUTE_UNIT_LIMIT_DISCRIMINANT: u8 = 2;
pub const SET_COMPUTE_UNIT_PRICE_DISCRIMINANT: u8 = 3;
const ADVANCE_NONCE_ACCOUNT_DISCRIMINANT: [u8; 4] = hex!("04000000");

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
    u32::from_le_bytes(bytes.try_into().ok()?).into()
}

pub fn parse_compute_unit_price_data(data: &[u8]) -> Option<u64> {
    let bytes = data.strip_prefix(&[SET_COMPUTE_UNIT_PRICE_DISCRIMINANT])?;
    u64::from_le_bytes(bytes.try_into().ok()?).into()
}

pub fn get_compute_unit_limit(instructions: &[Instruction]) -> Option<u32> {
    instructions
        .iter()
        .find_map(|instruction| (instruction.program_id == compute_budget_program()).then(|| parse_compute_unit_limit_data(&instruction.data)).flatten())
}

pub fn ensure_compute_unit_price(instructions: &mut Vec<Instruction>, micro_lamports: u64) -> bool {
    let has_price = instructions
        .iter()
        .any(|instruction| instruction.program_id == compute_budget_program() && parse_compute_unit_price_data(&instruction.data).is_some());
    if has_price {
        return false;
    }

    let insertion_index = usize::from(
        instructions
            .first()
            .is_some_and(|instruction| instruction.program_id == system_program() && instruction.data.get(0..4) == Some(&ADVANCE_NONCE_ACCOUNT_DISCRIMINANT)),
    );
    instructions.insert(insertion_index, set_compute_unit_price(micro_lamports));
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Pubkey;
    use crate::instructions::system::transfer;

    #[test]
    fn test_compute_budget_wire_formats_and_parsing() {
        let limit = set_compute_unit_limit(200_000);
        assert_eq!(limit.data, hex!("02400d0300"));
        assert_eq!(parse_compute_unit_limit_data(&limit.data), Some(200_000));
        assert_eq!(parse_compute_unit_limit_data(&[2, 1]), None);

        let price = set_compute_unit_price(1_000);
        assert_eq!(price.data, hex!("03e803000000000000"));
        assert_eq!(parse_compute_unit_price_data(&price.data), Some(1_000));
        assert_eq!(get_compute_unit_limit(&[price, limit]), Some(200_000));
    }

    #[test]
    fn test_compute_unit_price_insertion_preserves_nonce_order() {
        let payer = Pubkey::new([1; 32]);
        let recipient = Pubkey::new([2; 32]);
        let transfer = transfer(&payer, &recipient, 10);
        let mut instructions = vec![transfer];

        assert!(ensure_compute_unit_price(&mut instructions, 5_000));
        assert_eq!(instructions[0], set_compute_unit_price(5_000));
        assert!(!ensure_compute_unit_price(&mut instructions, 9_999));

        let nonce = Instruction {
            program_id: system_program(),
            accounts: vec![],
            data: ADVANCE_NONCE_ACCOUNT_DISCRIMINANT.to_vec(),
        };
        let mut instructions = vec![nonce.clone()];
        assert!(ensure_compute_unit_price(&mut instructions, 5_000));
        assert_eq!(instructions[0], nonce);
        assert_eq!(instructions[1], set_compute_unit_price(5_000));
    }
}
