use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{env, fs};

use config::{Config as FileConfig, ConfigError, Environment, File};
use primitives::Chain;
use reqwest::header::{HeaderName, HeaderValue};
use url::Url;

use super::{ChainConfig, Config, HeadersConfig};

fn configuration_path() -> Result<PathBuf, ConfigError> {
    let current = env::current_dir().map_err(|error| ConfigError::Message(format!("current directory is unavailable: {error}")))?;
    if let Some(path) = env::var_os("DYNODE_CONFIG").or_else(|| env::var_os("EGRESS_CONFIG")) {
        return Ok(current.join(path));
    }
    Ok(if current.join("config.yml").exists() {
        current.join("config.yml")
    } else {
        current.join("apps/dynode/config.yml")
    })
}

pub fn load_config() -> Result<(Config, HashMap<Chain, ChainConfig>), ConfigError> {
    let config_path = configuration_path()?;
    let (mut config, mut chains) = load_files(&config_path, Environment::default().separator("_"))?;
    prepare(&mut config, &mut chains, |name| env::var(name).ok())?;
    Ok((config, chains))
}

fn load_files(config_path: &Path, environment: Environment) -> Result<(Config, HashMap<Chain, ChainConfig>), ConfigError> {
    let base_dir = config_path.parent().ok_or_else(|| ConfigError::Message("configuration directory is unavailable".into()))?;
    let mut config: Config = FileConfig::builder()
        .add_source(File::from(config_path))
        .add_source(environment.clone())
        .build()?
        .try_deserialize()?;
    let mut chains = HashMap::new();
    let chain_files = find_chain_files(base_dir)?;
    if !chain_files.is_empty() {
        let mut builder = FileConfig::builder();
        for path in chain_files {
            let file = FileConfig::builder().add_source(File::from(path)).build()?;
            let overrides = file.get::<Vec<ChainConfig>>("chains")?;
            chains.extend(overrides.into_iter().map(|chain| (chain.chain, chain)));
            builder = builder.add_source(file);
        }
        config.chains = Some(builder.add_source(environment.clone()).build()?.try_deserialize()?);
    }
    let routes_path = base_dir.join("routes.yml");
    if routes_path.exists() {
        config.routes = Some(
            FileConfig::builder()
                .add_source(File::from(routes_path))
                .add_source(environment)
                .build()?
                .try_deserialize()?,
        );
    }
    Ok((config, chains))
}

fn prepare(config: &mut Config, chains: &mut HashMap<Chain, ChainConfig>, mut get: impl FnMut(&str) -> Option<String>) -> Result<(), ConfigError> {
    if let Some(config) = &mut config.chains {
        validate_forward_headers(&config.headers)?;
        if config.monitoring.enabled && config.monitoring.interval.is_zero() {
            return Err(ConfigError::Message("monitoring interval must be greater than zero".into()));
        }
        if let Some(webhook) = &mut config.webhook {
            webhook.url = expand_value(&webhook.url, &mut get)?;
            webhook.token = expand_value(&webhook.token, &mut get)?;
            if webhook.enabled {
                validate_url(&webhook.url)?;
            }
        }
    }
    if let Some(config) = &mut config.routes {
        for proxy in config.proxies.iter_mut().flat_map(|proxies| proxies.values_mut()) {
            proxy.url = expand_value(&proxy.url, &mut get)?;
            proxy.health.url = expand_value(&proxy.health.url, &mut get)?;
            validate_url(&proxy.health.url)?;
            if proxy.health.interval.is_zero() {
                return Err(ConfigError::Message("proxy health interval must be greater than zero".into()));
            }
        }
        for route in config.routes.values_mut() {
            if route.endpoints.is_empty() {
                return Err(ConfigError::Message("route must configure at least one endpoint".into()));
            }
            if route.rate.is_some_and(|rate| (rate.period / rate.requests.get()).is_zero()) {
                return Err(ConfigError::Message("rate period per request must be greater than zero".into()));
            }
            for endpoint in &mut route.endpoints {
                endpoint.url = expand_value(&endpoint.url, &mut get)?;
                validate_url(&endpoint.url)?;
                expand_headers(&mut endpoint.headers, &mut get)?;
                for value in endpoint.query.iter_mut().flat_map(|query| query.values_mut()) {
                    *value = expand_value(value, &mut get)?;
                }
            }
        }
    }
    for chain in chains.values_mut() {
        if chain.urls.is_empty() {
            return Err(ConfigError::Message(format!("chain {} must configure at least one URL", chain.chain)));
        }
        for url in &mut chain.urls {
            url.url = expand_value(&url.url, &mut get)?;
            validate_url(&url.url)?;
            expand_headers(&mut url.headers, &mut get)?;
        }
        for configuration in chain.overrides.iter_mut().flatten() {
            configuration.url = expand_value(&configuration.url, &mut get)?;
            validate_url(&configuration.url)?;
        }
    }
    Ok(())
}

