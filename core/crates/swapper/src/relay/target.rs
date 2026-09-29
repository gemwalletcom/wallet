use gem_client::{Target, build_path_with_query};

use super::model::RelayPartnerRequestsQuery;

#[derive(Clone, Debug)]
pub enum RelayTarget {
    Quote,
    Request { term: String },
    Requests { user: String, origin_chain_id: u64 },
    PartnerRequests(RelayPartnerRequestsQuery),
    Chains,
}

impl Target for RelayTarget {
    fn path(&self) -> String {
        match self {
            Self::Quote => "/quote/v2".to_string(),
            Self::Request { term } => format!("/requests/v3?term={term}"),
            Self::Requests { user, origin_chain_id } => format!("/requests/v3?user={user}&originChainId={origin_chain_id}"),
            Self::PartnerRequests(query) => build_path_with_query("/requests/v3", query),
            Self::Chains => "/chains".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_partner_requests_path() {
        let query = RelayPartnerRequestsQuery {
            api_key: "key".to_string(),
            referrer: "gemwallet".to_string(),
            sort_by: "updatedAt",
            sort_direction: "asc",
            limit: 100,
            include_authenticated_data: true,
            start_timestamp: Some(1790592300),
            continuation: None,
        };

        assert_eq!(
            RelayTarget::PartnerRequests(query).path(),
            "/requests/v3?apiKey=key&referrer=gemwallet&sortBy=updatedAt&sortDirection=asc&limit=100&includeAuthenticatedData=true&startTimestamp=1790592300"
        );
    }
}
