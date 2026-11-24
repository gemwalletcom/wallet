use primitives::FiatQuoteType;
use primitives::hex;
use sha2::{Digest, Sha512};
use url::Url;

const MERCURYO_REDIRECT_URL: &str = "https://exchange.mercuryo.io";

pub struct MercuryoWidget {
    widget_id: String,
    secret_key: String,
    merchant_transaction_id: String,
    address: String,
    ip_address: String,
    currency: String,
    network: String,
    quote_type: FiatQuoteType,
    amount: f64,
}

impl MercuryoWidget {
    pub fn new(widget_id: String, secret_key: String, merchant_transaction_id: String, address: String, ip_address: String, currency: String, amount: f64, quote_type: FiatQuoteType, network: String) -> Self {
        Self {
            widget_id,
            secret_key,
            merchant_transaction_id,
            address,
            ip_address,
            currency,
            network,
            quote_type,
            amount,
        }
    }

    fn signature(&self) -> String {
        let content = format!("{}{}{}{}", self.address, self.secret_key, self.ip_address, self.merchant_transaction_id);
        let hash = hex::encode(Sha512::digest(content));
        format!("v2:{}", hash)
    }

    pub fn to_url(&self) -> String {
        let mut url = Url::parse(MERCURYO_REDIRECT_URL).unwrap();

        url.query_pairs_mut()
            .append_pair("widget_id", &self.widget_id)
            .append_pair("merchant_transaction_id", &self.merchant_transaction_id)
            .append_pair("currency", &self.currency)
            .append_pair("address", &self.address)
            .append_pair("network", &self.network)
            .append_pair("signature", &self.signature());

        match self.quote_type {
            FiatQuoteType::Buy => {
                url.query_pairs_mut().append_pair("type", "buy").append_pair("fiat_amount", &self.amount.to_string());
            }
            FiatQuoteType::Sell => {
                url.query_pairs_mut().append_pair("type", "sell").append_pair("amount", &self.amount.to_string());
            }
        };

        url.as_str().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signature_v2_format() {
        let signature = MercuryoWidget::mock().signature();
        let expected_hash = hex::encode(Sha512::digest("0x123secret127.0.0.1tx123"));

        assert_eq!(signature, format!("v2:{}", expected_hash));
    }

    #[test]
    fn build_url_buy() {
        let url = MercuryoWidget::mock().to_url();

        assert_eq!(
            url,
            "https://exchange.mercuryo.io/?widget_id=widget123&merchant_transaction_id=tx123&currency=BTC&address=0x123&network=BITCOIN&signature=v2%3Aead820318be19870c157682e7f4ad1190b41b3811d85c1a31c14b4d614fed9a7806fdddd324e258d7c7d0be78427727d98bed6d44a5a13c3761aabf5b39978e7&type=buy&fiat_amount=1000"
        );
    }

    #[test]
    fn build_url_sell() {
        let url = MercuryoWidget {
            currency: "ETH".to_string(),
            amount: 1.5,
            quote_type: FiatQuoteType::Sell,
            ..MercuryoWidget::mock()
        }
        .to_url();

        assert_eq!(
            url,
            "https://exchange.mercuryo.io/?widget_id=widget123&merchant_transaction_id=tx123&currency=ETH&address=0x123&network=BITCOIN&signature=v2%3Aead820318be19870c157682e7f4ad1190b41b3811d85c1a31c14b4d614fed9a7806fdddd324e258d7c7d0be78427727d98bed6d44a5a13c3761aabf5b39978e7&type=sell&amount=1.5"
        );
    }
}
