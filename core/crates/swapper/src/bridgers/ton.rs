use std::str::FromStr;

use gem_ton::{
    address::Address,
    constants::JETTON_TRANSFER_OPCODE,
    tvm::{BagOfCells, Cell, CellArc, CellBuilder},
};
use num_bigint::BigUint;

use super::{
    model::{DepositMemo, TonTransaction},
    transaction::get_to_token_channel,
};
use crate::SwapperError;

const NATIVE_DEPOSIT_OPCODE: u32 = 0x7362d09c;
const NATIVE_MEMO_OPCODE: u32 = 0x3d0518f8;
const JETTON_MEMO_OPCODE: u32 = 0x6a5229ae;
const FORWARD_TON_AMOUNT: u64 = 10_000_000;
pub(super) const JETTON_TRANSFER_TON_AMOUNT: u64 = 60_000_000;

fn get_memo_cell(opcode: u32, sender: &Address, memo: &DepositMemo) -> Result<CellArc, SwapperError> {
    let text = serde_json::to_string(memo).map_err(SwapperError::compute_quote_error)?;
    let mut text_builder = CellBuilder::new();
    text_builder.store_u32(32, 0)?.store_string_snake(&text)?;
    let mut builder = CellBuilder::new();
    builder
        .store_u32(32, opcode)?
        .store_address(sender)?
        .store_coins(&BigUint::from(FORWARD_TON_AMOUNT))?
        .store_reference(&text_builder.end_cell()?.into_arc())?;
    Ok(builder.end_cell()?.into_arc())
}

pub(super) fn get_native_payload(sender: &Address, memo: &DepositMemo) -> Result<Cell, SwapperError> {
    let mut builder = CellBuilder::new();
    builder.store_u32(32, NATIVE_DEPOSIT_OPCODE)?.store_reference(&get_memo_cell(NATIVE_MEMO_OPCODE, sender, memo)?)?;
    Ok(builder.end_cell()?)
}

pub(super) fn get_jetton_payload(amount: &BigUint, router: &Address, sender: &Address, memo: &DepositMemo) -> Result<Cell, SwapperError> {
    let mut builder = CellBuilder::new();
    builder
        .store_u32(32, JETTON_TRANSFER_OPCODE)?
        .store_u64(64, 0)?
        .store_coins(amount)?
        .store_address(router)?
        .store_maybe_address(Some(sender))?
        .store_maybe_reference(None)?
        .store_coins(&BigUint::from(FORWARD_TON_AMOUNT))?
        .store_maybe_reference(Some(&get_memo_cell(JETTON_MEMO_OPCODE, sender, memo)?))?;
    Ok(builder.end_cell()?)
}

pub(super) fn get_payload_channel(transaction: &TonTransaction, to_code: &str, slippage: &str) -> Result<String, SwapperError> {
    let payload = BagOfCells::parse_base64_root(&transaction.payload).map_err(|_| SwapperError::InvalidRoute)?;
    let text = payload.references.first().and_then(|memo| memo.references.first()).ok_or(SwapperError::InvalidRoute)?;
    let bytes = snake_bytes(text).ok_or(SwapperError::InvalidRoute)?;
    let memo: DepositMemo = serde_json::from_slice(bytes.get(4..).ok_or(SwapperError::InvalidRoute)?).map_err(|_| SwapperError::InvalidRoute)?;
    get_to_token_channel(&memo.to_token, to_code, slippage).map(str::to_string)
}

fn snake_bytes(cell: &Cell) -> Option<Vec<u8>> {
    let mut bytes = Vec::new();
    let mut cell = cell;
    loop {
        if !cell.bit_len.is_multiple_of(8) {
            return None;
        }
        bytes.extend_from_slice(&cell.data);
        let Some(next) = cell.references.first() else {
            return Some(bytes);
        };
        cell = next;
    }
}

pub(super) fn get_transaction_payload(transaction: &TonTransaction, to: &Address, amount: &BigUint, payload: Cell) -> Result<String, SwapperError> {
    let address = Address::parse(&transaction.address).map_err(|_| SwapperError::InvalidRoute)?;
    let value = BigUint::from_str(&transaction.amount).map_err(|_| SwapperError::InvalidRoute)?;
    let received = BagOfCells::parse_base64_root(&transaction.payload).map_err(|_| SwapperError::InvalidRoute)?;
    if &address != to || &value != amount || received.hash != payload.hash {
        return Err(SwapperError::InvalidRoute);
    }
    Ok(BagOfCells::from_root(payload).to_base64(true)?)
}

