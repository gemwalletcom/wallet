use coinmarketcap::{Info, get_chain_for_coinmarketcap_platform, get_coinmarketcap_logo_url};

use crate::providers::{mapper::is_native_token, model::AssetImage};

pub(super) fn map_info(info: Info) -> Vec<AssetImage> {
    if info.logo.is_empty() || !info.is_token() {
        return vec![];
    }

    let Some(image_url) = get_coinmarketcap_logo_url(&info.logo) else {
        return vec![];
    };

    info.contract_address
        .into_iter()
        .filter_map(|contract| {
            let chain = get_chain_for_coinmarketcap_platform(&contract.platform)?;
            let image = AssetImage {
                chain,
                token_id: contract.contract_address,
                image_url: image_url.clone(),
            };
            (!is_native_token(&image)).then_some(image)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use coinmarketcap::{ContractAddress, Platform, PlatformCoin};

    #[test]
    fn test_map_info_skips_native_placeholders() {
        let token = Info {
            logo: "https://s2.coinmarketcap.com/static/img/coins/256x256/825.png".to_string(),
            platform: Some(Default::default()),
            contract_address: vec![
                contract("Ethereum", "ethereum", "0x0000000000000000000000000000000000000000"),
                contract("Ethereum", "ethereum", "0xdAC17F958D2ee523a2206206994597C13D831ec7"),
            ],
        };
        let images = map_info(token);

        assert_eq!(images.len(), 1);
        assert_eq!(images[0].token_id, "0xdAC17F958D2ee523a2206206994597C13D831ec7");
    }

    fn contract(name: &str, slug: &str, contract_address: &str) -> ContractAddress {
        ContractAddress {
            contract_address: contract_address.to_string(),
            platform: Platform {
                name: name.to_string(),
                coin: PlatformCoin { slug: slug.to_string() },
            },
        }
    }
}
