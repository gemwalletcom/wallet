use std::env;
use std::path::PathBuf;

use config::{Config, ConfigError, File};

use super::EgressConfig;

impl EgressConfig {
    pub(crate) fn load() -> Result<Self, ConfigError> {
        let path = if let Some(path) = env::var_os("EGRESS_CONFIG") {
            PathBuf::from(path)
        } else {
            let current = env::current_dir().map_err(|error| message(format!("current directory is unavailable: {error}")))?;
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
        for route in &mut self.routes {
            for endpoint in &mut route.endpoints {
                if let Some(headers) = &mut endpoint.headers {
                    for value in headers.values_mut() {
                        *value = expand_value(value, |name| env::var(name).ok())?;
                    }
                }
                if let Some(query) = &mut endpoint.query {
                    for value in query.values_mut() {
                        *value = expand_value(value, |name| env::var(name).ok())?;
                    }
                }
                if let Some(suffix) = &mut endpoint.suffix {
                    *suffix = expand_value(suffix, |name| env::var(name).ok())?;
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
            .ok_or_else(|| message("unterminated environment variable"))?;
        let name = &expanded[name_start..end];
        if name.is_empty() {
            return Err(message("empty environment variable"));
        }
        let replacement = get(name).ok_or_else(|| message(format!("missing environment variable: {name}")))?;
        expanded.replace_range(start..=end, &replacement);
        offset = start + replacement.len();
    }
    Ok(expanded)
}

fn message(value: impl Into<String>) -> ConfigError {
    ConfigError::Message(value.into())
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
