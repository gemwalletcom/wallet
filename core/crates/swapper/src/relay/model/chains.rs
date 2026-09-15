use std::collections::BTreeSet;

use serde::Deserialize;

use crate::relay::chain::RelayChain;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayChainsResponse {
    pub chains: Vec<RelayChainInfo>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayChainInfo {
    pub id: u64,
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
        Self::unique(self.chains.iter().filter_map(RelayChainInfo::depository))
    }

    pub fn depository(&self, chain_id: u64) -> Option<String> {
        self.chains.iter().find(|chain| chain.id == chain_id)?.depository()
    }

    pub fn send_addresses(&self) -> Vec<String> {
        Self::unique(self.chains.iter().flat_map(RelayChainInfo::solvers))
    }

    fn unique(addresses: impl Iterator<Item = String>) -> Vec<String> {
        addresses.collect::<BTreeSet<_>>().into_iter().collect()
    }
}

impl RelayChainInfo {
    fn depository(&self) -> Option<String> {
        let depository = self.protocol.as_ref()?.v2.as_ref()?.depository.as_deref()?;
        Some(RelayChain::from_chain_id(self.id)?.checksum_address(depository))
    }

    fn solvers(&self) -> Vec<String> {
        RelayChain::from_chain_id(self.id)
            .map(|chain| self.solver_addresses.iter().map(|address| chain.checksum_address(address)).collect())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        SwapperProvider,
        cross_chain::{DepositAddressMap, SendAddressMap, is_from_vault_address, swap_provider_with_vault_addresses},
    };
    use primitives::{AssetId, Chain, Transaction, TransactionUtxoInput};

    #[test]
    fn test_bitcoin_vault_indexing() {
        let response = RelayChainsResponse {
            chains: vec![RelayChainInfo::mock(
                8253038,
                Some("bc1qzmtn0q92ayejt2hpffvlktcpmyy7vvsd06sefu"),
                &["bc1qq2mvrp4g3ugd424dw4xv53rgsf8szkrv853jrc"],
            )],
        };
        let deposits = DepositAddressMap::from_iter(response.deposit_addresses().into_iter().map(|address| (address, SwapperProvider::Relay)));
        let senders = SendAddressMap::from_iter(response.send_addresses().into_iter().map(|address| (address, SwapperProvider::Relay)));
        let transaction = Transaction {
            asset_id: AssetId::from_chain(Chain::Bitcoin),
            memo: Some("0xf52d0ea86b4cf88ae40456dec58076f4261ad3a350485c6bd0a3095940570099".to_string()),
            ..Transaction::mock_utxo(
                vec![TransactionUtxoInput::new("sender".into(), 80_000u32.into())],
                vec![TransactionUtxoInput::new("bc1qzmtn0q92ayejt2hpffvlktcpmyy7vvsd06sefu".into(), 75_357u32.into())],
            )
        };
        assert_eq!(response.depository(8253038).as_deref(), Some("bc1qzmtn0q92ayejt2hpffvlktcpmyy7vvsd06sefu"));
        assert_eq!(response.depository(1), None);
        assert_eq!(swap_provider_with_vault_addresses(&transaction, &deposits), Some(SwapperProvider::Relay));
        let transaction = Transaction {
            utxo_inputs: Some(vec![TransactionUtxoInput::new("bc1qq2mvrp4g3ugd424dw4xv53rgsf8szkrv853jrc".into(), 140_000u32.into())]),
            ..transaction
        };
        assert!(is_from_vault_address(&transaction, &senders));
    }

    #[test]
    fn test_vault_addresses() {
        let response = RelayChainsResponse {
            chains: vec![
                RelayChainInfo::mock(1, Some("0x59916da825d2d2ec1bf878d71c88826f6633ecca"), &["0xf70da97812cb96acdf810712aa562db8dfa3dbef"]),
                RelayChainInfo::mock(8453, Some("0x4cd00e387622c35bddb9b4c962c136462338bc31"), &["0xf70da97812cb96acdf810712aa562db8dfa3dbef"]),
                RelayChainInfo::mock(
                    792703809,
                    Some("99vQwtBwYtrqqD9YSXbdum3KBdxPAVxYTaQ3cfnJSrN2"),
                    &["DNLbQ4t95LLPevvLdcFzN8RHNN83ntQJLt26E8VTnE7p"],
                ),
                RelayChainInfo::mock(
                    224235520,
                    Some("EQCrdGsDTqA2t6xRR4N6V4J705F7w_VQbUdHnofsh-8lVIPs"),
                    &["UQDBrIuXWeqPGbjyyNsUjqiTRTBWhlJkjoURNtVvNjYssR87"],
                ),
                RelayChainInfo::mock(728126428, None, &["TYVWGh8XkmU49Hi9PkGAZXiiJPB3J5zJZy"]),
                RelayChainInfo::mock(537724, Some("rJBdWA9p5KwBoqSQTyMdg3UHLsJVzGVu5m"), &["rE6xRr2GbS31KPoL9RaLgfiarJ4vqXj8Si"]),
            ],
        };

        assert_eq!(
            response.deposit_addresses(),
            vec![
                "0x4cD00E387622C35bDDB9b4c962C136462338BC31",
                "0x59916DA825D2D2eC1BF878D71c88826F6633ecca",
                "99vQwtBwYtrqqD9YSXbdum3KBdxPAVxYTaQ3cfnJSrN2",
                "UQCrdGsDTqA2t6xRR4N6V4J705F7w_VQbUdHnofsh-8lVN4p",
            ]
        );
        assert_eq!(
            response.send_addresses(),
            vec![
                "0xf70da97812CB96acDF810712Aa562db8dfA3dbEF",
                "DNLbQ4t95LLPevvLdcFzN8RHNN83ntQJLt26E8VTnE7p",
                "TYVWGh8XkmU49Hi9PkGAZXiiJPB3J5zJZy",
                "UQDBrIuXWeqPGbjyyNsUjqiTRTBWhlJkjoURNtVvNjYssR87",
            ]
        );
    }
}
