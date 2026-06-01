mod format;
mod loader;
mod runner;

use std::time::Duration;

use crate::AppState;
use crate::config::Settings;

pub(crate) struct ScheduleEntry {
    pub(crate) name: String,
    pub(crate) cadence: Duration,
    pub(crate) prompt: String,
}

pub fn spawn_all(state: AppState) {
    let dir = state.settings.agent_dir().join("schedules");
    for entry in loader::load_from_dir(&dir) {
        runner::spawn_one(state.clone(), entry);
    }
}

pub(crate) fn load_for(settings: &Settings) -> Vec<ScheduleEntry> {
    let dir = settings.agent_dir().join("schedules");
    loader::load_from_dir(&dir)
}