fn validate_url(value: &str) -> Result<(), ConfigError> {
    let url = Url::parse(value).map_err(|_| ConfigError::Message("invalid upstream URL".into()))?;
    if !["http", "https"].contains(&url.scheme()) || url.host_str().is_none() {
        return Err(ConfigError::Message("upstream URL must use HTTP(S) and include a host".into()));
    }
    Ok(())
}

fn validate_forward_headers(headers: &HeadersConfig) -> Result<(), ConfigError> {
    for name in &headers.forward {
        HeaderName::from_bytes(name.as_bytes()).map_err(|_| ConfigError::Message("invalid forwarding header name".into()))?;
    }
    Ok(())
}

fn expand_headers(headers: &mut Option<HashMap<String, String>>, get: &mut impl FnMut(&str) -> Option<String>) -> Result<(), ConfigError> {
    for (name, value) in headers.iter_mut().flat_map(|headers| headers.iter_mut()) {
        HeaderName::from_bytes(name.as_bytes()).map_err(|_| ConfigError::Message("invalid endpoint header name".into()))?;
        *value = expand_value(value, &mut *get)?;
        HeaderValue::from_str(value).map_err(|_| ConfigError::Message("invalid endpoint header value".into()))?;
    }
    Ok(())
}

fn expand_value(value: &str, mut get: impl FnMut(&str) -> Option<String>) -> Result<String, ConfigError> {
    let mut expanded = value.to_string();
    let mut offset = 0;
    while let Some(start) = expanded[offset..].find("${").map(|start| offset + start) {
        let name_start = start + 2;
        let end = expanded[name_start..]
            .find('}')
            .map(|end| name_start + end)
            .ok_or_else(|| ConfigError::Message("unterminated environment variable".into()))?;
        let name = &expanded[name_start..end];
        let replacement = get(name).ok_or_else(|| ConfigError::Message(format!("missing environment variable: {name}")))?;
        expanded.replace_range(start..=end, &replacement);
        offset = start + replacement.len();
    }
    Ok(expanded)
}

