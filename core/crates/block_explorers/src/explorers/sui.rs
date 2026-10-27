use crate::BlockExplorer;
use crate::explorers::metadata::{ACCOUNT_PATH, COIN_PATH, Explorer, Metadata, TX_PATH, VALIDATOR_PATH};

pub fn new_sui_scan() -> Box<dyn BlockExplorer> {
    Explorer::boxed(Metadata {
        name: "SuiScan",
        base_url: "https://suiscan.xyz/mainnet",
        tx_path: TX_PATH,
        address_path: ACCOUNT_PATH,
        token_path: Some(COIN_PATH),
        nft_path: None,
        validator_path: Some(VALIDATOR_PATH),
    })
}

#[cfg(test)]
mod tests {
    use primitives::{AssetId, Chain};

    use super::*;

    #[test]
    fn test_sui_scan_urls() {
        let explorer = new_sui_scan();

        assert_eq!(
            explorer.get_address_url("0x6f02af629f66a13c5b8cb857cddf43804422d205b0bb9bda9db98b2635fe59bb"),
            "https://suiscan.xyz/mainnet/account/0x6f02af629f66a13c5b8cb857cddf43804422d205b0bb9bda9db98b2635fe59bb"
        );
        assert_eq!(
            explorer.get_tx_url("ArS7DzeHUA54ccRG12SqEZwt7snQePcanZ77Mkm2KRos"),
            "https://suiscan.xyz/mainnet/tx/ArS7DzeHUA54ccRG12SqEZwt7snQePcanZ77Mkm2KRos"
        );
        assert_eq!(explorer.get_token_url(&AssetId::from_token(Chain::Sui, "token123")), Some("https://suiscan.xyz/mainnet/coin/token123".to_string()));

        assert_eq!(explorer.get_validator_url("val123"), Some("https://suiscan.xyz/mainnet/validator/val123".to_string()));
    }
}
