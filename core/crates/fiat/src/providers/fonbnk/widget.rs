use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};

use jsonwebtoken::{EncodingKey, Header, encode};
use primitives::{FiatQuoteType, FiatQuoteUrlData};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::model::FiatMapping;

const SIGNATURE_LIFETIME_SECONDS: u64 = 3600;
const OFF_RAMP_PATH: &str = "/offramp";

#[derive(Debug, Serialize, Deserialize)]
struct FonbnkWidgetClaims {
    uid: String,
    exp: u64,
}

pub struct FonbnkWidget {
    url: String,
    source: String,
    secret: String,
}

impl FonbnkWidget {
    pub fn new(url: String, source: String, secret: String) -> Self {
        Self { url, source, secret }
    }

    pub fn redirect_url(&self, data: &FiatQuoteUrlData) -> Result<String, Box<dyn Error + Send + Sync>> {
        let expires_at = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() + SIGNATURE_LIFETIME_SECONDS;
        self.signed_url(data, expires_at)
    }

    fn signed_url(&self, data: &FiatQuoteUrlData, expires_at: u64) -> Result<String, Box<dyn Error + Send + Sync>> {
        let network = FiatMapping::get_network(data.asset_symbol.network.clone())?;
        let claims = FonbnkWidgetClaims { uid: data.quote.id.clone(), exp: expires_at };
        let signature = encode(&Header::default(), &claims, &EncodingKey::from_secret(self.secret.as_bytes()))?;

        let mut url = Url::parse(&self.url)?;
        if data.quote.quote_type == FiatQuoteType::Sell {
            url.set_path(OFF_RAMP_PATH);
        }
        url.query_pairs_mut()
            .append_pair("source", &self.source)
            .append_pair("signature", &signature)
            .append_pair("network", &network)
            .append_pair("asset", &data.asset_symbol.symbol)
            .append_pair("amount", &data.quote.crypto_amount.to_string())
            .append_pair("currency", "crypto")
            .append_pair("countryIsoCode", &data.country_code)
            .append_pair("orderParams", &data.quote.id)
            .append_pair("hideSwitch", "true");
        match data.quote.quote_type {
            FiatQuoteType::Buy => {
                url.query_pairs_mut().append_pair("address", &data.wallet_address).append_pair("freezeWallet", "true");
            }
            FiatQuoteType::Sell => {}
        }
        Ok(url.into())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
    use primitives::{FiatAssetSymbol, FiatProviderName, FiatQuote};

    use super::*;

    fn data(quote_type: FiatQuoteType) -> FiatQuoteUrlData {
        FiatQuoteUrlData {
            quote: FiatQuote {
                id: "quote_id".to_string(),
                quote_type,
                crypto_amount: 48.748907,
                ..FiatQuote::mock(FiatProviderName::Fonbnk)
            },
            asset_symbol: FiatAssetSymbol {
                symbol: "USDC".to_string(),
                network: Some("BASE".to_string()),
            },
            wallet_address: "0x41018795fA95783117242244303fd7e26e964eE8".to_string(),
            ip_address: "192.0.2.1".to_string(),
            country_code: "NG".to_string(),
            locale: "en".to_string(),
        }
    }

    fn widget() -> FonbnkWidget {
        FonbnkWidget::new("https://pay.fonbnk.com".to_string(), "test_source".to_string(), "test_widget_secret".to_string())
    }

    #[test]
    fn test_signed_url_buy() {
        let url = Url::parse(&widget().signed_url(&data(FiatQuoteType::Buy), 1_800_000_000).unwrap()).unwrap();
        let params: HashMap<String, String> = url.query_pairs().into_owned().collect();
        let claims = decode::<FonbnkWidgetClaims>(&params["signature"], &DecodingKey::from_secret(b"test_widget_secret"), &Validation::new(Algorithm::HS256))
            .unwrap()
            .claims;

        assert_eq!(url.path(), "/");
        assert_eq!(claims.uid, "quote_id");
        assert_eq!(claims.exp, 1_800_000_000);
        assert_eq!(params["source"], "test_source");
        assert_eq!(params["network"], "BASE");
        assert_eq!(params["asset"], "USDC");
        assert_eq!(params["amount"], "48.748907");
        assert_eq!(params["currency"], "crypto");
        assert_eq!(params["countryIsoCode"], "NG");
        assert_eq!(params["orderParams"], "quote_id");
        assert_eq!(params["address"], "0x41018795fA95783117242244303fd7e26e964eE8");
        assert_eq!(params["freezeWallet"], "true");
    }

    #[test]
    fn test_signed_url_sell() {
        let url = Url::parse(&widget().signed_url(&data(FiatQuoteType::Sell), 1_800_000_000).unwrap()).unwrap();
        let params: HashMap<String, String> = url.query_pairs().into_owned().collect();

        assert_eq!(url.path(), "/offramp");
        assert_eq!(params["orderParams"], "quote_id");
        assert!(!params.contains_key("address"));
    }
}
