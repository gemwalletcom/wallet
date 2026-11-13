use axum::extract::FromRequestParts;
use http::request::Parts;
use http::{HeaderMap, HeaderName};

const COUNTRY_HEADER: HeaderName = HeaderName::from_static("cf-ipcountry");
const UNKNOWN_COUNTRY: &str = "XX";

pub struct ClientCountry(pub Option<String>);

impl<S: Send + Sync> FromRequestParts<S> for ClientCountry {
    type Rejection = !;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(Self(country(&parts.headers)))
    }
}

fn country(headers: &HeaderMap) -> Option<String> {
    let mut values = headers.get_all(COUNTRY_HEADER).iter();
    let (Some(value), None) = (values.next(), values.next()) else {
        return None;
    };
    let value = value.to_str().ok()?;
    (value.len() == 2 && value.bytes().all(|byte| byte.is_ascii_uppercase()) && value != UNKNOWN_COUNTRY).then(|| value.to_string())
}

#[cfg(test)]
mod tests {
    use http::{HeaderMap, HeaderValue};

    use super::{COUNTRY_HEADER, country};

    #[test]
    fn test_country() {
        assert_eq!(country(&HeaderMap::from_iter([(COUNTRY_HEADER, HeaderValue::from_static("DE"))])), Some("DE".to_string()));
        assert_eq!(country(&HeaderMap::new()), None);
        assert_eq!(country(&HeaderMap::from_iter([(COUNTRY_HEADER, HeaderValue::from_static("US")), (COUNTRY_HEADER, HeaderValue::from_static("DE"))])), None);
        assert_eq!(country(&HeaderMap::from_iter([(COUNTRY_HEADER, HeaderValue::from_static("XX"))])), None);
        assert_eq!(country(&HeaderMap::from_iter([(COUNTRY_HEADER, HeaderValue::from_static("T1"))])), None);
        assert_eq!(country(&HeaderMap::from_iter([(COUNTRY_HEADER, HeaderValue::from_static("de"))])), None);
        assert_eq!(country(&HeaderMap::from_iter([(COUNTRY_HEADER, HeaderValue::from_static("USA"))])), None);
    }
}
