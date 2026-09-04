use std::collections::BTreeSet;

use gem_evm::address::ethereum_address_checksum;
use gem_ton::Address as TonAddress;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayChainsResponse {
    pub chains: Vec<RelayChainInfo>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayChainInfo {
    #[serde(default)]
    pub solver_addresses: Vec<String>,
    pub protocol: Option<RelayProtocol>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayProtocol {
    pub v2: Option<RelayProtocolV2>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayProtocolV2 {
    pub depository: Option<String>,
}

impl RelayChainsResponse {
    pub fn deposit_addresses(&self) -> Vec<String> {
        normalize_addresses(self.chains.iter().filter_map(|chain| chain.protocol.as_ref()?.v2.as_ref()?.depository.as_deref()))
    }

    pub fn send_addresses(&self) -> Vec<String> {
        normalize_addresses(self.chains.iter().flat_map(|chain| chain.solver_addresses.iter().map(String::as_str)))
    }
}

fn normalize_addresses<'a>(addresses: impl Iterator<Item = &'a str>) -> Vec<String> {
    addresses.map(normalize_address).collect::<BTreeSet<_>>().into_iter().collect()
}

fn normalize_address(address: &str) -> String {
    if let Ok(checksum) = ethereum_address_checksum(address) {
        return checksum;
    }
    match TonAddress::try_parse_base64(address) {
        Some(ton_address) => ton_address.encode_non_bounceable(),
        None => address.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deposit_addresses() {
        let response = RelayChainsResponse {
            chains: vec![
                RelayChainInfo::mock_depository(Some("0x59916da825d2d2ec1bf878d71c88826f6633ecca")),
                RelayChainInfo::mock_depository(Some("0x4cd00e387622c35bddb9b4c962c136462338bc31")),
                RelayChainInfo::mock_depository(Some("EQCrdGsDTqA2t6xRR4N6V4J705F7w_VQbUdHnofsh-8lVIPs")),
                RelayChainInfo::mock_depository(None),
            ],
        };

        assert_eq!(
            response.deposit_addresses(),
            vec![
                "0x4cD00E387622C35bDDB9b4c962C136462338BC31",
                "0x59916DA825D2D2eC1BF878D71c88826F6633ecca",
                "UQCrdGsDTqA2t6xRR4N6V4J705F7w_VQbUdHnofsh-8lVN4p",
            ]
        );
    }

    #[test]
    fn test_send_addresses() {
        let response = RelayChainsResponse {
            chains: vec![
                RelayChainInfo::mock_solvers(&["0xf70da97812cb96acdf810712aa562db8dfa3dbef", "UQDBrIuXWeqPGbjyyNsUjqiTRTBWhlJkjoURNtVvNjYssR87"]),
                RelayChainInfo::mock_solvers(&["0xf70da97812cb96acdf810712aa562db8dfa3dbef", "TYVWGh8XkmU49Hi9PkGAZXiiJPB3J5zJZy"]),
            ],
        };

        assert_eq!(
            response.send_addresses(),
            vec![
                "0xf70da97812CB96acDF810712Aa562db8dfA3dbEF",
                "TYVWGh8XkmU49Hi9PkGAZXiiJPB3J5zJZy",
                "UQDBrIuXWeqPGbjyyNsUjqiTRTBWhlJkjoURNtVvNjYssR87",
            ]
        );
    }
}
