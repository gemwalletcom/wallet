use primitives::AssetId;

use crate::BlockExplorer;
use crate::explorers::metadata::{ACCOUNT_PATH, ASSET_PATH, Explorer, Metadata, TX_PATH};

pub fn new() -> Box<dyn BlockExplorer> {
    Box::new(StellarExpert {
        explorer: Explorer::new(Metadata {
            name: "StellarExpert",
            base_url: "https://stellar.expert/explorer/public",
            tx_path: TX_PATH,
            address_path: ACCOUNT_PATH,
            token_path: Some(ASSET_PATH),
            nft_path: None,
            validator_path: None,
        }),
    })
}

struct StellarExpert {
    explorer: Explorer,
}

impl BlockExplorer for StellarExpert {
    fn name(&self) -> String {
        self.explorer.name()
    }

    fn get_tx_url(&self, hash: &str) -> String {
        self.explorer.get_tx_url(hash)
    }

    fn get_address_url(&self, address: &str) -> String {
        self.explorer.get_address_url(address)
    }

    fn get_token_url(&self, asset_id: &AssetId) -> Option<String> {
        let (issuer, code) = asset_id.split_sub_token_parts()?;
        self.explorer.token_url(&format!("{code}-{issuer}"))
    }
}

#[cfg(test)]
mod tests {
    use primitives::asset_constants::STELLAR_USDC_ASSET_ID;

    use super::*;

    #[test]
    fn test_stellar_expert_account_url() {
        let explorer = new();
        let address = "GBRCDFSBJFC4K3ZJV7WQVGDCROU6MCF3FDR3XGAMBG3ENKTGLNZHFKGJ";
        let url = explorer.get_address_url(address);
        assert_eq!(url, "https://stellar.expert/explorer/public/account/GBRCDFSBJFC4K3ZJV7WQVGDCROU6MCF3FDR3XGAMBG3ENKTGLNZHFKGJ");
    }

    #[test]
    fn test_stellar_expert_token_url() {
        let explorer = new();

        assert_eq!(
            explorer.get_token_url(&STELLAR_USDC_ASSET_ID),
            Some("https://stellar.expert/explorer/public/asset/USDC-GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN".to_string())
        );
    }

    #[test]
    fn test_stellar_expert_tx_url() {
        let explorer = new();
        let tx_hash = "249f71606fce57f3325e7abd8df7c8815594fe0fa986a2b6a838921190347bf6";
        let url = explorer.get_tx_url(tx_hash);
        assert_eq!(url, "https://stellar.expert/explorer/public/tx/249f71606fce57f3325e7abd8df7c8815594fe0fa986a2b6a838921190347bf6");
    }
}
