use gem_hash::sha2::sha512_half;
use primitives::SignerError;

use crate::address::XrpAddress;
use crate::currency::currency_code_bytes;
use crate::models::XrpPathStep;
use crate::signer::amount::XrpAmount;

const SIGNING_PREFIX: [u8; 4] = [0x53, 0x54, 0x58, 0x00];
const TRANSACTION_TYPE_PAYMENT: u16 = 0;
const TRANSACTION_TYPE_TRUST_SET: u16 = 20;
const TRANSACTION_TYPE_BATCH: u16 = 71;

const TF_PARTIAL_PAYMENT: u32 = 0x0002_0000;
const TF_ALL_OR_NOTHING: u32 = 0x0001_0000;
const TF_INNER_BATCH_TXN: u32 = 0x4000_0000;

const TYPE_UINT16: u8 = 1;
const TYPE_UINT32: u8 = 2;
const TYPE_AMOUNT: u8 = 6;
const TYPE_VL: u8 = 7;
const TYPE_ACCOUNT_ID: u8 = 8;
const TYPE_ST_OBJECT: u8 = 14;
const TYPE_ST_ARRAY: u8 = 15;
const TYPE_PATH_SET: u8 = 18;

const FIELD_TRANSACTION_TYPE: u8 = 2;
const FIELD_FLAGS: u8 = 2;
const FIELD_SEQUENCE: u8 = 4;
const FIELD_DESTINATION_TAG: u8 = 14;
const FIELD_LAST_LEDGER_SEQUENCE: u8 = 27;
const FIELD_AMOUNT: u8 = 1;
const FIELD_LIMIT_AMOUNT: u8 = 3;
const FIELD_FEE: u8 = 8;
const FIELD_SEND_MAX: u8 = 9;
const FIELD_DELIVER_MIN: u8 = 10;
const FIELD_SIGNING_PUB_KEY: u8 = 3;
const FIELD_TXN_SIGNATURE: u8 = 4;
const FIELD_MEMO_DATA: u8 = 13;
const FIELD_ACCOUNT: u8 = 1;
const FIELD_DESTINATION: u8 = 3;
const FIELD_MEMO: u8 = 10;
const FIELD_MEMOS: u8 = 9;
const FIELD_RAW_TRANSACTION: u8 = 34;
const FIELD_RAW_TRANSACTIONS: u8 = 30;
const FIELD_PATHS: u8 = 1;

