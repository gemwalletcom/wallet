use std::time::Instant;

use gem_tracing::{DurationMs, error_fields, info_with_fields};
use reqwest::Method;
use rocket::http::Status;

use super::route::Route;

pub(super) struct AccessLog<'a> {
    id: String,
    caller: &'a str,
    group: &'a str,
    service: &'a str,
    method: &'a Method,
    uri: &'a str,
    start: Instant,
}

impl<'a> AccessLog<'a> {
    pub(super) fn new(caller: &'a str, route: &'a Route, method: &'a Method, uri: &'a str) -> Self {
        Self {
            id: format!("{:016x}", rand::random::<u64>()),
            caller,
            group: &route.group,
            service: &route.service,
            method,
            uri,
            start: Instant::now(),
        }
    }

    pub(super) fn route_not_found(method: &'a Method, uri: &'a str) {
        let access = Self {
            id: format!("{:016x}", rand::random::<u64>()),
            caller: "none",
            group: "none",
            service: "none",
            method,
            uri,
            start: Instant::now(),
        };
        access.request();
        access.unavailable(Status::NotFound.code, "route");
    }

    pub(super) fn request(&self) {
        info_with_fields!(
            "Egress request",
            id = self.id.as_str(),
            caller = self.caller,
            group = self.group,
            service = self.service,
            method = self.method.as_str(),
            uri = self.uri,
        );
    }

    pub(super) fn failover(&self, endpoint: &str, remote_host: &str, status: u16) {
        info_with_fields!(
            "Egress failover",
            id = self.id.as_str(),
            caller = self.caller,
            group = self.group,
            service = self.service,
            endpoint = endpoint,
            remote_host = remote_host,
            method = self.method.as_str(),
            uri = self.uri,
            status = status,
            latency = DurationMs(self.start.elapsed()),
        );
    }

    pub(super) fn upstream_failed(&self, endpoint: &str, remote_host: &str, reason: &str) {
        error_fields!(
            "Egress upstream failed",
            id = self.id.as_str(),
            caller = self.caller,
            group = self.group,
            service = self.service,
            endpoint = endpoint,
            remote_host = remote_host,
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
            id = self.id.as_str(),
            caller = self.caller,
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

    pub(super) fn response(&self, endpoint: &str, remote_host: &str, status: u16) {
        info_with_fields!(
            "Egress response",
            id = self.id.as_str(),
            caller = self.caller,
            group = self.group,
            service = self.service,
            endpoint = endpoint,
            remote_host = remote_host,
            method = self.method.as_str(),
            uri = self.uri,
            status = status,
            latency = DurationMs(self.start.elapsed()),
        );
    }
}
