use super::model::AssetImage;
use primitives::Chain;
use std::{collections::HashMap, sync::LazyLock};

const EVM_NATIVE_TOKEN_ID: &str = "0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";
const ZERO_ADDRESS_TOKEN_ID: &str = "0x0000000000000000000000000000000000000000";
const POLYGON_NATIVE_TOKEN_ID: &str = "0x0000000000000000000000000000000000001010";
const ZKSYNC_NATIVE_TOKEN_ID: &str = "0x000000000000000000000000000000000000800a";
const COMMON_NATIVE_TOKEN_IDS: &[&str] = &[EVM_NATIVE_TOKEN_ID, ZERO_ADDRESS_TOKEN_ID];

static CHAIN_NATIVE_TOKEN_IDS: LazyLock<HashMap<Chain, &[&str]>> =
    LazyLock::new(|| HashMap::from([(Chain::Polygon, &[POLYGON_NATIVE_TOKEN_ID][..]), (Chain::ZkSync, &[ZKSYNC_NATIVE_TOKEN_ID][..])]));

pub fn is_native_token(image: &AssetImage) -> bool {
    let token_id = image.token_id.to_lowercase();
    COMMON_NATIVE_TOKEN_IDS.contains(&token_id.as_str())
        || CHAIN_NATIVE_TOKEN_IDS.get(&image.chain).is_some_and(|ids| ids.contains(&token_id.as_str()))
        || image.chain.as_denom().is_some_and(|denom| denom.eq_ignore_ascii_case(&image.token_id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_native_token_placeholders_are_skipped() {
        assert!(is_native_token(&asset_image(Chain::Ethereum, "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE")));
        assert!(is_native_token(&asset_image(Chain::Polygon, "0x0000000000000000000000000000000000001010")));
        assert!(is_native_token(&asset_image(Chain::ZkSync, "0x000000000000000000000000000000000000800A")));
    }

    fn asset_image(chain: Chain, token_id: &str) -> AssetImage {
        AssetImage {
            chain,
            token_id: token_id.to_string(),
            image_url: String::new(),
        }
    }
}