#[cfg(test)]
mod tests {
    use primitives::{asset_constants::TON_USDT_TOKEN_ID, contract_constants::EVM_NATIVE_TOKEN_ADDRESS};

    use super::*;
    use crate::bridgers::{
        model::{BridgersResponse, SwapData, SwapTransaction},
        testkit::{TEST_EVM_WALLET, TEST_TON_WALLET},
    };

    #[test]
    fn test_get_transaction_payload() {
        let native: BridgersResponse = serde_json::from_str(include_str!("testdata/swap_ton_bsc_usdt.json")).unwrap();
        let jetton: BridgersResponse = serde_json::from_str(include_str!("testdata/swap_ton_usdt_bsc_usdt.json")).unwrap();
        let SwapTransaction::Ton(native_tx) = serde_json::from_value::<SwapData>(native.data).unwrap().tx_data else {
            panic!()
        };
        let SwapTransaction::Ton(jetton_tx) = serde_json::from_value::<SwapData>(jetton.data).unwrap().tx_data else {
            panic!()
        };
        let sender = Address::parse(TEST_TON_WALLET).unwrap();
        let router = Address::parse("EQCqTuPS-cMN9SSRhaFrwJOGL7dKkFF8C0a05g2WzI9uRJ19").unwrap();
        let jetton_wallet = Address::parse("EQCcPR6bD3c161LJe1hVCFni-z9y7fCkFtpSYwrKncccG4nG").unwrap();
        let native_memo = DepositMemo {
            from_token: EVM_NATIVE_TOKEN_ADDRESS.to_lowercase(),
            to_token: "USDT(BSC)|ht6zut|0.005|bridgers|0".to_string(),
            sender: TEST_TON_WALLET.to_string(),
            destination: TEST_EVM_WALLET.to_string(),
            min_return_amount: "33449512000000000000".to_string(),
        };
        let jetton_memo = DepositMemo {
            from_token: TON_USDT_TOKEN_ID.to_string(),
            min_return_amount: "49590000000000000000".to_string(),
            ..native_memo.clone()
        };
        let native_payload = get_native_payload(&sender, &native_memo).unwrap();
        let jetton_payload = get_jetton_payload(&BigUint::from(50_000_000u64), &router, &sender, &jetton_memo).unwrap();
        let other_destination = get_native_payload(
            &sender,
            &DepositMemo {
                destination: "0x0000000000000000000000000000000000000001".to_string(),
                ..native_memo
            },
        )
        .unwrap();

        assert_eq!(
            get_transaction_payload(&native_tx, &router, &BigUint::from(20_000_000_000u64), native_payload.clone()).unwrap(),
            BagOfCells::from_root(native_payload).to_base64(true).unwrap()
        );
        assert_eq!(
            get_transaction_payload(&jetton_tx, &jetton_wallet, &BigUint::from(JETTON_TRANSFER_TON_AMOUNT), jetton_payload.clone()).unwrap(),
            BagOfCells::from_root(jetton_payload).to_base64(true).unwrap()
        );
        assert_eq!(get_transaction_payload(&native_tx, &router, &BigUint::from(20_000_000_000u64), other_destination).unwrap_err(), SwapperError::InvalidRoute);
    }

    #[test]
    fn test_get_payload_channel() {
        let native: BridgersResponse = serde_json::from_str(include_str!("testdata/swap_ton_bsc_usdt.json")).unwrap();
        let jetton: BridgersResponse = serde_json::from_str(include_str!("testdata/swap_ton_usdt_bsc_usdt.json")).unwrap();
        let SwapTransaction::Ton(native_tx) = serde_json::from_value::<SwapData>(native.data).unwrap().tx_data else {
            panic!()
        };
        let SwapTransaction::Ton(jetton_tx) = serde_json::from_value::<SwapData>(jetton.data).unwrap().tx_data else {
            panic!()
        };

        assert_eq!(get_payload_channel(&native_tx, "USDT(BSC)", "0.005").unwrap(), "ht6zut");
        assert_eq!(get_payload_channel(&jetton_tx, "USDT(BSC)", "0.005").unwrap(), "ht6zut");
        assert_eq!(get_payload_channel(&native_tx, "BNB(BSC)", "0.005").unwrap_err(), SwapperError::InvalidRoute);
    }
}
