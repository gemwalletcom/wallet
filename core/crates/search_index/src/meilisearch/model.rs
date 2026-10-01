use serde::{Deserialize, Serialize};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct CreateIndexRequest<'a> {
    pub(super) uid: &'a str,
    pub(super) primary_key: &'a str,
}

#[derive(Serialize)]
pub(super) struct SearchRequest<'a> {
    #[serde(rename = "q")]
    pub(super) query: &'a str,
    pub(super) filter: &'a str,
    pub(super) sort: &'a [&'a str],
    pub(super) limit: usize,
    pub(super) offset: usize,
}

#[derive(Deserialize)]
pub(super) struct SearchResponse<T> {
    pub(super) hits: Vec<T>,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[derive(Debug, Deserialize, PartialEq)]
    struct Document {
        id: String,
    }

    #[test]
    fn test_create_index_request() {
        let request = CreateIndexRequest { uid: "assets", primary_key: "id" };

        assert_eq!(serde_json::to_value(request).unwrap(), json!({ "uid": "assets", "primaryKey": "id" }));
    }

    #[test]
    fn test_search_request() {
        let request = SearchRequest {
            query: "ethereum",
            filter: "chain = ethereum",
            sort: &["score.rank:asc"],
            limit: 20,
            offset: 40,
        };

        assert_eq!(
            serde_json::to_value(request).unwrap(),
            json!({
                "q": "ethereum",
                "filter": "chain = ethereum",
                "sort": ["score.rank:asc"],
                "limit": 20,
                "offset": 40,
            })
        );
    }

    #[test]
    fn test_search_response() {
        let response: SearchResponse<Document> = serde_json::from_value(json!({
            "hits": [{ "id": "ethereum" }],
            "processingTimeMs": 1,
            "limit": 20,
            "offset": 0,
        }))
        .unwrap();

        assert_eq!(response.hits, vec![Document { id: "ethereum".to_string() }]);
    }
}
