use super::MayanClient;
use crate::{
    SwapperError,
    mayan::{
        constants::{MAYAN_FORWARDER, MAYAN_PROGRAM_ID, SDK_VERSION},
        model::{ErrorResponse, MayanQuote, QuoteParams, QuoteResponse},
    },
};
use gem_client::{Client, ClientExt};
use primitives::swap::SlippageMode;
use serde::Serialize;
use std::fmt::Debug;

const QUOTE_DEFAULTS: [(&str, &str); 11] = [
    ("wormhole", "false"),
    ("swift", "true"),
    ("mctp", "true"),
    ("shuttle", "false"),
    ("fastMctp", "true"),
    ("gasless", "false"),
    ("onlyDirect", "false"),
    ("fullList", "false"),
    ("monoChain", "true"),
    ("solanaProgram", MAYAN_PROGRAM_ID),
    ("forwarderAddress", MAYAN_FORWARDER),
];

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct QuoteDynamicQuery {
    amount_in64: String,
    from_token: String,
    from_chain: String,
    to_token: String,
    to_chain: String,
    referrer: String,
    referrer_bps: u32,
    sdk_version: &'static str,
    slippage_bps: String,
}

impl From<QuoteParams> for QuoteDynamicQuery {
    fn from(params: QuoteParams) -> Self {
        let slippage_bps = match params.slippage_mode {
            SlippageMode::Auto => "auto".to_string(),
            SlippageMode::Exact => params.slippage_bps.to_string(),
        };
        Self {
            amount_in64: params.amount_in64,
            from_token: params.from_token,
            from_chain: params.from_chain,
            to_token: params.to_token,
            to_chain: params.to_chain,
            referrer: params.referrer,
            referrer_bps: params.referrer_bps,
            sdk_version: SDK_VERSION,
            slippage_bps,
        }
    }
}

impl<C> MayanClient<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    pub async fn get_quotes(&self, params: QuoteParams) -> Result<Vec<MayanQuote>, SwapperError> {
        let path = quote_path(params)?;
        let response = self.client.get_or_error::<QuoteResponse, ErrorResponse>(&path).await.map_err(SwapperError::from)?;
        Ok(response.quotes)
    }
}

fn quote_path(params: QuoteParams) -> Result<String, SwapperError> {
    let defaults = serde_urlencoded::to_string(QUOTE_DEFAULTS)?;
    let query = serde_urlencoded::to_string(QuoteDynamicQuery::from(params))?;
    Ok(format!("/quote?{defaults}&{query}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quote_path() {
        let path = quote_path(QuoteParams {
            amount_in64: "1000000".to_string(),
            from_token: "0x0000000000000000000000000000000000000000".to_string(),
            from_chain: "ethereum".to_string(),
            to_token: "So11111111111111111111111111111111111111112".to_string(),
            to_chain: "solana".to_string(),
            referrer: "0x1111111111111111111111111111111111111111".to_string(),
            referrer_bps: 50,
            slippage_bps: 100,
            slippage_mode: SlippageMode::Auto,
        })
        .unwrap();

        assert_eq!(
            path,
            "/quote?wormhole=false&swift=true&mctp=true&shuttle=false&fastMctp=true&gasless=false&onlyDirect=false&fullList=false&monoChain=true&solanaProgram=FC4eXxkyrMPTjiYUpp4EAnkmwMbQyZ6NDCh1kfLn6vsf&forwarderAddress=0x337685fdaB40D39bd02028545a4FfA7D287cC3E2&amountIn64=1000000&fromToken=0x0000000000000000000000000000000000000000&fromChain=ethereum&toToken=So11111111111111111111111111111111111111112&toChain=solana&referrer=0x1111111111111111111111111111111111111111&referrerBps=50&sdkVersion=14_1_0&slippageBps=auto"
        );
    }
}
