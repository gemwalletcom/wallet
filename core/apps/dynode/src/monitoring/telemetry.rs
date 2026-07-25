use gem_tracing::{DurationMs, error_fields, error_fields_impl, info_with_fields_impl};
use primitives::NodeStatusState;

use crate::config::{ChainConfig, Url};

use super::observation::NodeStatusObservation;
use super::selection::NodeSwitchResult;

pub struct NodeTelemetry;

impl NodeTelemetry {
    pub(super) fn log_observations(chain_config: &ChainConfig, current: &Url, observations: &[NodeStatusObservation]) {
        let chain = chain_config.chain.as_ref();
        for observation in observations {
            match &observation.state {
                NodeStatusState::Healthy(status) if observation.url == *current && status.in_sync => log_observation("Node ok", chain, observation, info_with_fields_impl),
                NodeStatusState::Healthy(_) if observation.url == *current => log_observation("Node out of sync", chain, observation, error_fields_impl),
                NodeStatusState::Error { .. } if observation.url == *current => log_observation("Node check error", chain, observation, error_fields_impl),
                NodeStatusState::Healthy(_) | NodeStatusState::Error { .. } => log_observation("Node check", chain, observation, info_with_fields_impl),
            }
        }
    }

    pub(super) fn log_node_switch(chain_config: &ChainConfig, previous: &Url, switch: &NodeSwitchResult<'_>) {
        let chain = chain_config.chain.as_ref();
        let observation = &switch.observation;
        let latency = DurationMs(observation.latency);
        let (latest, current) = match &observation.state {
            NodeStatusState::Healthy(status) => (status.latest_block_number, if status.in_sync { None } else { status.current_block_number }),
            NodeStatusState::Error { .. } => (None, None),
        };

        log_info_event(
            "Node switch",
            chain,
            [("new_host", observation.url.host()), ("old_host", previous.host()), ("reason", switch.reason.to_string())],
            &latency,
            latest,
            current,
        );
    }

    pub(super) fn log_no_candidate(chain_config: &ChainConfig, observations: &[NodeStatusObservation]) {
        error_fields!(
            "Node switch unavailable",
            chain = chain_config.chain.as_ref(),
            statuses = &format_status_summary(observations),
        );
    }

    pub(super) fn log_missing_current(chain_config: &ChainConfig) {
        error_fields!("Node monitor current missing", chain = chain_config.chain.as_ref());
    }
}

fn log_observation(message: &'static str, chain: &str, observation: &NodeStatusObservation, sink: impl Fn(&'static str, &[(&str, &dyn std::fmt::Display)])) {
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
            emit_event(message, chain, [("host", observation.url.host()), ("message", error.clone())], &latency, None, None, sink);
        }
    }
}

fn log_info_event<I>(message: &'static str, chain: &str, fields: I, latency: &DurationMs, latest: Option<u64>, current: Option<u64>)
where
    I: IntoIterator<Item = (&'static str, String)>,
{
    emit_event(message, chain, fields, latency, latest, current, info_with_fields_impl);
}

fn emit_event<I>(
    message: &'static str,
    chain: &str,
    fields: I,
    latency: &DurationMs,
    latest: Option<u64>,
    current: Option<u64>,
    sink: impl Fn(&'static str, &[(&str, &dyn std::fmt::Display)]),
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

    let mut display: Vec<(&str, &dyn std::fmt::Display)> = Vec::with_capacity(values.len() + 2);
    display.push(("chain", &chain));
    for (key, value) in &values {
        display.push((*key, value));
    }
    display.push(("latency", latency));

    sink(message, &display);
}

fn format_status_summary(observations: &[NodeStatusObservation]) -> String {
    observations
        .iter()
        .map(|observation| match &observation.state {
            NodeStatusState::Healthy(status) => format!(
                "{}:in_sync={} latest={} current={} latency={}ms",
                observation.url.url,
                status.in_sync,
                format_optional_number(status.latest_block_number),
                format_optional_number(status.current_block_number),
                observation.latency.as_millis()
            ),
            NodeStatusState::Error { message } => format!("{}:error={} latency={}ms", observation.url.url, message, observation.latency.as_millis()),
        })
        .collect::<Vec<_>>()
        .join("; ")
}

fn format_optional_number(value: Option<u64>) -> String {
    value.map(|value| value.to_string()).unwrap_or_else(|| "unknown".to_string())
}
