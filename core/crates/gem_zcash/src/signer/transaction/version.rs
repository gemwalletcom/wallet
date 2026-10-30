use orchard::bundle::TxVersion;

const OVERWINTERED_VERSION_5: u32 = 0x8000_0005;
const OVERWINTERED_VERSION_6: u32 = 0x8000_0006;
const VERSION_GROUP_ID_V5: u32 = 0x26a7_270a;
const VERSION_GROUP_ID_V6: u32 = 0xd884_b698;

#[derive(Debug, Clone, Copy)]
pub(in crate::signer) enum TransactionVersion {
    V5,
    V6,
}

impl TransactionVersion {
    pub(super) fn header(self) -> u32 {
        match self {
            Self::V5 => OVERWINTERED_VERSION_5,
            Self::V6 => OVERWINTERED_VERSION_6,
        }
    }

    pub(super) fn version_group_id(self) -> u32 {
        match self {
            Self::V5 => VERSION_GROUP_ID_V5,
            Self::V6 => VERSION_GROUP_ID_V6,
        }
    }

    pub(super) fn tx_version(self) -> TxVersion {
        match self {
            Self::V5 => TxVersion::V5,
            Self::V6 => TxVersion::V6,
        }
    }
}
