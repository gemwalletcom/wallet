use std::time::Instant;

use gem_tracing::{DurationMs, error_fields, info_with_fields};
use reqwest::Method;
use rocket::http::Status;

use super::route::Route;

pub(super) struct AccessLog<'a> {
    source: &'a str,
    group: &'a str,
    service: &'a str,
    method: &'a Method,
    uri: &'a str,
    start: Instant,
}

impl<'a> AccessLog<'a> {
    pub(super) fn new(source: &'a str, route: &'a Route, method: &'a Method, uri: &'a str) -> Self {
        Self {
            source,
            group: &route.group,
            service: &route.service,
            method,
            uri,
            start: Instant::now(),
        }
    }

    pub(super) fn rejected(method: &'a Method, uri: &'a str, status: u16, reason: &str) {
        let access = Self {
            source: "none",
            group: "none",
            service: "none",
            method,
            uri,
            start: Instant::now(),
        };
        access.request();
        access.unavailable(status, reason);
    }

    pub(super) fn request(&self) {
        info_with_fields!(
            "Egress request",
            source = self.source,
            group = self.group,
            service = self.service,
            method = self.method.as_str(),
            uri = self.uri,
        );
    }

    pub(super) fn failover(&self, endpoint: &str, host: &str, status: u16) {
        info_with_fields!(
            "Egress failover",
            source = self.source,
            group = self.group,
            service = self.service,
            endpoint = endpoint,
            host = host,
            method = self.method.as_str(),
            uri = self.uri,
            status = status,
            latency = DurationMs(self.start.elapsed()),
        );
    }

    pub(super) fn upstream_failed(&self, endpoint: &str, host: &str, reason: &str) {
        error_fields!(
            "Egress upstream failed",
            source = self.source,
            group = self.group,
            service = self.service,
            endpoint = endpoint,
            host = host,
            method = self.method.as_str(),
            uri = self.uri,
            status = Status::BadGateway.code,
            reason = reason,
            latency = DurationMs(self.start.elapsed()),
        );
    }

    pub(super) fn unavailable(&self, status: u16, reason: &str) {
        error_fields!(
            "Egress unavailable",
            source = self.source,
            group = self.group,
            service = self.service,
            endpoint = "none",
            method = self.method.as_str(),
            uri = self.uri,
            status = status,
            reason = reason,
            latency = DurationMs(self.start.elapsed()),
        );
    }

    pub(super) fn response(&self, endpoint: &str, host: &str, status: u16) {
        info_with_fields!(
            "Egress response",
            source = self.source,
            group = self.group,
            service = self.service,
            endpoint = endpoint,
            host = host,
            method = self.method.as_str(),
            uri = self.uri,
            status = status,
            latency = DurationMs(self.start.elapsed()),
        );
    }
}
