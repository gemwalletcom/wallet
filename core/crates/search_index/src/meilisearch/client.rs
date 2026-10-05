use std::{error::Error, fmt};

use reqwest::{Client, Method, RequestBuilder, Response, StatusCode};
use serde::{Serialize, de::DeserializeOwned};

use super::model::{CreateIndexRequest, SearchRequest, SearchResponse};
use crate::IndexConfig;

#[derive(Clone)]
pub struct SearchIndexClient {
    client: Client,
    url: String,
    api_key: String,
    config: SearchIndexConfig,
}

#[derive(Debug, Clone, Copy)]
pub struct SearchIndexConfig {
    pub batch_size: usize,
}

#[derive(Debug)]
enum SearchIndexError {
    InvalidBatchSize,
    Response { status: StatusCode, body: String },
}

impl fmt::Display for SearchIndexError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidBatchSize => write!(formatter, "Meilisearch batch size must be greater than zero"),
            Self::Response { status, body } => write!(formatter, "Meilisearch request failed with {status}: {body}"),
        }
    }
}

impl Error for SearchIndexError {}

impl fmt::Debug for SearchIndexClient {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("SearchIndexClient").field("url", &self.url).field("config", &self.config).finish()
    }
}

impl SearchIndexClient {
    pub fn new(url: &str, api_key: &str, config: SearchIndexConfig) -> Self {
        Self {
            client: Client::new(),
            url: url.to_string(),
            api_key: api_key.to_string(),
            config,
        }
    }

    pub async fn get_or_create_index(&self, name: &str, primary_key: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        let response = self.request(Method::GET, &format!("indexes/{name}")).send().await?;
        if response.status() == StatusCode::NOT_FOUND {
            self.send_json(Method::POST, "indexes", &CreateIndexRequest { uid: name, primary_key }).await?;
        } else {
            Self::validate_response(response, StatusCode::OK).await?;
        }
        Ok(())
    }

    async fn add_documents<T: Serialize + Send + Sync>(&self, index: &str, documents: &[T]) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.send_json(Method::POST, &format!("indexes/{index}/documents"), documents).await
    }

    async fn delete_all_documents(&self, index: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        let request = self.request(Method::DELETE, &format!("indexes/{index}/documents"));
        self.send(request, StatusCode::ACCEPTED).await?;
        Ok(())
    }

    pub async fn replace_documents<T: Serialize + Send + Sync>(&self, index: &str, documents: Vec<T>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let batch_size = self.batch_size()?;
        self.delete_all_documents(index).await?;
        self.index_documents_with_batch_size(index, documents, batch_size).await
    }

    pub async fn index_documents<T: Serialize + Send + Sync>(&self, index: &str, documents: Vec<T>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let batch_size = self.batch_size()?;
        self.index_documents_with_batch_size(index, documents, batch_size).await
    }

    async fn index_documents_with_batch_size<T: Serialize + Send + Sync>(&self, index: &str, documents: Vec<T>, batch_size: usize) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let count = documents.len();
        for batch in documents.chunks(batch_size) {
            self.add_documents(index, batch).await?;
        }
        Ok(count)
    }

    fn batch_size(&self) -> Result<usize, SearchIndexError> {
        match self.config.batch_size {
            0 => Err(SearchIndexError::InvalidBatchSize),
            batch_size => Ok(batch_size),
        }
    }

    async fn set_filterable_attributes(&self, index: &str, attributes: &[&str]) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.send_json(Method::PUT, &format!("indexes/{index}/settings/filterable-attributes"), attributes).await
    }

    async fn set_sortable_attributes(&self, index: &str, attributes: &[&str]) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.send_json(Method::PUT, &format!("indexes/{index}/settings/sortable-attributes"), attributes).await
    }

    async fn set_searchable_attributes(&self, index: &str, attributes: &[&str]) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.send_json(Method::PUT, &format!("indexes/{index}/settings/searchable-attributes"), attributes).await
    }

    async fn set_ranking_rules(&self, index: &str, attributes: &[&str]) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.send_json(Method::PUT, &format!("indexes/{index}/settings/ranking-rules"), attributes).await
    }

    pub async fn setup(&self, configs: &[IndexConfig], primary_key: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        for config in configs {
            self.get_or_create_index(config.name, primary_key).await?;
            self.set_filterable_attributes(config.name, config.filters).await?;
            self.set_sortable_attributes(config.name, config.sorts).await?;
            self.set_searchable_attributes(config.name, config.search_attributes).await?;
            self.set_ranking_rules(config.name, config.ranking_rules).await?;
        }
        Ok(())
    }

    pub(crate) async fn search<T: DeserializeOwned + Send + Sync + 'static>(&self, index: &str, query: &str, filter: &str, sort: &[&str], limit: usize, offset: usize) -> Result<Vec<T>, Box<dyn Error + Send + Sync>> {
        let request = SearchRequest { query, filter, sort, limit, offset };
        let response = self.request(Method::POST, &format!("indexes/{index}/search")).json(&request);
        Ok(self.send(response, StatusCode::OK).await?.json::<SearchResponse<T>>().await?.hits)
    }

    fn request(&self, method: Method, path: &str) -> RequestBuilder {
        self.client.request(method, format!("{}/{path}", self.url)).bearer_auth(&self.api_key)
    }

    async fn send_json<T: Serialize + ?Sized>(&self, method: Method, path: &str, body: &T) -> Result<(), Box<dyn Error + Send + Sync>> {
        let request = self.request(method, path).json(body);
        self.send(request, StatusCode::ACCEPTED).await?;
        Ok(())
    }

    async fn send(&self, request: RequestBuilder, expected_status: StatusCode) -> Result<Response, Box<dyn Error + Send + Sync>> {
        Self::validate_response(request.send().await?, expected_status).await
    }

    async fn validate_response(response: Response, expected_status: StatusCode) -> Result<Response, Box<dyn Error + Send + Sync>> {
        let status = response.status();
        if status == expected_status {
            return Ok(response);
        }
        let body = response.text().await?;
        Err(Box::new(SearchIndexError::Response { status, body }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_omits_api_key() {
        let client = SearchIndexClient::new("https://search.example", "secret-api-key", SearchIndexConfig { batch_size: 100 });

        assert_eq!(format!("{client:?}"), "SearchIndexClient { url: \"https://search.example\", config: SearchIndexConfig { batch_size: 100 } }");
    }

    #[test]
    fn test_zero_batch_size_is_rejected() {
        let client = SearchIndexClient::new("https://search.example", "secret-api-key", SearchIndexConfig { batch_size: 0 });

        assert_eq!(client.batch_size().unwrap_err().to_string(), "Meilisearch batch size must be greater than zero");
    }
}
