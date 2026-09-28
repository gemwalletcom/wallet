use crate::params::{QueryLimitParam, SearchQueryParam};
use rocket::FromForm;

#[derive(FromForm)]
pub struct SearchParams<'r> {
    pub(crate) query: SearchQueryParam,
    pub(crate) chains: Option<&'r str>,
    pub(crate) tags: Option<&'r str>,
    pub(crate) limit: QueryLimitParam,
    pub(crate) offset: Option<usize>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::MAX_QUERY_LIMIT;
    use rocket::form::Form;
    use services::assets::SearchRequest;

    #[test]
    fn search_params_defaults_limit() {
        let params = Form::<SearchParams<'_>>::parse("query=btc").unwrap();

        assert_eq!(params.limit.0, MAX_QUERY_LIMIT);
    }

    #[test]
    fn search_params_accepts_max_limit() {
        let query = format!("query=btc&limit={MAX_QUERY_LIMIT}");
        let params = Form::<SearchParams<'_>>::parse(&query).unwrap();

        assert_eq!(params.limit.0, MAX_QUERY_LIMIT);
    }

    #[test]
    fn search_request_defaults_limit_above_max() {
        let query = format!("query=btc&limit={}", MAX_QUERY_LIMIT + 1);
        let params = Form::<SearchParams<'_>>::parse(&query).unwrap();
        let request = SearchRequest::new(&params.query.0, params.chains, params.tags, params.limit.0, params.offset);

        assert_eq!(request.limit, MAX_QUERY_LIMIT);
    }
}
