use super::MayanClient;
use crate::{
    SwapperError,
    mayan::{
        model::{ErrorResponse, MayanQuote, QuoteParams, QuoteQuery, QuoteResponse},
        target::MayanTarget,
    },
};
use gem_client::{Client, ClientExt};
use std::fmt::Debug;

impl<C> MayanClient<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    pub async fn get_quotes(&self, params: QuoteParams) -> Result<Vec<MayanQuote>, SwapperError> {
        let response = self
            .client
            .get_or_error::<QuoteResponse, ErrorResponse>(MayanTarget::Quote { query: QuoteQuery::from(params) })
            .await
            .map_err(SwapperError::from)?;
        Ok(response.quotes)
    }
}
