#[cfg(feature = "signer")]
use num_bigint::BigUint;
use primitives::Address as _;

use crate::address::TronAddress;

const ABI_WORD_LEN: usize = 32;
const TRC20_APPROVE_SELECTOR: [u8; 4] = [0x09, 0x5e, 0xa7, 0xb3];
#[cfg(feature = "signer")]
const TRC20_TRANSFER_SELECTOR: [u8; 4] = [0xa9, 0x05, 0x9c, 0xbb];

#[cfg(feature = "signer")]
pub(crate) fn encode_transfer(destination: &TronAddress, value: &str) -> Result<Vec<u8>, &'static str> {
    let amount = value.parse::<BigUint>().map_err(|_| "invalid TRC20 amount")?;
    let mut data = TRC20_TRANSFER_SELECTOR.to_vec();
    data.extend(pad_left(destination.as_bytes(), ABI_WORD_LEN)?);
    data.extend(pad_left(&amount.to_bytes_be(), ABI_WORD_LEN)?);
    Ok(data)
}

pub(crate) fn encode_approval_max(spender: &TronAddress) -> Result<Vec<u8>, &'static str> {
    let mut data = TRC20_APPROVE_SELECTOR.to_vec();
    data.extend(pad_left(spender.as_bytes(), ABI_WORD_LEN)?);
    data.extend([0xff; ABI_WORD_LEN]);
    Ok(data)
}

fn pad_left(data: &[u8], len: usize) -> Result<Vec<u8>, &'static str> {
    if data.len() > len {
        return Err("value does not fit padded length");
    }
    let mut padded = vec![0u8; len - data.len()];
    padded.extend_from_slice(data);
    Ok(padded)
}
