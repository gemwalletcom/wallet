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
}

impl RelayChainsResponse {
    pub fn send_addresses(&self) -> Vec<String> {
        self.chains.iter().flat_map(RelayChainInfo::solvers).collect::<BTreeSet<_>>().into_iter().collect()
    }
}

impl RelayChainInfo {
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
        cross_chain::{SendAddressMap, is_from_vault_address},
    };
    use primitives::{AssetId, Chain, Transaction, TransactionUtxoInput};

    #[test]
    fn test_bitcoin_vault_indexing() {
        let response = RelayChainsResponse {
            chains: vec![RelayChainInfo::mock(8253038, &["bc1qq2mvrp4g3ugd424dw4xv53rgsf8szkrv853jrc"])],
        };
        let senders = SendAddressMap::from_iter(response.send_addresses().into_iter().map(|address| (address, SwapperProvider::Relay)));
        let transaction = Transaction {
            asset_id: AssetId::from_chain(Chain::Bitcoin),
            ..Transaction::mock_utxo(
                vec![TransactionUtxoInput::new("bc1qq2mvrp4g3ugd424dw4xv53rgsf8szkrv853jrc".into(), 140_000u32.into())],
                vec![TransactionUtxoInput::new("receiver".into(), 135_000u32.into())],
            )
        };
        assert!(is_from_vault_address(&transaction, &senders));
    }

    #[test]
    fn test_send_addresses() {
        let response = RelayChainsResponse {
            chains: vec![
                RelayChainInfo::mock(1, &["0xf70da97812cb96acdf810712aa562db8dfa3dbef"]),
                RelayChainInfo::mock(8453, &["0xf70da97812cb96acdf810712aa562db8dfa3dbef"]),
                RelayChainInfo::mock(792703809, &["DNLbQ4t95LLPevvLdcFzN8RHNN83ntQJLt26E8VTnE7p"]),
                RelayChainInfo::mock(224235520, &["UQDBrIuXWeqPGbjyyNsUjqiTRTBWhlJkjoURNtVvNjYssR87"]),
                RelayChainInfo::mock(728126428, &["TYVWGh8XkmU49Hi9PkGAZXiiJPB3J5zJZy"]),
                RelayChainInfo::mock(537724, &["rE6xRr2GbS31KPoL9RaLgfiarJ4vqXj8Si"]),
            ],
        };

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
