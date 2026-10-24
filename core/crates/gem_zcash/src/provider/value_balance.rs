use bitcoin::{TxIn, TxOut, VarInt, consensus::Decodable};

const VERSION_MASK: u32 = 0x7fff_ffff;
const V4_VERSION_GROUP_LEN: usize = 4;
const V5_HEADER_TAIL_LEN: usize = 16;
const LOCK_TIME_AND_EXPIRY_LEN: usize = 8;
const V4_SAPLING_SPEND_LEN: usize = 384;
const V4_SAPLING_OUTPUT_LEN: usize = 948;
const JOINSPLIT_VALUES_LEN: usize = 16;
const JOINSPLIT_LEN: usize = 1698;
const JOINSPLIT_PUBKEY_LEN: usize = 32;
const SAPLING_SPEND_LEN: usize = 96;
const SAPLING_OUTPUT_LEN: usize = 756;
const SAPLING_SPEND_PROOF_AND_SIGNATURE_LEN: usize = 192 + 64;
const SAPLING_OUTPUT_PROOF_LEN: usize = 192;
const ACTION_LEN: usize = 820;
const FLAGS_LEN: usize = 1;
const ANCHOR_LEN: usize = 32;
const SIGNATURE_LEN: usize = 64;

pub(super) struct ValueBalanceReader<'a> {
    bytes: &'a [u8],
}

impl ValueBalanceReader<'_> {
    pub(super) fn read(hex: &str) -> Option<i64> {
        let bytes = hex::decode(hex).ok()?;
        ValueBalanceReader { bytes: &bytes }.value_balance()
    }

    fn value_balance(mut self) -> Option<i64> {
        let balance = match self.decode::<u32>()? & VERSION_MASK {
            4 => self.v4_value_balance()?,
            5 => self.v5_value_balance()?,
            6 => self.v5_value_balance()?.checked_add(self.action_value_balance()?)?,
            _ => return None,
        };
        self.bytes.is_empty().then_some(balance)
    }

    fn v4_value_balance(&mut self) -> Option<i64> {
        self.skip(V4_VERSION_GROUP_LEN)?;
        self.skip_transparent()?;
        self.skip(LOCK_TIME_AND_EXPIRY_LEN)?;
        let sapling = self.decode::<i64>()?;
        let spends = self.compact_size()?;
        self.skip_items(spends, V4_SAPLING_SPEND_LEN)?;
        let outputs = self.compact_size()?;
        self.skip_items(outputs, V4_SAPLING_OUTPUT_LEN)?;
        let sprout = self.sprout_value_balance()?;
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

    fn sprout_value_balance(&mut self) -> Option<i64> {
        let joinsplits = self.compact_size()?;
        let mut balance = 0i64;
        for _ in 0..joinsplits {
            let vpub_old = i64::try_from(self.decode::<u64>()?).ok()?;
            let vpub_new = i64::try_from(self.decode::<u64>()?).ok()?;
            self.skip(JOINSPLIT_LEN - JOINSPLIT_VALUES_LEN)?;
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
    use super::*;

    #[test]
    fn test_read() {
        let signed = include_str!("../../testdata/zcash_shielded_transfer.hex").trim();

        assert_eq!(ValueBalanceReader::read(signed), Some(-20_000));
        assert_eq!(ValueBalanceReader::read(&signed[..signed.len() - 2]), None);
        assert_eq!(ValueBalanceReader::read(&format!("{signed}00")), None);
        assert_eq!(ValueBalanceReader::read("not hex"), None);

        let sprout_to_sapling = include_str!("../../testdata/transaction_sprout_to_sapling.hex").trim();
        assert_eq!(ValueBalanceReader::read(sprout_to_sapling), Some(20_000));
        assert_eq!(ValueBalanceReader::read(&sprout_to_sapling[..sprout_to_sapling.len() - 2]), None);
    }
}
