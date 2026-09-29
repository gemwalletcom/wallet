use std::fmt::Debug;

use async_trait::async_trait;
use chrono::{Duration, Utc};
use gem_client::Client;
use primitives::SwapProvider;

use super::{
    client::RelayClient,
    mapper::map_partner_transaction,
    model::{RelayPartnerCursor, RelayPartnerRequestsQuery, RelayPartnerRequestsResponse},
};
use crate::{
    SwapperError,
    fees::DEFAULT_REFERRER,
    partner::{SwapPartnerCursor, SwapPartnerProvider, SwapPartnerTransactionsPage},
};

const REQUESTS_LIMIT: u32 = 50;
const LATEST_LOOKBACK: Duration = Duration::days(1);

#[derive(Debug)]
pub struct RelayPartnerProvider<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    client: RelayClient<C>,
}

impl<C> RelayPartnerProvider<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    pub fn new(client: C) -> Self {
        Self { client: RelayClient::new(client) }
    }
}

fn map_next_cursor(cursor: RelayPartnerCursor, response: &RelayPartnerRequestsResponse, requested_at: i64) -> Result<SwapPartnerCursor, SwapperError> {
    match &response.continuation {
        Some(continuation) => Ok(SwapPartnerCursor::Next(serde_json::to_string(&RelayPartnerCursor {
            start_timestamp: cursor.start_timestamp,
            continuation: Some(continuation.clone()),
        })?)),
        None => Ok(SwapPartnerCursor::Latest(serde_json::to_string(&RelayPartnerCursor {
            start_timestamp: Some(requested_at - LATEST_LOOKBACK.num_seconds()),
            continuation: None,
        })?)),
    }
}

#[async_trait]
impl<C> SwapPartnerProvider for RelayPartnerProvider<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    fn provider(&self) -> SwapProvider {
        SwapProvider::Relay
    }

    async fn get_transactions(&self, cursor: Option<String>) -> Result<SwapPartnerTransactionsPage, SwapperError> {
        let cursor: RelayPartnerCursor = cursor.map(|cursor| serde_json::from_str(&cursor)).transpose()?.unwrap_or_default();
        let requested_at = Utc::now().timestamp();
        let query = RelayPartnerRequestsQuery {
            referrer: DEFAULT_REFERRER.to_string(),
            sort_by: "createdAt",
            sort_direction: "asc",
            limit: REQUESTS_LIMIT,
            start_timestamp: cursor.start_timestamp,
            continuation: cursor.continuation.clone(),
        };
        let response = self.client.get_partner_requests(query).await?;
        Ok(SwapPartnerTransactionsPage {
            transactions: response.requests.iter().filter_map(map_partner_transaction).collect(),
            cursor: map_next_cursor(cursor, &response, requested_at)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_next_cursor() {
        let response: RelayPartnerRequestsResponse = serde_json::from_str(include_str!("testdata/partner_requests.json")).unwrap();
        let start = RelayPartnerCursor {
            start_timestamp: Some(1_790_000_000),
            continuation: None,
        };

        assert_eq!(
            map_next_cursor(start.clone(), &response, 1_790_600_000).unwrap(),
            SwapPartnerCursor::Next(r#"{"startTimestamp":1790000000,"continuation":"page-2"}"#.to_string())
        );

        let last_page = RelayPartnerRequestsResponse { continuation: None, ..response };
        assert_eq!(map_next_cursor(start, &last_page, 1_790_600_000).unwrap(), SwapPartnerCursor::Latest(r#"{"startTimestamp":1790513600}"#.to_string()));
    }
}
