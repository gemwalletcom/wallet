use std::fmt::Display;

use gem_tracing::{DurationMs, error_fields, error_fields_impl, info_with_fields, info_with_fields_impl};
use primitives::{Chain, NodeStatusState};

use crate::config::Url;

use super::observation::NodeStatusObservation;
use super::selection::NodeSwitchResult;

pub(super) struct NodeTelemetry;

impl NodeTelemetry {
    pub(super) fn log_check_started(chain: Chain, url: &Url) {
        info_with_fields!("Node check started", chain = chain.as_ref(), host = url.host());
    }

    pub(super) fn log_observation(chain: Chain, current: &Url, observation: &NodeStatusObservation) {
        let chain = chain.as_ref();
        match &observation.state {
            NodeStatusState::Healthy(status) if observation.url == *current && status.in_sync => log_observation("Node ok", chain, observation, info_with_fields_impl),
            NodeStatusState::Healthy(_) if observation.url == *current => log_observation("Node out of sync", chain, observation, error_fields_impl),
            NodeStatusState::Healthy(_) => log_observation("Node check", chain, observation, info_with_fields_impl),
            NodeStatusState::Error { .. } => log_observation("Node check error", chain, observation, error_fields_impl),
        }
    }

    pub(super) fn log_node_switch(chain: Chain, previous: &Url, switch: &NodeSwitchResult<'_>) {
        let chain = chain.as_ref();
        let observation = &switch.observation;
        let latency = DurationMs(observation.latency);
        let (latest, current) = match &observation.state {
            NodeStatusState::Healthy(status) => (status.latest_block_number, if status.in_sync { None } else { status.current_block_number }),
            NodeStatusState::Error { .. } => (None, None),
        };

        emit_event(
            "Node switch",
            chain,
            [("new_host", observation.url.host()), ("old_host", previous.host()), ("reason", switch.reason.to_string())],
            &latency,
            latest,
            current,
            info_with_fields_impl,
        );
    }

    pub(super) fn log_no_candidate(chain: Chain) {
        error_fields!("Node switch unavailable", chain = chain.as_ref());
    }

    pub(super) fn log_missing_current(chain: Chain) {
        error_fields!("Node monitor current missing", chain = chain.as_ref());
    }
}

fn log_observation(message: &'static str, chain: &str, observation: &NodeStatusObservation, sink: impl Fn(&'static str, &[(&str, &dyn Display)])) {
    let latency = DurationMs(observation.latency);
    match &observation.state {
        NodeStatusState::Healthy(status) => {
            let mut fields = vec![("host", observation.url.host())];
            if !status.in_sync {
                fields.push(("in_sync", "false".to_string()));
            }
            let current = if status.in_sync { None } else { status.current_block_number };
            emit_event(message, chain, fields, &latency, status.latest_block_number, current, sink);
        }
        NodeStatusState::Error { message: error } => {
            let fields = [
                ("node_host", observation.url.host()),
                ("error_type", observation.monitor_error.as_ref().to_string()),
                ("reason", observation.monitor_error.to_string()),
                ("error", error.clone()),
            ];
            emit_event(message, chain, fields, &latency, None, None, sink);
        }
    }
}

fn emit_event<I>(
    message: &'static str,
    chain: &str,
    fields: I,
    latency: &DurationMs,
    latest: Option<u64>,
    current: Option<u64>,
    sink: impl Fn(&'static str, &[(&str, &dyn Display)]),
) where
    I: IntoIterator<Item = (&'static str, String)>,
{
    let mut values: Vec<(&'static str, String)> = fields.into_iter().collect();
    if let Some(latest) = latest {
        values.push(("latest_block", latest.to_string()));
    }
    if let Some(current) = current {
        values.push(("current_block", current.to_string()));
    }

    let mut display: Vec<(&str, &dyn Display)> = Vec::with_capacity(values.len() + 2);
    display.push(("chain", &chain));
    display.push(("latency", latency));
    for (key, value) in &values {
        display.push((*key, value));
    }

    sink(message, &display);
}
