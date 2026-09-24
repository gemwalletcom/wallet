use gem_encoding::decode_base64;
use serde_json::Value;

use crate::{
    CompiledInstruction, Message, MessageAddressTableLookup, MessageHeader, Pubkey, SignatureBytes, TransactionConfig, VersionedMessageV0, VersionedMessageV1, VersionedTransaction,
    testkit::{TEST_BLOCKHASH, test_wallet_pubkey},
};

pub(crate) fn mock_transaction(programs: &[(&str, Vec<u8>)]) -> VersionedTransaction {
    let mut account_keys = vec![Pubkey::new([1; 32])];
    account_keys.extend(programs.iter().map(|(program, _)| Pubkey::from_base58(program).unwrap()));
    let instructions = programs.iter().enumerate().map(|(index, (_, data))| CompiledInstruction::mock((index + 1) as u8, vec![], data.clone())).collect();

    mock_transaction_with_accounts(account_keys, instructions)
}

#[cfg(feature = "signer")]
pub(crate) fn mock_legacy_transaction() -> VersionedTransaction {
    mock_transaction_with_accounts(vec![test_wallet_pubkey(), Pubkey::new([2; 32])], vec![CompiledInstruction::mock(1, vec![0], vec![])])
}

pub(crate) fn mock_transaction_with_accounts(account_keys: Vec<Pubkey>, instructions: Vec<CompiledInstruction>) -> VersionedTransaction {
    VersionedTransaction::Legacy {
        signatures: vec![],
        message: Message {
            header: MessageHeader::mock(1, (account_keys.len() - 1) as u8),
            account_keys,
            recent_blockhash: [0; 32],
            instructions,
        },
    }
}

pub(crate) fn mock_v0_transaction(account_keys: Vec<Pubkey>, instructions: Vec<CompiledInstruction>, address_table_lookups: Vec<MessageAddressTableLookup>) -> VersionedTransaction {
    VersionedTransaction::V0 {
        signatures: vec![SignatureBytes::default()],
        message: VersionedMessageV0 {
            message: Message {
                header: MessageHeader::mock(1, 1),
                account_keys,
                recent_blockhash: TEST_BLOCKHASH,
                instructions,
            },
            address_table_lookups,
        },
    }
}

pub(crate) fn mock_v1_transaction(signature_count: u8, wallet_index: u8) -> VersionedTransaction {
    let mut account_keys = (1..=signature_count).map(|value| Pubkey::new([value; 32])).collect::<Vec<_>>();
    if let Some(account) = account_keys.get_mut(wallet_index as usize) {
        *account = test_wallet_pubkey();
    }
    account_keys.push(Pubkey::new([100; 32]));
    VersionedTransaction::V1 {
        signatures: (0..signature_count).map(|value| if value == wallet_index { SignatureBytes::default() } else { SignatureBytes::new([value + 1; 64]) }).collect(),
        message: VersionedMessageV1::mock(signature_count, account_keys, vec![CompiledInstruction::mock(signature_count, vec![0], vec![0xde, 0xad])], TransactionConfig::mock()),
    }
}

pub(crate) const TEST_LEGACY_TX: &str = "AQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAAgWAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEbrtjJdvWJAv9GZTGL8LaZtMvDe4j2ery4z7rOkRbioxZflXLFqWqlAt1REFSiam0ljvfB1tbBruEpGRTcUQIyQ+ddH9NRneQZQXje5U/3c4cZ2f1JESi76CvBvRoQ6I1LeNzfZ4ZONkowCnqCyeo5+D6Q21gn3U7HVw/KD3HyUW5gVpu5F8ZojWkXLg/+3N6q3ojiaqYyBIbz7VP7jS5Yktrxv5b22C/EFSDs5jUPA7Gz3GLdBNs0iwBHlqUqNEeyNpDX0HWNHV2LiVDOx6m018ea6P+1xroNvWKhmDeTW7oqHXAEK1ih5IO68BBiiKqWNR5VZdBgBsnR+rZKfpfuyE3yQziYO+SoWzCXuvQLyVcRCNKJrACzaN8XXUR1z3rOt8T1lYUIIAQS7tqgcLRsn18N4vVQgXQyv3bQWjh3JtpQT3Bgy9N9myGC4PDjGuVnx2Y7mF4eqlysb0rgrdrB2+FMK6YBPXtlXF4QPTY6rEe+hxkBpCoGK7UJu5BHUK4gJhAewgMolkoyq6sTbFQFuR86447k9ky2veh5uGg40gAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAjJclj04kifG7PRApFI4NgwtaE5na/xCEBI572Nvp+FkDBkZv5SEXMv/srbpyw5vnvIzlu8X3EmssQ5s6QAAAAMb6evO+2606PWXzaqvJdDGxu+TC0vbg5HymAgNFL11hBUpTWpkpIQZNJOhxYNo4fHw1td28kruB5B+oQEEFRI0Gm4hX/quBhPtof2NGGMA12sQ53BrrO1WYoPAAAAAAAQbd9uHXZaGT2cvhRs7reawctIXtX1s3kTqM9YV+/wCpDgNoX46QkFPkWBIcZvWnau3HcGqhHIL4qpUqjyt4ealuCa42Moiy1mB8REcWJlkis4eCMyKfY2HMRfldn8r2XwcQAAUCoGgGABAACQNwEQEAAAAAAA8GAAYAEw4UAQAVERQUEgAHExEGCQoCBAULDAgBMSsE7QsayR5iC50OAAAAAAA8XqkAAAAAAAAAAAAAAAAAAAAAAAAAAAABAAEBAAAABgIUAwYAAAEJFAMKAwAJA8wSAAAAAAAADgIADQwCAAAAODEAAAAAAAA=";

