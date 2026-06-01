use std::time::Duration;

use gem_tracing::tracing::{info, warn};
use tokio::time::{Instant, interval_at};

use crate::slack::mrkdwn::to_slack_mrkdwn;
use crate::{AppState, DISPATCH_SOURCE, DispatchSource};

use super::ScheduleEntry;
use super::format::{self, Status};

pub(super) fn spawn_one(state: AppState, entry: ScheduleEntry) {
    tokio::spawn(async move { run_loop(state, entry.name, entry.prompt, entry.cadence).await });
}

async fn run_loop(state: AppState, name: String, prompt: String, cadence: Duration) {
    info!(schedule = %name, cadence_secs = cadence.as_secs(), "scheduler started");
    let mut tick = interval_at(Instant::now() + cadence, cadence);
    loop {
        tick.tick().await;
        let started = Instant::now();
        info!(schedule = %name, "scheduler firing");
        post_status(&state, &name, Status::Started).await;
        let result = DISPATCH_SOURCE.scope(DispatchSource::Scheduled, state.agent.prompt_response(&prompt)).await;
        match result {
            Ok(response) => {
                let elapsed = started.elapsed();
                let reply_chars = response.output.len();
                post_status(&state, &name, Status::Succeeded { elapsed, response: &response }).await;
                info!(schedule = %name, reply_chars, "scheduler completed");
            }
            Err(e) => {
                let elapsed = started.elapsed();
                let error = e.to_string();
                post_status(&state, &name, Status::Failed { elapsed, error: &error }).await;
                warn!(schedule = %name, error = %e, "scheduler failed");
            }
        }
    }
}

async fn post_status(state: &AppState, schedule: &str, status: Status<'_>) {
    let channel = &state.settings.scheduler.status_channel;
    let text = format::status(&state.settings.agent_name, schedule, status);
    let text = to_slack_mrkdwn(&text);
    match state.slack.post_message(channel, None, &text).await {
        Ok(ts) => info!(channel = %channel, ts = %ts, "schedule status posted"),
        Err(e) => warn!(channel = %channel, error = %e, "schedule status post failed"),
    }
}
