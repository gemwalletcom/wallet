use std::time::Instant;

use gem_tracing::{DurationMs, error_fields, info_with_fields, path};
use reqwest::Method;

use super::route::Route;

pub(super) struct AccessLog<'a> {
    id: String,
    method: &'a Method,
    uri: String,
    start: Instant,
}

impl<'a> AccessLog<'a> {
    pub(super) fn new(method: &'a Method, uri: &'a str) -> Self {
        Self {
            id: format!("{:016x}", rand::random::<u64>()),
            method,
            uri: path::redact(uri),
            start: Instant::now(),
        }
    }

    pub(super) fn request(&self, route: Option<&Route>) {
        let (group, service) = route.map_or(("none", "none"), |route| (route.group.as_str(), route.service.as_str()));
        info_with_fields!(
            "Egress request",
            id = self.id.as_str(),
            group = group,
            service = service,
            method = self.method.as_str(),
            uri = self.uri,
        );
    }

    pub(super) fn failover(&self, route: &Route, endpoint: &str, remote_host: &str, status: u16) {
        info_with_fields!(
            "Egress failover",
            id = self.id.as_str(),
            group = route.group.as_str(),
            service = route.service.as_str(),
            endpoint = endpoint,
            remote_host = remote_host,
            method = self.method.as_str(),
            uri = self.uri,
            status = status,
            latency = DurationMs(self.start.elapsed()),
        );
    }

    pub(super) fn upstream_failed(&self, route: &Route, endpoint: &str, remote_host: &str, reason: &str) {
        error_fields!(
            "Egress upstream failed",
            id = self.id.as_str(),
            group = route.group.as_str(),
            service = route.service.as_str(),
            endpoint = endpoint,
            remote_host = remote_host,
            method = self.method.as_str(),
            uri = self.uri,
            status = 502,
            reason = reason,
            latency = DurationMs(self.start.elapsed()),
        );
    }

    pub(super) fn unavailable(&self, route: Option<&Route>, status: u16, reason: &str) {
        let (group, service) = route.map_or(("none", "none"), |route| (route.group.as_str(), route.service.as_str()));
        error_fields!(
            "Egress unavailable",
            id = self.id.as_str(),
            group = group,
            service = service,
            endpoint = "none",
            method = self.method.as_str(),
            uri = self.uri,
            status = status,
            reason = reason,
            latency = DurationMs(self.start.elapsed()),
        );
    }

    pub(super) fn response(&self, route: &Route, endpoint: &str, remote_host: &str, status: u16) {
        info_with_fields!(
            "Egress response",
            id = self.id.as_str(),
            group = route.group.as_str(),
            service = route.service.as_str(),
            endpoint = endpoint,
            remote_host = remote_host,
            method = self.method.as_str(),
            uri = self.uri,
            status = status,
            latency = DurationMs(self.start.elapsed()),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_is_excluded_from_logs() {
        let method = Method::GET;
        let access = AccessLog::new(&method, "/prices_tonapi/v2/rates?token=secret");
        assert_eq!(access.uri, "/prices_tonapi/v2/rates");
    }

    #[test]
    fn dynamic_segments_are_redacted() {
        let method = Method::GET;
        let access = AccessLog::new(&method, "/indexer_toncenter/api/v3/wallet/0:123456789012345678901234567890/42");
        assert_eq!(access.uri, "/indexer_toncenter/api/v3/wallet/:value/:number");
    }
}
