use std::env;
use std::path::PathBuf;

use config::{Config, ConfigError, File};

use super::EgressConfig;

impl EgressConfig {
    pub(crate) fn load() -> Result<Self, ConfigError> {
        let path = if let Some(path) = env::var_os("EGRESS_CONFIG") {
            PathBuf::from(path)
        } else {
            let current = env::current_dir().map_err(|error| ConfigError::Message(format!("current directory is unavailable: {error}")))?;
            if current.join("config.yml").exists() {
                current.join("config.yml")
            } else {
                current.join("apps/egress/config.yml")
            }
        };
        let mut config = Config::builder().add_source(File::from(path)).build()?.try_deserialize::<Self>()?;
        config.expand_environment()?;
        Ok(config)
    }

    fn expand_environment(&mut self) -> Result<(), ConfigError> {
        for caller in self.callers.values_mut() {
            caller.key = expand_value(&caller.key, |name| env::var(name).ok())?;
        }
        for services in self.routes.values_mut() {
            for route in services.values_mut() {
                for endpoint in &mut route.endpoints {
                    endpoint.url = expand_value(&endpoint.url, |name| env::var(name).ok())?;
                    for values in [&mut endpoint.headers, &mut endpoint.query].into_iter().flatten() {
                        for value in values.values_mut() {
                            *value = expand_value(value, |name| env::var(name).ok())?;
                        }
                    }
                }
            }
        }
        Ok(())
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_value() {
        let value = expand_value("Bearer ${TOKEN}", |name| (name == "TOKEN").then(|| "secret".to_string())).unwrap();
        assert_eq!(value, "Bearer secret");
    }
}
