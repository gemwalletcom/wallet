use crate::block_explorer::BlockExplorer;
use crate::explorers::metadata::{ADDRESS_PATH, Explorer, Metadata, TOKEN_PATH, TX_PATH};

pub struct TempoExplorer;

impl TempoExplorer {
    pub fn boxed() -> Box<dyn BlockExplorer> {
        Explorer::boxed(Metadata {
            name: "Tempo Explorer",
            base_url: "https://explore.tempo.xyz",
            tx_path: TX_PATH,
            address_path: ADDRESS_PATH,
            token_path: Some(TOKEN_PATH),
            nft_path: None,
            validator_path: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tempo_explorer_urls() {
        let explorer = TempoExplorer::boxed();
        let transaction_hash = "0x6e2f0394ec87001f207e93d1ec0b125981e860c876d7e4dadd9761cb893ce66e";
        let address = "0x9858EfFD232B4033E47d90003D41EC34EcaEda94";
        let token = "0x20C000000000000000000000b9537d11c60E8b50";

        assert_eq!(
            explorer.get_tx_url(transaction_hash),
            "https://explore.tempo.xyz/tx/0x6e2f0394ec87001f207e93d1ec0b125981e860c876d7e4dadd9761cb893ce66e"
        );
        assert_eq!(
            explorer.get_address_url(address),
            "https://explore.tempo.xyz/address/0x9858EfFD232B4033E47d90003D41EC34EcaEda94"
        );
        assert_eq!(
            explorer.get_token_url(token).unwrap(),
            "https://explore.tempo.xyz/token/0x20C000000000000000000000b9537d11c60E8b50"
        );
    }
}