fn find_chain_files(base_dir: &Path) -> Result<Vec<PathBuf>, ConfigError> {
    let mut files: Vec<PathBuf> = fs::read_dir(base_dir)
        .map_err(|error| ConfigError::Message(format!("cannot read chain configuration directory: {error}")))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| ConfigError::Message(format!("cannot read chain configuration entry: {error}")))?
        .into_iter()
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("chains") && name.ends_with(".yml"))
        })
        .collect();
    files.sort();
    Ok(files)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use config::FileFormat;
    use uuid::Uuid;

    use super::*;
    use crate::config::Override;
    use crate::config::routes::{ProxyConfig, ProxyHealthConfig};
    use crate::testkit::config::chain_config;

    fn sample_config() -> Config {
        let mut config: Config = FileConfig::builder()
            .add_source(File::from_str(include_str!("../../config.yml"), FileFormat::Yaml))
            .build()
            .unwrap()
            .try_deserialize()
            .unwrap();
        config.chains = Some(
            FileConfig::builder()
                .add_source(File::from_str(include_str!("../../chains.yml"), FileFormat::Yaml))
                .build()
                .unwrap()
                .try_deserialize()
                .unwrap(),
        );
        config.routes = Some(
            FileConfig::builder()
                .add_source(File::from_str(include_str!("../../routes.yml"), FileFormat::Yaml))
                .build()
                .unwrap()
                .try_deserialize()
                .unwrap(),
        );
        config
    }

    #[test]
    fn test_expand_value_preserves_literal_environment_contents() {
        assert_eq!(
            expand_value("Bearer ${TOKEN}", |_| Some("secret${LITERAL}".to_string())).unwrap(),
            "Bearer secret${LITERAL}"
        );
        assert_eq!(expand_value("${FIRST}/${SECOND}", |name| Some(name.to_lowercase())).unwrap(), "first/second");
        assert_eq!(expand_value("${MISSING}", |_| None).unwrap_err().to_string(), "missing environment variable: MISSING");
        assert_eq!(expand_value("${MISSING", |_| None).unwrap_err().to_string(), "unterminated environment variable");
    }

    #[test]
    fn test_prepare_expands_credentials_for_both_route_types() {
        let mut config = sample_config();
        let endpoint = &mut config.routes.as_mut().unwrap().routes.get_mut("fastnear_tx").unwrap().endpoints[0];
        endpoint.url = "https://${HOST}/provider".to_string();
        endpoint.headers = Some(HashMap::from([("authorization".to_string(), "Bearer ${KEY}".to_string())]));
        endpoint.query = Some(HashMap::from([("key".to_string(), "${KEY}".to_string())]));
        endpoint.proxy = Some("outbound".to_string());
        config.routes.as_mut().unwrap().proxies = Some(HashMap::from([(
            "outbound".to_string(),
            ProxyConfig {
                url: "http://${HOST}:8080".to_string(),
                health: ProxyHealthConfig {
                    url: "https://${HOST}/health".to_string(),
                    interval: Duration::from_secs(10),
                },
            },
        )]));
        let mut chain = chain_config(Chain::Ethereum, "https://${HOST}/rpc");
        chain.urls[0].headers = Some(HashMap::from([("x-api-key".to_string(), "${KEY}".to_string())]));
        chain.overrides = Some(vec![Override {
            rpc_method: Some("eth_chainId".to_string()),
            path: None,
            url: "https://${HOST}/override".to_string(),
        }]);
        let mut chains = HashMap::from([(Chain::Ethereum, chain)]);
        let values = HashMap::from([("KEY", "test-key"), ("HOST", "example.invalid")]);
        prepare(&mut config, &mut chains, |name| values.get(name).map(|value| value.to_string())).unwrap();

        let endpoint = &config.routes.as_ref().unwrap().routes["fastnear_tx"].endpoints[0];
        assert_eq!(endpoint.url, "https://example.invalid/provider");
        assert_eq!(endpoint.headers.as_ref().unwrap()["authorization"], "Bearer test-key");
        assert_eq!(endpoint.query.as_ref().unwrap()["key"], "test-key");
        assert_eq!(config.routes.as_ref().unwrap().proxies.as_ref().unwrap()["outbound"].url, "http://example.invalid:8080");
        assert_eq!(
            config.routes.as_ref().unwrap().proxies.as_ref().unwrap()["outbound"].health.url,
            "https://example.invalid/health"
        );
        assert_eq!(chains[&Chain::Ethereum].urls[0].url, "https://example.invalid/rpc");
        assert_eq!(chains[&Chain::Ethereum].urls[0].headers.as_ref().unwrap()["x-api-key"], "test-key");
        assert_eq!(chains[&Chain::Ethereum].overrides.as_ref().unwrap()[0].url, "https://example.invalid/override");
    }

    #[test]
    fn test_prepare_rejects_invalid_configuration_without_exposing_credentials() {
        let mut config = sample_config();
        let mut chains = HashMap::new();
        config.routes.as_mut().unwrap().routes.get_mut("fastnear_tx").unwrap().endpoints[0].url = "invalid-test-secret".to_string();
        assert_eq!(prepare(&mut config, &mut chains, |_| None).unwrap_err().to_string(), "invalid upstream URL");
        config.routes.as_mut().unwrap().routes.get_mut("fastnear_tx").unwrap().endpoints.clear();
        assert_eq!(
            prepare(&mut config, &mut chains, |_| None).unwrap_err().to_string(),
            "route must configure at least one endpoint"
        );
    }

    #[test]
    fn test_load_files_preserves_region_replacement_and_environment_precedence() {
        let directory = env::temp_dir().join(format!("dynode-config-{}", Uuid::new_v4()));
        fs::create_dir(&directory).unwrap();
        fs::write(directory.join("config.yml"), include_str!("../../config.yml")).unwrap();
        fs::write(directory.join("chains.yml"), include_str!("../../chains.yml")).unwrap();
        fs::write(directory.join("routes.yml"), include_str!("../../routes.yml")).unwrap();
        fs::write(
            directory.join("chains_region.yml"),
            "request:\n  timeout: 7s\ncache:\n  memory:\n    max: 1 MiB\nchains:\n  - chain: ethereum\n    urls:\n      - url: https://region.example.invalid\n",
        )
        .unwrap();
        let source = Environment::default().separator("_").source(Some(HashMap::from([
            ("REQUEST_TIMEOUT".to_string(), "9s".to_string()),
            ("PORT".to_string(), "8080".to_string()),
            ("METRICS_SOURCE".to_string(), "parser".to_string()),
            ("CHAINS_CARDANO_URL".to_string(), "https://credential.example.invalid".to_string()),
        ])));
        let (config, chains) = load_files(&directory.join("config.yml"), source).unwrap();
        let chain_settings = config.chains.as_ref().unwrap();
        let route_settings = config.routes.as_ref().unwrap();
        assert_eq!(config.port, 8080);
        assert_eq!(config.metrics.source, "parser");
        assert_eq!(chain_settings.request.timeout, Duration::from_secs(9));
        assert_eq!(route_settings.request.timeout, Duration::from_secs(9));
        assert_eq!(chain_settings.request.limit, 32 * 1024 * 1024);
        assert_eq!(chain_settings.cache.memory.max, 1024 * 1024);
        assert_eq!(route_settings.cache.memory.max, 64_000_000);
        assert_eq!(route_settings.routes.len(), 2);
        assert_eq!(route_settings.routes["fastnear_tx"].group, "indexer");
        let expected_chains = FileConfig::builder()
            .add_source(File::from_str(include_str!("../../chains.yml"), FileFormat::Yaml))
            .build()
            .unwrap()
            .get::<Vec<ChainConfig>>("chains")
            .unwrap();
        assert_eq!(chains.len(), expected_chains.len());
        assert_eq!(chains[&Chain::Ethereum].urls, chain_config(Chain::Ethereum, "https://region.example.invalid").urls);
        assert_eq!(chains[&Chain::Ethereum].overrides, None);
        for chain in expected_chains.iter().filter(|chain| chain.chain != Chain::Ethereum) {
            assert_eq!(chains[&chain.chain].urls, chain.urls);
        }
        let source = Environment::default()
            .separator("_")
            .source(Some(HashMap::from([("CACHE_MEMORY_MAX".to_string(), "2 MiB".to_string())])));
        let (config, _) = load_files(&directory.join("config.yml"), source).unwrap();
        assert_eq!(config.chains.unwrap().cache.memory.max, 2 * 1024 * 1024);
        assert_eq!(config.routes.unwrap().cache.memory.max, 2 * 1024 * 1024);
        fs::write(directory.join("chains_region.yml"), "chain: []").unwrap();
        assert!(load_files(&directory.join("config.yml"), Environment::default().source(Some(HashMap::new()))).is_err());
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn test_load_files_configures_only_present_families_and_ignores_removed_cache_file() {
        let directory = env::temp_dir().join(format!("dynode-config-{}", Uuid::new_v4()));
        fs::create_dir(&directory).unwrap();
        fs::write(directory.join("config.yml"), include_str!("../../config.yml")).unwrap();
        fs::write(directory.join("cache.yml"), "invalid retired cache content: [").unwrap();
        let (config, chains) = load_files(&directory.join("config.yml"), Environment::default().source(Some(HashMap::new()))).unwrap();
        assert!(config.chains.is_none());
        assert!(config.routes.is_none());
        assert!(chains.is_empty());
        let mut config = config;
        prepare(&mut config, &mut HashMap::new(), |_| None).unwrap();

        fs::write(directory.join("routes.yml"), include_str!("../../routes.yml")).unwrap();
        let (config, chains) = load_files(&directory.join("config.yml"), Environment::default().source(Some(HashMap::new()))).unwrap();
        assert!(config.chains.is_none());
        assert!(chains.is_empty());
        assert_eq!(config.routes.as_ref().unwrap().cache.memory.max, 64_000_000);
        assert_eq!(config.routes.as_ref().unwrap().routes.len(), 2);

        fs::remove_file(directory.join("routes.yml")).unwrap();
        fs::write(directory.join("chains.yml"), include_str!("../../chains.yml")).unwrap();
        let (config, chains) = load_files(&directory.join("config.yml"), Environment::default().source(Some(HashMap::new()))).unwrap();
        assert!(config.chains.is_some());
        assert!(config.routes.is_none());
        assert!(!chains.is_empty());
        assert_eq!(config.chains.as_ref().unwrap().cache.memory.max, 2_000_000_000);
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn test_route_inventory_requires_routes_map_and_group() {
        let directory = env::temp_dir().join(format!("dynode-config-{}", Uuid::new_v4()));
        fs::create_dir(&directory).unwrap();
        fs::write(directory.join("config.yml"), include_str!("../../config.yml")).unwrap();
        for routes in [
            include_str!("../../routes.yml").replace("routes:", "route:"),
            include_str!("../../routes.yml").replace("    group: indexer\n", ""),
        ] {
            fs::write(directory.join("routes.yml"), routes).unwrap();
            assert!(load_files(&directory.join("config.yml"), Environment::default().source(Some(HashMap::new()))).is_err());
        }
        fs::remove_dir_all(directory).unwrap();
    }
}
