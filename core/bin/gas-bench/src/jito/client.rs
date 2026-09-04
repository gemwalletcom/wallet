use super::{JitoTipFloor, JitoTipFloorEntry};
use gem_client::{ClientExt, ReqwestClient, Target};
use std::error::Error;

const JITO_URL: &str = "https://bundles.jito.wtf";

#[derive(Clone, Debug)]
enum JitoTarget {
    TipFloor,
}

impl Target for JitoTarget {
    fn path(&self) -> String {
        match self {
            Self::TipFloor => "/api/v1/bundles/tip_floor".to_string(),
        }
    }
}

pub struct JitoClient {
    client: ReqwestClient,
}

impl Default for JitoClient {
    fn default() -> Self {
        Self::new()
    }
}

impl JitoClient {
    pub fn new() -> Self {
        Self {
            client: ReqwestClient::new(JITO_URL.to_string(), gem_client::reqwest_client()),
        }
    }

    pub async fn fetch_tip_floor(&self) -> Result<JitoTipFloor, Box<dyn Error + Send + Sync>> {
        let entries: Vec<JitoTipFloorEntry> = self.client.get(JitoTarget::TipFloor).await?;
        let entry = entries.first().ok_or("No tip floor data returned from Jito API")?;
        Ok(JitoTipFloor::from_entry(entry))
    }
}
