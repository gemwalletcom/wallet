use gem_encoding::borsh::{BorshDeserialize, DecodeError, Reader};
use primitives::swap::HUNDRED_PERCENT_IN_BPS;

use crate::{Pubkey, SolanaError, error::Result as SolanaResult};

const MINT_LEN: usize = 82;
const SUPPLY_OFFSET: usize = 36;
const ACCOUNT_TYPE_OFFSET: usize = 165;
const EXTENSIONS_OFFSET: usize = 166;
const MINT_ACCOUNT_TYPE: u8 = 1;
const TRANSFER_FEES_OFFSET: usize = 72;
const FROZEN_ACCOUNT_STATE: u8 = 2;

const UNINITIALIZED: u16 = 0;
const TRANSFER_FEE_CONFIG: u16 = 1;
const DEFAULT_ACCOUNT_STATE: u16 = 6;
const NON_TRANSFERABLE: u16 = 9;
const TRANSFER_HOOK: u16 = 14;
const PAUSABLE: u16 = 26;

struct ExtensionHeader {
    extension_type: u16,
    length: u16,
}

impl BorshDeserialize for ExtensionHeader {
    fn deserialize_reader(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            extension_type: reader.read()?,
            length: reader.read()?,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransferFee {
    pub epoch: u64,
    pub maximum_fee: u64,
    pub basis_points: u16,
}

impl BorshDeserialize for TransferFee {
    fn deserialize_reader(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            epoch: reader.read()?,
            maximum_fee: reader.read()?,
            basis_points: reader.read()?,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransferFeeConfig {
    pub older: TransferFee,
    pub newer: TransferFee,
}

impl BorshDeserialize for TransferFeeConfig {
    fn deserialize_reader(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            older: reader.read()?,
            newer: reader.read()?,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MintExtension {
    TransferFee(TransferFeeConfig),
    TransferHook { program_id: Option<Pubkey> },
    NonTransferable,
    DefaultAccountState { frozen: bool },
    Pausable { paused: bool },
    Other(u16),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Mint {
    pub supply: u64,
    pub extensions: Vec<MintExtension>,
}

impl Mint {
    pub fn decode(data: &[u8]) -> SolanaResult<Self> {
        let invalid = || SolanaError::invalid_input("Invalid mint account");
        let supply = data.get(SUPPLY_OFFSET..).and_then(<[u8]>::first_chunk).map(|bytes| u64::from_le_bytes(*bytes)).ok_or_else(invalid)?;
        let extensions = match (data.len(), data.get(ACCOUNT_TYPE_OFFSET)) {
            (MINT_LEN, _) => vec![],
            (_, Some(&MINT_ACCOUNT_TYPE)) => decode_extensions(&data[EXTENSIONS_OFFSET..])?,
            _ => return Err(invalid()),
        };
        Ok(Self { supply, extensions })
    }

    pub fn transfer_fee(&self, epoch: u64, amount: u64) -> u64 {
        self.extensions
            .iter()
            .find_map(|extension| match extension {
                MintExtension::TransferFee(config) => Some(config.fee(epoch, amount)),
                _ => None,
            })
            .unwrap_or_default()
    }

    pub fn has_transfer_restrictions(&self) -> bool {
        self.extensions.iter().any(|extension| match extension {
            MintExtension::TransferHook { program_id } => program_id.is_some(),
            MintExtension::NonTransferable => true,
            MintExtension::DefaultAccountState { frozen } => *frozen,
            MintExtension::Pausable { paused } => *paused,
            MintExtension::TransferFee(_) | MintExtension::Other(_) => false,
        })
    }
}

impl TransferFeeConfig {
    fn fee(&self, epoch: u64, amount: u64) -> u64 {
        let fee = if epoch >= self.newer.epoch { self.newer } else { self.older };
        let raw_fee = (u128::from(amount) * u128::from(fee.basis_points)).div_ceil(u128::from(HUNDRED_PERCENT_IN_BPS));
        u64::try_from(raw_fee).unwrap_or(u64::MAX).min(fee.maximum_fee)
    }
}

fn decode_extensions(mut tlv: &[u8]) -> SolanaResult<Vec<MintExtension>> {
    let invalid = || SolanaError::invalid_input("Invalid mint extension");
    let mut extensions = vec![];
    while let Ok(header) = ExtensionHeader::deserialize(&mut tlv)
        && header.extension_type != UNINITIALIZED
    {
        let (value, rest) = tlv.split_at_checked(usize::from(header.length)).ok_or_else(invalid)?;
        extensions.push(decode_extension(header.extension_type, value)?);
        tlv = rest;
    }
    Ok(extensions)
}

fn decode_extension(extension_type: u16, value: &[u8]) -> SolanaResult<MintExtension> {
    let invalid = || SolanaError::invalid_input("Invalid mint extension");
    match extension_type {
        TRANSFER_FEE_CONFIG => {
            let fees = value.get(TRANSFER_FEES_OFFSET..).ok_or_else(invalid)?;
            Ok(MintExtension::TransferFee(TransferFeeConfig::deserialize(&mut &fees[..]).map_err(|_| invalid())?))
        }
        TRANSFER_HOOK => {
            let program_id = value.get(32..64).ok_or_else(invalid)?;
            Ok(MintExtension::TransferHook {
                program_id: program_id.iter().any(|byte| *byte != 0).then(|| Pubkey::new(program_id.try_into().unwrap_or_default())),
            })
        }
        NON_TRANSFERABLE => Ok(MintExtension::NonTransferable),
        DEFAULT_ACCOUNT_STATE => Ok(MintExtension::DefaultAccountState {
            frozen: *value.first().ok_or_else(invalid)? == FROZEN_ACCOUNT_STATE,
        }),
        PAUSABLE => Ok(MintExtension::Pausable {
            paused: *value.get(32).ok_or_else(invalid)? != 0,
        }),
        other => Ok(MintExtension::Other(other)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gem_encoding::decode_base64;

    fn decode(data: &str) -> Mint {
        Mint::decode(&decode_base64(data.trim()).unwrap()).unwrap()
    }

    #[test]
    fn test_decode() {
        let fee = |epoch, basis_points| TransferFee { epoch, maximum_fee: u64::MAX, basis_points };

        assert_eq!(
            decode(include_str!("../../testdata/mint_usdc.base64")),
            Mint {
                supply: 7_901_856_805_554_361,
                extensions: vec![]
            }
        );
        assert_eq!(
            decode(include_str!("../../testdata/mint_transfer_fee.base64")),
            Mint {
                supply: 957_493_468_947_067,
                extensions: vec![MintExtension::TransferFee(TransferFeeConfig {
                    older: fee(1035, 200),
                    newer: fee(1035, 200)
                })],
            }
        );
        assert_eq!(
            decode(include_str!("../../testdata/mint_xstock.base64")),
            Mint {
                supply: 32_126_240_400_142,
                extensions: vec![
                    MintExtension::Other(18),
                    MintExtension::Other(12),
                    MintExtension::DefaultAccountState { frozen: false },
                    MintExtension::Other(25),
                    MintExtension::Pausable { paused: false },
                    MintExtension::Other(4),
                    MintExtension::TransferHook { program_id: None },
                    MintExtension::Other(19),
                ],
            }
        );
        assert_eq!(Mint::decode(&[0; 10]), Err(SolanaError::invalid_input("Invalid mint account")));
        assert_eq!(Mint::decode(&[0; 165]), Err(SolanaError::invalid_input("Invalid mint account")));
        assert_eq!(
            Mint::decode(&[vec![0; ACCOUNT_TYPE_OFFSET], vec![MINT_ACCOUNT_TYPE], [NON_TRANSFERABLE.to_le_bytes(), [0; 2]].concat().repeat(20_000)].concat()),
            Ok(Mint {
                supply: 0,
                extensions: vec![MintExtension::NonTransferable; 20_000]
            })
        );
    }

    #[test]
    fn test_transfer_fee() {
        let flat = decode(include_str!("../../testdata/mint_transfer_fee.base64"));
        let schedule = decode(include_str!("../../testdata/mint_transfer_fee_schedule.base64"));
        let capped = TransferFeeConfig {
            older: TransferFee { epoch: 0, maximum_fee: 5, basis_points: 200 },
            newer: TransferFee { epoch: 0, maximum_fee: 5, basis_points: 200 },
        };

        assert_eq!(flat.transfer_fee(1_059, 1_000_000), 20_000);
        assert_eq!(flat.transfer_fee(1_059, 1), 1);
        assert_eq!(flat.transfer_fee(1_059, 0), 0);
        assert_eq!(schedule.transfer_fee(1_049, 1_000_000), 50_000);
        assert_eq!(schedule.transfer_fee(1_050, 1_000_000), 10_000);
        assert_eq!(capped.fee(1, 1_000_000), 5);
        assert_eq!(decode(include_str!("../../testdata/mint_usdc.base64")).transfer_fee(1_059, 1_000_000), 0);
    }

    #[test]
    fn test_has_transfer_restrictions() {
        let restricted = |extension: MintExtension| Mint { supply: 0, extensions: vec![extension] }.has_transfer_restrictions();

        assert!(!decode(include_str!("../../testdata/mint_xstock.base64")).has_transfer_restrictions());
        assert!(!decode(include_str!("../../testdata/mint_transfer_fee_schedule.base64")).has_transfer_restrictions());
        assert!(restricted(MintExtension::TransferHook { program_id: Some(Pubkey::new([1; 32])) }));
        assert!(restricted(MintExtension::NonTransferable));
        assert!(restricted(MintExtension::DefaultAccountState { frozen: true }));
        assert!(restricted(MintExtension::Pausable { paused: true }));
    }
}
