use std::collections::HashMap;

use crate::{CONTENT_TYPE, ContentType};

pub trait Target {
    fn path(&self) -> String;

    fn headers(&self) -> HashMap<String, String> {
        HashMap::new()
    }

    fn content_type(&self) -> ContentType {
        ContentType::ApplicationJson
    }
}

impl Target for &str {
    fn path(&self) -> String {
        self.to_string()
    }
}

impl Target for &String {
    fn path(&self) -> String {
        self.to_string()
    }
}

pub(crate) fn body_headers(target: &impl Target) -> HashMap<String, String> {
    let mut headers = target.headers();
    headers.entry(CONTENT_TYPE.to_string()).or_insert_with(|| target.content_type().as_str().to_string());
    headers
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ClientExt, testkit::MockClient};
    use serde_json::Value;

    #[tokio::test]
    async fn test_post_declares_json_next_to_call_site_headers() {
        let client = MockClient::new().with_post_with_headers(|path, body, headers| {
            assert_eq!(path, "/messages");
            assert_eq!(body, br#"{"content":"hello"}"#);
            assert_eq!(headers.get(CONTENT_TYPE).map(String::as_str), Some(ContentType::ApplicationJson.as_str()));
            assert_eq!(headers.get("x-auth-token").map(String::as_str), Some("token"));
            Ok(b"{}".to_vec())
        });

        let _: Value = client
            .post("/messages", &serde_json::json!({"content": "hello"}))
            .headers(HashMap::from([("x-auth-token".to_string(), "token".to_string())]))
            .await
            .unwrap();
    }
}
