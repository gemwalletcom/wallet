use std::{error::Error, str::FromStr};

use chain_traits::ChainTraits;
use gem_client::builder;
use primitives::{Chain, NodeType};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use settings_chain::{ProviderConfig, ProviderFactory, ProviderKeyConfig};

pub(crate) fn new_provider(chain: Chain, node: NodeType, url: &str, headers: &[String]) -> Result<Box<dyn ChainTraits>, Box<dyn Error + Send + Sync>> {
    let headers = headers
        .iter()
        .map(|header| {
            let (name, value) = header.split_once(':').ok_or_else(|| format!("invalid header: {header}"))?;
            Ok((HeaderName::from_str(name.trim())?, HeaderValue::from_str(value.trim())?))
        })
        .collect::<Result<HeaderMap, Box<dyn Error + Send + Sync>>>()?;
    let client = builder().default_headers(headers).build()?;
    let config = ProviderConfig::new(chain, url, node, ProviderKeyConfig::default());
    Ok(ProviderFactory::new_provider_with_client(config, "nodecheck", client))
}
