use crate::address::StellarAddress;
use crate::models::signing::{Memo, Operation, StellarAssetCode, StellarAssetData, StellarTransaction};
use primitives::Address;
use signer::ED25519_KEY_TYPE;

const ASSET_TYPE_NATIVE: u32 = 0;
const ASSET_TYPE_ALPHANUM4: u32 = 1;
const ASSET_TYPE_ALPHANUM12: u32 = 2;

/// XDR-encode a Stellar transaction (unsigned envelope body).
pub(crate) fn encode_transaction(tx: &StellarTransaction) -> Vec<u8> {
    let mut data = Vec::new();
    encode_address(&mut data, &tx.account);
    write_u32(&mut data, tx.fee);
    write_u64(&mut data, tx.sequence);
    encode_time_bounds(&mut data, tx);
    encode_memo(&mut data, &tx.memo);
    write_u32(&mut data, tx.operations.len() as u32);
    for operation in &tx.operations {
        encode_operation(&mut data, operation);
    }

    // ext (union void)
    write_u32(&mut data, 0);
    data
}

fn encode_operation(data: &mut Vec<u8>, operation: &Operation) {
    write_u32(data, 0);
    write_u32(data, operation.operation_type());

    match operation {
        Operation::CreateAccount { destination, amount } => encode_create_account(data, destination, *amount),
        Operation::Payment { destination, asset, amount } => encode_payment(data, destination, asset.as_ref(), *amount),
        Operation::ChangeTrust { asset } => encode_change_trust(data, asset),
        Operation::PathPaymentStrictSend {
            send_asset,
            send_amount,
            destination,
            destination_asset,
            destination_min,
            path,
        } => encode_path_payment_strict_send(data, send_asset.as_ref(), *send_amount, destination, destination_asset.as_ref(), *destination_min, path),
    }
}

fn encode_create_account(data: &mut Vec<u8>, destination: &StellarAddress, amount: u64) {
    encode_address(data, destination);
    write_u64(data, amount);
}

fn encode_payment(data: &mut Vec<u8>, destination: &StellarAddress, asset: Option<&StellarAssetData>, amount: u64) {
    encode_address(data, destination);
    encode_asset(data, asset);
    write_u64(data, amount);
}

fn encode_change_trust(data: &mut Vec<u8>, asset: &StellarAssetData) {
    encode_asset(data, Some(asset));
    write_u64(data, i64::MAX as u64);
}

fn encode_path_payment_strict_send(
    data: &mut Vec<u8>,
    send_asset: Option<&StellarAssetData>,
    send_amount: u64,
    destination: &StellarAddress,
    destination_asset: Option<&StellarAssetData>,
    destination_min: u64,
    path: &[Option<StellarAssetData>],
) {
    encode_asset(data, send_asset);
    write_u64(data, send_amount);
    encode_address(data, destination);
    encode_asset(data, destination_asset);
    write_u64(data, destination_min);
    write_u32(data, path.len() as u32);
    path.iter().for_each(|asset| encode_asset(data, asset.as_ref()));
}

fn encode_time_bounds(data: &mut Vec<u8>, tx: &StellarTransaction) {
    if let Some(to) = tx.time_bounds.filter(|v| *v > 0) {
        write_u32(data, 1);
        write_u64(data, 0);
        write_u64(data, to);
    } else {
        write_u32(data, 0);
    }
}

fn encode_memo(data: &mut Vec<u8>, memo: &Memo) {
    match memo {
        Memo::None => write_u32(data, 0),
        Memo::Text(text) => {
            write_u32(data, 1);
            write_u32(data, text.len() as u32);
            data.extend_from_slice(text.as_bytes());
            pad4(data);
        }
        Memo::Id(id) => {
            write_u32(data, 2);
            write_u64(data, *id);
        }
    }
}

fn encode_asset(data: &mut Vec<u8>, asset: Option<&StellarAssetData>) {
    match asset {
        Some(asset) => {
            match &asset.code {
                StellarAssetCode::Alphanum4(code) => {
                    write_u32(data, ASSET_TYPE_ALPHANUM4);
                    data.extend_from_slice(code);
                }
                StellarAssetCode::Alphanum12(code) => {
                    write_u32(data, ASSET_TYPE_ALPHANUM12);
                    data.extend_from_slice(code);
                }
            }
            encode_address(data, &asset.issuer);
        }
        None => write_u32(data, ASSET_TYPE_NATIVE),
    }
}

fn encode_address(data: &mut Vec<u8>, address: &StellarAddress) {
    write_u32(data, ED25519_KEY_TYPE as u32);
    data.extend_from_slice(address.as_bytes());
}

fn write_u32(data: &mut Vec<u8>, value: u32) {
    data.extend_from_slice(&value.to_be_bytes());
}