pub(crate) const TEST_MAYAN_V0_TX: &str = "ATzYOiofQZSWsNe3SxxEPip+Xp9A2Fji+h0xfs7FkmvQxNNgwjeEbTlMr7+e42q9vcvExw2CX4PgNBRuY77O+waAAQAEDPlBHYJN7SVAqQdmNtdFQsCIDVJuEnf59VTtTCOGI7yLh4jpmImexNtJSORTO+sbJ63Aysdx88si41jIW1Wf65qHxwlVbaZ8xI24o/VzmleK1NqPB2lMTcy78ZFbqJ6agIqQAqWC7XmuIVDA/VxhSMZPxFOazPZMJbWyD+TYtXxA3sS/qzC61MydFxPOY3xt62Ug5Tp3r/hC0NimkXNfrMH0UmoX+WTY7c2jVeACjg8EqVgtZZSXgaQRvotGaelPhCySBd5s0S8tvrZZSGGBUknE3Jjh4aGsgXpNY0QHkFnJayU0QDsmAQ7sF/E5yI6Oq1k8w8tnKB6wJR28JzZwp3KVGAf9PgfpG6VoBYOYtT4QWhLzz8wJo5Da/9f9tVVfo7Qj5Z1paZLqq3kUJ1PAm9bYE1qpQE9jUkcSHEnSn0OVAwZGb+UhFzL/7K26csOb57yM5bvF9xJrLEObOkAAAAAGTCSuZOXkbU4/LKndRkF4gm16E7to0DdpTPoefoS0rYF08m4FFLws+yIpIkWYIyALDIz0sekCn1BgZGSqLNo5CwsAF0FkUEJ2ZE5kVGxlWmNsc25JeDVkeUExCgAFAhxCBwAKAAkDBBcBAAAAAAAJBxUABgUWGRQACQcVAAEAGxkUAQEJAxkAAQwCAAAAC/UHPQYAAAAJAhQBAREJBxUAAwAWGRQBAQgoHAABAxsWFBQGHRwhACITAQMPERIUBBACBxweAA4gHw0MAQMbFhQUGjIBLQAAALtk+swxxK8UC/UHPQYAAAD8nvqKAAAAAGQAAAAAAAIAAAAaQAYAAl8A0CAAAgkEFAEAAAEJCQoYAAAFBgMWFxQZxgEgTCkMJ6KE2yRjS4oAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAhOLnPgTBemY6Iqc117gEL72WnXMeAAAAAAAAAAAAAAAAAIM1ifzW7bbgj0x8MtT3G1S9oCkT/ySLiQAAAAAAAAAAAAAAAG4DAAAAAAAA8dcFAAAAAAA7a4ppAAAAAAAAAAAAAAAAAAAAAN3bmpXkQ6IE64ZQ1epXjtcH/iEjAAMC0SsrhG32PclzqA5blk8lqOrlLpR5OoOt60ksQpGLgw4D2cMN+5YRja4DNaX11bThHmwP8vCzdRYtSPXpFGWT2KsACgUGESAhKSowMTQme3jXiWKuyj4qkRn+CZK3WspZpXBM+tnHyaYm4WA/BAMICgsDBgkMIbxt+8RM8X78HZP9nB+0Ah2xfOX9io4UH0AdkLgPT00Fdnd6fH4CdHU=";

pub(crate) fn mock_v1_mainnet_transaction_bytes() -> Vec<u8> {
    let fixture: Value = serde_json::from_str(include_str!("../../testdata/transaction_v1_mainnet.json")).unwrap();
    decode_base64(fixture["transaction"][0].as_str().unwrap()).unwrap()
}
