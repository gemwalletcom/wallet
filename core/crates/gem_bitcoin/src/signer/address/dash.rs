use primitives::SignerError;

use super::script::AddressScript;

pub(crate) const P2PKH_VERSIONS: [u8; 1] = [76];
const P2SH_VERSIONS: [u8; 1] = [16];

pub(super) fn script(address: &str) -> Result<AddressScript, SignerError> {
    AddressScript::from_prefixed_address(address, &P2PKH_VERSIONS, &P2SH_VERSIONS, None)
}