fn write_u64(data: &mut Vec<u8>, value: u64) {
    data.extend_from_slice(&value.to_be_bytes());
}

fn pad4(data: &mut Vec<u8>) {
    let padding = (4 - (data.len() % 4)) % 4;
    data.extend(std::iter::repeat_n(0, padding));
}

#[cfg(test)]
mod tests {
    use super::*;
    use gem_encoding::decode_base64;

    fn address(value: &str) -> StellarAddress {
        StellarAddress::from_str(value).unwrap()
    }

    fn asset(code: &str, issuer: &str) -> Option<StellarAssetData> {
        Some(StellarAssetData::new(issuer, code).unwrap())
    }

    #[test]
    fn test_encode_path_payment_strict_send_matches_stellar_sdk() {
        let operation = Operation::PathPaymentStrictSend {
            send_asset: None,
            send_amount: 1_000_000_000,
            destination: address("GC2GT6BHYJUKD7SVAKXVLBYBCELCHY577CAXJM5QNVLERDGFF37LR35K"),
            destination_asset: asset("USD", "GCNY5OXYSY4FKHOPT2SPOQZAOEIGXB5LBYW3HVU3OWSTQITS65M5RCNY"),
            destination_min: 2_000_000_000,
            path: vec![
                asset("CATCOIN", "GDJVFDG5OCW5PYWHB64MGTHGFF57DRRJEDUEFDEL2SLNIOONHYJWHA3Z"),
                asset("PANDA", "GDJVFDG5OCW5PYWHB64MGTHGFF57DRRJEDUEFDEL2SLNIOONHYJWHA3Z"),
            ],
        };
        let mut encoded = Vec::new();
        encode_operation(&mut encoded, &operation);

        assert_eq!(
            encoded,
            decode_base64("AAAAAAAAAA0AAAAAAAAAADuaygAAAAAAtGn4J8Joof5VAq9VhwERFiPjv/iBdLOwbVZIjMUu/rgAAAABVVNEAAAAAACbjrr4ljhVHc+epPdDIHEQa4erDi2z1pt1pTgicvdZ2AAAAAB3NZQAAAAAAgAAAAJDQVRDT0lOAAAAAAAAAAAA01KM3XCt1+LHD7jDTOYpe/HGKSDoQoyL1JbUOc0+E2MAAAACUEFOREEAAAAAAAAAAAAAANNSjN1wrdfixw+4w0zmKXvxxikg6EKMi9SW1DnNPhNj").unwrap(),
            "py-stellar-base tests/operation/test_path_payment_strict_send.py, case without_source"
        );
    }

    #[test]
    fn test_encode_swap_to_self_matches_a_mainnet_transaction() {
        let sender = address("GCVQA5JVHNDJFPODJZU6BM7H5344VK2KGBFXSUUEP7ZH3T5DXGMCR6NL");
        let transaction = StellarTransaction {
            account: sender.clone(),
            fee: 100,
            sequence: 278_896_359_950_521_622,
            memo: Memo::None,
            time_bounds: Some(1_792_902_087),
            operations: vec![Operation::PathPaymentStrictSend {
                send_asset: None,
                send_amount: 20_034_027,
                destination: sender,
                destination_asset: asset("USDC", "GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN"),
                destination_min: 3_262_747,
                path: vec![],
            }],
        };
        let envelope = decode_base64("AAAAAgAAAACrAHU1O0aSvcNOaeCz5+75yqtKMEt5UoR/8n3Po7mYKAAAAGQD3tbDAAAVFgAAAAEAAAAAAAAAAAAAAABq3YPHAAAAAAAAAAEAAAAAAAAADQAAAAAAAAAAATGx6wAAAACrAHU1O0aSvcNOaeCz5+75yqtKMEt5UoR/8n3Po7mYKAAAAAFVU0RDAAAAADuZETgO/piLoKiQDrHP5E82b32+lGvtB3JA9/Yk3xXFAAAAAAAxyRsAAAAAAAAAAAAAAAGjuZgoAAAAQC86O1b6KMIGFYR0qQSpz6/K1X2ilQYn8DFN4VHfjc+mkKPdYmXG41sWH+gb00qfEgwIpIKiIK0EFRSWqXj/nw4=").unwrap();
        let signatures_length = 4 + 4 + 4 + 64;

        assert_eq!(&envelope[..4], &2u32.to_be_bytes(), "a v1 envelope");
        assert_eq!(
            encode_transaction(&transaction),
            envelope[4..envelope.len() - signatures_length],
            "mainnet b652bc89d2c3b29573a512d76088d8fc3108298947451197af019709185f2ef2: 2.0034027 XLM to at least 0.3262747 USDC, paid to its sender"
        );
    }
}
