use bitcoin::{TxIn, TxOut, VarInt, consensus::Decodable};
use primitives::decode_hex;

const VERSION_MASK: u32 = 0x7fff_ffff;
const VERSION_GROUP_LEN: usize = 4;
const V5_HEADER_TAIL_LEN: usize = 16;
const LOCK_TIME_LEN: usize = 4;
const LOCK_TIME_AND_EXPIRY_LEN: usize = 8;
const BCTV14_JOINSPLIT_LEN: usize = 1802;
const V4_SAPLING_SPEND_LEN: usize = 384;
const V4_SAPLING_OUTPUT_LEN: usize = 948;
const JOINSPLIT_VALUES_LEN: usize = 16;
const GROTH_JOINSPLIT_LEN: usize = 1698;
const JOINSPLIT_PUBKEY_LEN: usize = 32;
const SAPLING_SPEND_LEN: usize = 96;
const SAPLING_OUTPUT_LEN: usize = 756;
const SAPLING_SPEND_PROOF_AND_SIGNATURE_LEN: usize = 192 + 64;
const SAPLING_OUTPUT_PROOF_LEN: usize = 192;
const ACTION_LEN: usize = 820;
const FLAGS_LEN: usize = 1;
const ANCHOR_LEN: usize = 32;
const SIGNATURE_LEN: usize = 64;

pub struct ValueBalanceReader<'a> {
    bytes: &'a [u8],
}

