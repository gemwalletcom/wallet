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
            Self::PartnerRequests(query) => build_path_with_query("/requests/v2", query),
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
            referrer: "gemwallet".to_string(),
            sort_by: "createdAt",
            sort_direction: "asc",
            limit: 50,
            start_timestamp: Some(1790592300),
            continuation: None,
        };

        assert_eq!(
            RelayTarget::PartnerRequests(query).path(),
            "/requests/v2?referrer=gemwallet&sortBy=createdAt&sortDirection=asc&limit=50&startTimestamp=1790592300"
        );
    }
}
