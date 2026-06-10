use primitives::SignerError;

use super::script::AddressScript;

const P2PKH_VERSIONS: [u8; 1] = [48];
// Modern LTC P2SH (50, `M…`) only; legacy 5 (`3…`) collides with Bitcoin mainnet P2SH.
const P2SH_VERSIONS: [u8; 1] = [50];
pub(crate) const HRP: &str = "ltc";

pub(super) fn script(address: &str) -> Result<AddressScript, SignerError> {
    AddressScript::from_prefixed_address(address, &P2PKH_VERSIONS, &P2SH_VERSIONS, Some(HRP))
}
