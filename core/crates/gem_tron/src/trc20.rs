use num_bigint::BigUint;
use primitives::Address as _;

use crate::address::TronAddress;

const ABI_WORD_LEN: usize = 32;
#[cfg(feature = "rpc")]
const GASFREE_PERMIT_TRANSFER_SELECTOR: [u8; 4] = [0x6f, 0x21, 0xb8, 0x98];
const TRC20_APPROVE_SELECTOR: [u8; 4] = [0x09, 0x5e, 0xa7, 0xb3];
#[cfg(feature = "signer")]
const TRC20_TRANSFER_SELECTOR: [u8; 4] = [0xa9, 0x05, 0x9c, 0xbb];

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ApprovalCall {
    pub spender: TronAddress,
    pub value: BigUint,
}

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

pub(crate) fn decode_approval_hex(data: &str) -> Option<ApprovalCall> {
    decode_approval(&hex::decode(data).ok()?)
}

pub(crate) fn decode_approval(data: &[u8]) -> Option<ApprovalCall> {
    if data.len() != TRC20_APPROVE_SELECTOR.len() + ABI_WORD_LEN * 2 || !data.starts_with(&TRC20_APPROVE_SELECTOR) {
        return None;
    }

    let spender = TronAddress::from_abi_word(data.get(TRC20_APPROVE_SELECTOR.len()..TRC20_APPROVE_SELECTOR.len() + ABI_WORD_LEN)?)?;
    let value = BigUint::from_bytes_be(data.get(TRC20_APPROVE_SELECTOR.len() + ABI_WORD_LEN..)?);

    Some(ApprovalCall { spender, value })
}

#[cfg(feature = "rpc")]
pub(crate) fn decode_gasfree_permit_transfer_hex(data: &str) -> Option<(TronAddress, TronAddress, BigUint)> {
    let data = hex::decode(data).ok()?;
    let arguments = data.strip_prefix(&GASFREE_PERMIT_TRANSFER_SELECTOR)?;
    let mut words = arguments.chunks_exact(ABI_WORD_LEN);
    let token = TronAddress::from_abi_word(words.next()?)?;
    words.next()?;
    let receiver = TronAddress::from_abi_word(words.next()?)?;
    let value = BigUint::from_bytes_be(words.next()?);
    words.nth(4)?;

    Some((token, receiver, value))
}

fn pad_left(data: &[u8], len: usize) -> Result<Vec<u8>, &'static str> {
    if data.len() > len {
        return Err("value does not fit padded length");
    }
    let mut padded = vec![0u8; len - data.len()];
    padded.extend_from_slice(data);
    Ok(padded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_approval() {
        let approval =
            decode_approval_hex("095ea7b3000000000000000000000000019e353a35efaa8e27c2a602a791ae1b19d9c9fa0000000000000000000000000000000000000000000000000000000000000000")
                .unwrap();

        assert_eq!(approval.spender.encode(), "TA7mCjHFfo68FG3wc6pDCeRGbJSPZkBfL7");
        assert_eq!(approval.value, BigUint::from(0u32));

        let prefixed_approval =
            decode_approval_hex("095ea7b3000000000000000000000041c148af9b50bc03cc0c616cd85c66aae9bd90cd80ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff")
                .unwrap();

        assert_eq!(prefixed_approval.spender.encode(), "TTbCVPfUZmPhrB9sYC8GKgGBQQEdZovkmS");
        assert_eq!(prefixed_approval.value, BigUint::from_bytes_be(&[0xff; ABI_WORD_LEN]));

        assert_eq!(
            decode_approval_hex("87517c45000000000000000000000000a614f803b6fd780986a42c78ec9c7f77e6ded13c000000000000000000000000a31d689a84244bc01be56e07aeafb7686f56bb89"),
            None
        );
        assert_eq!(decode_approval_hex("095ea7b3"), None);
        assert_eq!(
            decode_approval_hex("095ea7b3000000000000000000010000019e353a35efaa8e27c2a602a791ae1b19d9c9fa0000000000000000000000000000000000000000000000000000000000000000",),
            None
        );
    }
}
