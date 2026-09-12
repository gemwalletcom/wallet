use std::error::Error;

pub type BoxError = Box<dyn Error + Send + Sync>;

pub mod cache;
pub mod config;
mod failure_reason;
mod gateway;
pub mod jsonrpc_types;
pub mod metrics;
pub mod monitoring;
pub mod node_service;
pub mod proxy;
pub mod response;
pub mod server;
#[cfg(any(test, feature = "testkit"))]
pub mod testkit;
pub mod webhook;