impl ValueBalanceReader<'_> {
    pub fn read(hex: &str) -> Option<i64> {
        let bytes = decode_hex(hex).ok()?;
        ValueBalanceReader { bytes: &bytes }.value_balance()
    }

    fn value_balance(mut self) -> Option<i64> {
        let balance = match self.decode::<u32>()? & VERSION_MASK {
            1 => self.v1_value_balance()?,
            2 => self.v2_value_balance()?,
            3 => self.v3_value_balance()?,
            4 => self.v4_value_balance()?,
            5 => self.v5_value_balance()?,
            6 => self.v5_value_balance()?.checked_add(self.action_value_balance()?)?,
            _ => return None,
        };
        self.bytes.is_empty().then_some(balance)
    }

    fn v1_value_balance(&mut self) -> Option<i64> {
        self.skip_transparent()?;
        self.skip(LOCK_TIME_LEN)?;
        Some(0)
    }

    fn v2_value_balance(&mut self) -> Option<i64> {
        self.skip_transparent()?;
        self.skip(LOCK_TIME_LEN)?;
        self.sprout_value_balance(BCTV14_JOINSPLIT_LEN)
    }

    fn v3_value_balance(&mut self) -> Option<i64> {
        self.skip(VERSION_GROUP_LEN)?;
        self.skip_transparent()?;
        self.skip(LOCK_TIME_AND_EXPIRY_LEN)?;
        self.sprout_value_balance(BCTV14_JOINSPLIT_LEN)
    }

    fn v4_value_balance(&mut self) -> Option<i64> {
        self.skip(VERSION_GROUP_LEN)?;
        self.skip_transparent()?;
        self.skip(LOCK_TIME_AND_EXPIRY_LEN)?;
        let sapling = self.decode::<i64>()?;
        let spends = self.compact_size()?;
        self.skip_items(spends, V4_SAPLING_SPEND_LEN)?;
        let outputs = self.compact_size()?;
        self.skip_items(outputs, V4_SAPLING_OUTPUT_LEN)?;
        let sprout = self.sprout_value_balance(GROTH_JOINSPLIT_LEN)?;
        if spends > 0 || outputs > 0 {
            self.skip(SIGNATURE_LEN)?;
        }
        sapling.checked_add(sprout)
    }

    fn v5_value_balance(&mut self) -> Option<i64> {
        self.skip(V5_HEADER_TAIL_LEN)?;
        self.skip_transparent()?;
        self.sapling_value_balance()?.checked_add(self.action_value_balance()?)
    }

    fn sprout_value_balance(&mut self, joinsplit_len: usize) -> Option<i64> {
        let joinsplits = self.compact_size()?;
        let mut balance = 0i64;
        for _ in 0..joinsplits {
            let vpub_old = i64::try_from(self.decode::<u64>()?).ok()?;
            let vpub_new = i64::try_from(self.decode::<u64>()?).ok()?;
            self.skip(joinsplit_len - JOINSPLIT_VALUES_LEN)?;
            balance = balance.checked_add(vpub_new)?.checked_sub(vpub_old)?;
        }
        if joinsplits > 0 {
            self.skip(JOINSPLIT_PUBKEY_LEN + SIGNATURE_LEN)?;
        }
        Some(balance)
    }

    fn sapling_value_balance(&mut self) -> Option<i64> {
        let spends = self.compact_size()?;
        self.skip_items(spends, SAPLING_SPEND_LEN)?;
        let outputs = self.compact_size()?;
        self.skip_items(outputs, SAPLING_OUTPUT_LEN)?;
        if spends == 0 && outputs == 0 {
            return Some(0);
        }
        let balance = self.decode::<i64>()?;
        if spends > 0 {
            self.skip(ANCHOR_LEN)?;
        }
        self.skip_items(spends, SAPLING_SPEND_PROOF_AND_SIGNATURE_LEN)?;
        self.skip_items(outputs, SAPLING_OUTPUT_PROOF_LEN)?;
        self.skip(SIGNATURE_LEN)?;
        Some(balance)
    }

    fn action_value_balance(&mut self) -> Option<i64> {
        let actions = self.compact_size()?;
        if actions == 0 {
            return Some(0);
        }
        self.skip_items(actions, ACTION_LEN)?;
        self.skip(FLAGS_LEN)?;
        let balance = self.decode::<i64>()?;
        self.skip(ANCHOR_LEN)?;
        let proof_len = self.compact_size()?;
        self.skip(proof_len)?;
        self.skip_items(actions, SIGNATURE_LEN)?;
        self.skip(SIGNATURE_LEN)?;
        Some(balance)
    }

    fn skip_transparent(&mut self) -> Option<()> {
        self.decode::<Vec<TxIn>>()?;
        self.decode::<Vec<TxOut>>()?;
        Some(())
    }

    fn decode<T: Decodable>(&mut self) -> Option<T> {
        T::consensus_decode(&mut self.bytes).ok()
    }

    fn compact_size(&mut self) -> Option<usize> {
        usize::try_from(self.decode::<VarInt>()?.0).ok()
    }

    fn skip(&mut self, len: usize) -> Option<()> {
        self.bytes = self.bytes.get(len..)?;
        Some(())
    }

    fn skip_items(&mut self, count: usize, len: usize) -> Option<()> {
        self.skip(count.checked_mul(len)?)
    }
}

#[cfg(test)]
mod tests {
    use gem_bitcoin::models::Transaction;

    use super::*;

    #[test]
    fn test_read() {
        let hex = |json: &str| serde_json::from_str::<Transaction>(json).unwrap().hex.unwrap();
        let shielded_send = hex(include_str!("../../testdata/transaction_shielded_send_change.json"));
        let shielded_receive = hex(include_str!("../../testdata/transaction_shielded_receive.json"));
        let sprout_to_sapling = hex(include_str!("../../testdata/transaction_sprout_to_sapling.json"));
        let joinsplit = hex(include_str!("../../testdata/transaction_joinsplit.json"));

        assert_eq!(ValueBalanceReader::read(&shielded_send), Some(-200_000));
        assert_eq!(ValueBalanceReader::read(&shielded_receive), Some(2_015_000));
        assert_eq!(ValueBalanceReader::read(&sprout_to_sapling), Some(20_000));
        assert_eq!(ValueBalanceReader::read(&joinsplit), Some(-3_396_917_713));
        assert_eq!(ValueBalanceReader::read(&shielded_send[..shielded_send.len() - 2]), None);
        assert_eq!(ValueBalanceReader::read(&format!("{shielded_send}00")), None);
        assert_eq!(ValueBalanceReader::read("not hex"), None);
    }
}
