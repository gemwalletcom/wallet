use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct MetricsConfig {
    pub prefix: String,
    pub source: String,
}