const OBJECT_END_MARKER: u8 = 0xe1;
const ARRAY_END_MARKER: u8 = 0xf1;
const PATH_SEPARATOR: u8 = 0xff;
const PATH_SET_END: u8 = 0x00;
const PATH_STEP_ACCOUNT: u8 = 0x01;
const PATH_STEP_CURRENCY: u8 = 0x10;
const PATH_STEP_ISSUER: u8 = 0x20;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct XrpTransaction {
    flags: u32,
    sequence: u32,
    last_ledger_sequence: Option<u32>,
    fee: u64,
    signing_pub_key: Vec<u8>,
    account: XrpAddress,
    operation: XrpOperation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum XrpOperation {
    Payment {
        amount: XrpAmount,
        destination: XrpAddress,
        memo: XrpPaymentMemo,
        exchange: Option<Box<XrpExchange>>,
    },
    TrustSet {
        limit_amount: XrpAmount,
    },
    Batch {
        transactions: Vec<XrpTransaction>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct XrpExchange {
    pub(crate) send_max: XrpAmount,
    pub(crate) deliver_min: XrpAmount,
    pub(crate) paths: Vec<Vec<XrpPathStep>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum XrpPaymentMemo {
    None,
    DestinationTag(u32),
    Memo(Vec<u8>),
}

impl XrpTransaction {
    pub(crate) fn new_payment(params: XrpTransactionParams, amount: XrpAmount, destination: &str, memo: XrpPaymentMemo) -> Result<Self, SignerError> {
        Ok(Self::new(params, XrpOperation::new_payment(amount, destination, memo)?))
    }

    pub(crate) fn new_trust_set(params: XrpTransactionParams, limit_amount: XrpAmount) -> Self {
        Self::new(params, XrpOperation::TrustSet { limit_amount })
    }

    pub(crate) fn new_batch(params: XrpTransactionParams, operations: Vec<XrpOperation>) -> Result<Self, SignerError> {
        let account = params.account;
        let transactions = operations
            .into_iter()
            .zip(1u32..)
            .map(|(operation, offset)| {
                Ok(Self {
                    flags: TF_INNER_BATCH_TXN | operation.flags(),
                    sequence: params.sequence.checked_add(offset).ok_or_else(|| SignerError::invalid_input("XRP sequence overflow"))?,
                    last_ledger_sequence: None,
                    fee: 0,
                    signing_pub_key: vec![],
                    account,
                    operation,
                })
            })
            .collect::<Result<Vec<_>, SignerError>>()?;
        Ok(Self::new(params, XrpOperation::Batch { transactions }))
    }

    pub(crate) fn new(params: XrpTransactionParams, operation: XrpOperation) -> Self {
        Self {
            flags: operation.flags(),
            sequence: params.sequence,
            last_ledger_sequence: Some(params.last_ledger_sequence),
            fee: params.fee,
            signing_pub_key: params.signing_pub_key,
            account: params.account,
            operation,
        }
    }

    pub(crate) fn sign(&self, private_key: &[u8]) -> Result<String, SignerError> {
        let unsigned = self.encode(None)?;
        let mut preimage = Vec::with_capacity(SIGNING_PREFIX.len() + unsigned.len());
        preimage.extend_from_slice(&SIGNING_PREFIX);
        preimage.extend_from_slice(&unsigned);
        let digest = sha512_half(&preimage);
        let mut signature = ::signer::Signer::sign_digest(::signer::SignatureScheme::Secp256k1, &digest, private_key)?;
        if signature.len() < 64 {
            return Err(SignerError::signing_error("secp256k1 signature too short"));
        }
        signature.truncate(64);
        let der_signature = der_encode_signature(&signature)?;
        Ok(hex::encode(self.encode(Some(&der_signature))?))
    }

    fn encode(&self, signature: Option<&[u8]>) -> Result<Vec<u8>, SignerError> {
        let mut buffer = Vec::new();

        append_u16(&mut buffer, FIELD_TRANSACTION_TYPE, self.operation.transaction_type());
        append_u32(&mut buffer, FIELD_FLAGS, self.flags);
        append_u32(&mut buffer, FIELD_SEQUENCE, self.sequence);

        if let XrpOperation::Payment {
            memo: XrpPaymentMemo::DestinationTag(destination_tag),
            ..
        } = &self.operation
        {
            append_u32(&mut buffer, FIELD_DESTINATION_TAG, *destination_tag);
        }

        if let Some(last_ledger_sequence) = self.last_ledger_sequence {
            append_u32(&mut buffer, FIELD_LAST_LEDGER_SEQUENCE, last_ledger_sequence);
        }

        match &self.operation {
            XrpOperation::Payment { amount, .. } => append_amount(&mut buffer, FIELD_AMOUNT, amount)?,
            XrpOperation::TrustSet { limit_amount } => append_amount(&mut buffer, FIELD_LIMIT_AMOUNT, limit_amount)?,
            XrpOperation::Batch { .. } => {}
        }

        append_amount(&mut buffer, FIELD_FEE, &XrpAmount::Native(self.fee))?;

        if let XrpOperation::Payment { exchange: Some(exchange), .. } = &self.operation {
            append_amount(&mut buffer, FIELD_SEND_MAX, &exchange.send_max)?;
            append_amount(&mut buffer, FIELD_DELIVER_MIN, &exchange.deliver_min)?;
        }

        append_blob(&mut buffer, FIELD_SIGNING_PUB_KEY, &self.signing_pub_key)?;

        if let Some(signature) = signature {
            append_blob(&mut buffer, FIELD_TXN_SIGNATURE, signature)?;
        }

        append_address(&mut buffer, FIELD_ACCOUNT, &self.account)?;

        match &self.operation {
            XrpOperation::Payment { destination, memo, exchange, .. } => {
                append_address(&mut buffer, FIELD_DESTINATION, destination)?;
                append_memo(&mut buffer, memo)?;
                if let Some(exchange) = exchange {
                    append_paths(&mut buffer, &exchange.paths)?;
                }
            }
            XrpOperation::TrustSet { .. } => {}
            XrpOperation::Batch { transactions } => append_raw_transactions(&mut buffer, transactions)?,
        }

        Ok(buffer)
    }
}

impl XrpOperation {
    pub(crate) fn new_payment(amount: XrpAmount, destination: &str, memo: XrpPaymentMemo) -> Result<Self, SignerError> {
        Ok(Self::Payment {
            amount,
            destination: XrpAddress::parse(destination)?,
            memo,
            exchange: None,
        })
    }

    pub(crate) fn new_exchange(account: &XrpAddress, amount: XrpAmount, exchange: XrpExchange) -> Self {
        Self::Payment {
            amount,
            destination: *account,
            memo: XrpPaymentMemo::None,
            exchange: Some(Box::new(exchange)),
        }
    }

    fn transaction_type(&self) -> u16 {
        match self {
            Self::Payment { .. } => TRANSACTION_TYPE_PAYMENT,
            Self::TrustSet { .. } => TRANSACTION_TYPE_TRUST_SET,
            Self::Batch { .. } => TRANSACTION_TYPE_BATCH,
        }
    }

    fn flags(&self) -> u32 {
        match self {
            Self::Payment { exchange: Some(_), .. } => TF_PARTIAL_PAYMENT,
            Self::Batch { .. } => TF_ALL_OR_NOTHING,
            Self::Payment { .. } | Self::TrustSet { .. } => 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct XrpTransactionParams {
    pub(crate) account: XrpAddress,
    pub(crate) fee: u64,
    pub(crate) sequence: u32,
    pub(crate) last_ledger_sequence: u32,
    pub(crate) signing_pub_key: Vec<u8>,
}

fn append_u16(buffer: &mut Vec<u8>, field: u8, value: u16) {
    append_header(buffer, TYPE_UINT16, field);
    buffer.extend_from_slice(&value.to_be_bytes());
}

fn append_u32(buffer: &mut Vec<u8>, field: u8, value: u32) {
    append_header(buffer, TYPE_UINT32, field);
    buffer.extend_from_slice(&value.to_be_bytes());
}

fn append_amount(buffer: &mut Vec<u8>, field: u8, amount: &XrpAmount) -> Result<(), SignerError> {
    append_header(buffer, TYPE_AMOUNT, field);
    amount.encode(buffer)
}

fn append_blob(buffer: &mut Vec<u8>, field: u8, value: &[u8]) -> Result<(), SignerError> {
    append_header(buffer, TYPE_VL, field);
    append_variable_length(buffer, value.len())?;
    buffer.extend_from_slice(value);
    Ok(())
}

fn append_address(buffer: &mut Vec<u8>, field: u8, value: &XrpAddress) -> Result<(), SignerError> {
    append_header(buffer, TYPE_ACCOUNT_ID, field);
    append_variable_length(buffer, value.as_bytes().len())?;
    buffer.extend_from_slice(value.as_bytes());
    Ok(())
}

fn append_memo(buffer: &mut Vec<u8>, memo: &XrpPaymentMemo) -> Result<(), SignerError> {
    match memo {
        XrpPaymentMemo::Memo(memo) => {
            append_header(buffer, TYPE_ST_ARRAY, FIELD_MEMOS);
            append_header(buffer, TYPE_ST_OBJECT, FIELD_MEMO);
            append_blob(buffer, FIELD_MEMO_DATA, memo)?;
            buffer.push(OBJECT_END_MARKER);
            buffer.push(ARRAY_END_MARKER);
            Ok(())
        }
        XrpPaymentMemo::None | XrpPaymentMemo::DestinationTag(_) => Ok(()),
    }
}

fn append_raw_transactions(buffer: &mut Vec<u8>, transactions: &[XrpTransaction]) -> Result<(), SignerError> {
    append_header(buffer, TYPE_ST_ARRAY, FIELD_RAW_TRANSACTIONS);
    for transaction in transactions {
        append_header(buffer, TYPE_ST_OBJECT, FIELD_RAW_TRANSACTION);
        buffer.extend_from_slice(&transaction.encode(None)?);
        buffer.push(OBJECT_END_MARKER);
    }
    buffer.push(ARRAY_END_MARKER);
    Ok(())
}

fn append_paths(buffer: &mut Vec<u8>, paths: &[Vec<XrpPathStep>]) -> Result<(), SignerError> {
    if paths.is_empty() {
        return Ok(());
    }
    append_header(buffer, TYPE_PATH_SET, FIELD_PATHS);
    for (index, path) in paths.iter().enumerate() {
        if index > 0 {
            buffer.push(PATH_SEPARATOR);
        }
        for step in path {
            append_path_step(buffer, step)?;
        }
    }
    buffer.push(PATH_SET_END);
    Ok(())
}

fn append_path_step(buffer: &mut Vec<u8>, step: &XrpPathStep) -> Result<(), SignerError> {
    let account = step.account.as_deref().map(XrpAddress::parse).transpose()?;
    let currency = step.currency.as_deref().map(currency_code_bytes).transpose()?;
    let issuer = step.issuer.as_deref().map(XrpAddress::parse).transpose()?;
    let fields = [
        (PATH_STEP_ACCOUNT, account.as_ref().map(XrpAddress::as_bytes)),
        (PATH_STEP_CURRENCY, currency.as_ref()),
        (PATH_STEP_ISSUER, issuer.as_ref().map(XrpAddress::as_bytes)),
    ];
    buffer.push(fields.iter().filter(|(_, bytes)| bytes.is_some()).fold(0, |kind, (flag, _)| kind | flag));
    fields.iter().filter_map(|(_, bytes)| *bytes).for_each(|bytes| buffer.extend_from_slice(bytes));
    Ok(())
}

fn append_header(buffer: &mut Vec<u8>, type_code: u8, field_code: u8) {
    match (type_code < 16, field_code < 16) {
        (true, true) => buffer.push((type_code << 4) | field_code),
        (true, false) => {
            buffer.push(type_code << 4);
            buffer.push(field_code);
        }
        (false, true) => {
            buffer.push(field_code);
            buffer.push(type_code);
        }
        (false, false) => {
            buffer.push(0);
            buffer.push(type_code);
            buffer.push(field_code);
        }
    }
}

fn append_variable_length(buffer: &mut Vec<u8>, length: usize) -> Result<(), SignerError> {
    if length <= 192 {
        buffer.push(length as u8);
        return Ok(());
    }

    if length < 12_481 {
        let length = length - 193;
        buffer.push(((length >> 8) + 193) as u8);
        buffer.push((length & 0xff) as u8);
        return Ok(());
    }

    if length <= 918_744 {
        let length = length - 12_481;
        buffer.push(((length >> 16) + 241) as u8);
        buffer.push(((length >> 8) & 0xff) as u8);
        buffer.push((length & 0xff) as u8);
        return Ok(());
    }

    Err(SignerError::invalid_input("XRP field is too large"))
}

fn der_encode_signature(signature: &[u8]) -> Result<Vec<u8>, SignerError> {
    if signature.len() != 64 {
        return Err(SignerError::signing_error("invalid secp256k1 signature length"));
    }

    let r = der_integer(&signature[..32]);
    let s = der_integer(&signature[32..]);
    let payload_len = r.len() + s.len();
    let mut der = Vec::with_capacity(payload_len + 2);
    der.push(0x30);
    der.push(payload_len.try_into().map_err(|_| SignerError::signing_error("signature is too large"))?);
    der.extend_from_slice(&r);
    der.extend_from_slice(&s);
    Ok(der)
}

fn der_integer(value: &[u8]) -> Vec<u8> {
    let mut bytes = value.iter().skip_while(|b| **b == 0).copied().collect::<Vec<_>>();
    if bytes.is_empty() {
        bytes.push(0);
    }
    if bytes[0] & 0x80 != 0 {
        bytes.insert(0, 0);
    }

    let mut encoded = Vec::with_capacity(bytes.len() + 2);
    encoded.push(0x02);
    encoded.push(bytes.len() as u8);
    encoded.extend_from_slice(&bytes);
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;

    const TF_SET_NO_RIPPLE: u32 = 0x0002_0000;

    const MAINNET_BATCH_WITHOUT_ITS_SECOND_PAYMENT: &str = "12004722000100002406424CAC68400000000000006473210284B1DE798765483FBD17373846D4A39378227368E1F76EC074E602FF1F5F41F5744630440220586F2D5B3F7A4471EB4B08C29623A6D290A50148FFF4C981AE2CD2CA1E4B43A60220368170E36F1985C86F3F17DC7B19F06DAABF03E2D31943E7D105A2D62FA9395D811439FBDD91DE7E37E8431918EFC96E590B936727EAF01EE02212001422400200002406424CAD63D8438D7EA4C680004252544F00000000000000000000000000000000CDAE366CD6AE13BC6C9B165AD6F20E2721414BFA6840000000000000007300811439FBDD91DE7E37E8431918EFC96E590B936727EAE1E02212000022400200002406424CAF6140038D7EA4C6800068400000000000000069D8438D7EA4C680000000000000000000000000004E5853000000000095C7AFC4CCA0A6D27DEE3B661A8F18FEDE2C11416A400000000001AD2E7300811439FBDD91DE7E37E8431918EFC96E590B936727EA831439FBDD91DE7E37E8431918EFC96E590B936727EAE1F1";

    #[test]
    fn test_encode_batch() {
        let account = XrpAddress::parse("raHbNigTZBS1iNSXLn7Tnbg6WBxenbwpuJ").unwrap();
        let trust_set = XrpTransaction {
            flags: TF_INNER_BATCH_TXN | TF_SET_NO_RIPPLE,
            sequence: 105_008_301,
            last_ledger_sequence: None,
            fee: 0,
            signing_pub_key: vec![],
            account,
            operation: XrpOperation::TrustSet {
                limit_amount: XrpAmount::issued("1000000000000000", "4252544F00000000000000000000000000000000", "rKkYDqhye97uBNYqSDdKSKpa3YsScnd7M7").unwrap(),
            },
        };
        let sell = XrpTransaction {
            flags: TF_INNER_BATCH_TXN | TF_PARTIAL_PAYMENT,
            sequence: 105_008_303,
            operation: XrpOperation::new_exchange(
                &account,
                XrpAmount::Native(1_000_000_000_000_000),
                XrpExchange {
                    send_max: XrpAmount::issued("1000000000000000", "NXS", "rNexusA23ZQdtejTCeHZZaiKoJRsrnXboq").unwrap(),
                    deliver_min: XrpAmount::Native(109_870),
                    paths: vec![],
                },
            ),
            ..trust_set.clone()
        };
        let batch = XrpTransaction {
            flags: TF_ALL_OR_NOTHING,
            sequence: 105_008_300,
            last_ledger_sequence: None,
            fee: 100,
            signing_pub_key: hex::decode("0284B1DE798765483FBD17373846D4A39378227368E1F76EC074E602FF1F5F41F5").unwrap(),
            account,
            operation: XrpOperation::Batch { transactions: vec![trust_set, sell] },
        };
        let signature = hex::decode("30440220586F2D5B3F7A4471EB4B08C29623A6D290A50148FFF4C981AE2CD2CA1E4B43A60220368170E36F1985C86F3F17DC7B19F06DAABF03E2D31943E7D105A2D62FA9395D").unwrap();

        assert_eq!(hex::encode_upper(batch.encode(Some(&signature)).unwrap()), MAINNET_BATCH_WITHOUT_ITS_SECOND_PAYMENT);
    }

    #[test]
    fn test_variable_length_encoding() {
        let mut bytes = Vec::new();
        append_variable_length(&mut bytes, 180).unwrap();
        assert_eq!(hex::encode(&bytes), "b4");

        let mut bytes = Vec::new();
        append_variable_length(&mut bytes, 1000).unwrap();
        assert_eq!(hex::encode(&bytes), "c427");

        let mut bytes = Vec::new();
        append_variable_length(&mut bytes, 12_580).unwrap();
        assert_eq!(hex::encode(&bytes), "f10063");
    }
}
