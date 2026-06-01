use std::fs;
use std::path::Path;
use std::time::Duration;

use config::{Config, File, FileFormat};
use gem_tracing::tracing::error;
use serde::Deserialize;
use serde_serializers::duration;

use super::ScheduleEntry;

pub(super) fn load_from_dir(dir: &Path) -> Vec<ScheduleEntry> {
    let mut entries = Vec::new();
    if dir.exists() {
        collect_md(dir, &mut entries);
    }
    entries
}

fn collect_md(dir: &Path, entries: &mut Vec<ScheduleEntry>) {
    let Ok(read) = fs::read_dir(dir) else {
        error!(path = %dir.display(), "cannot read schedules dir");
        return;
    };
    for f in read.filter_map(|e| e.ok()) {
        let path = f.path();
        if path.is_dir() {
            collect_md(&path, entries);
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            match parse_schedule_md(&path) {
                Ok(entry) => entries.push(entry),
                Err(e) => error!(path = %path.display(), error = %e, "invalid schedule; skipping"),
            }
        }
    }
}

#[derive(Deserialize)]
struct ScheduleMeta {
    name: String,
    #[serde(deserialize_with = "duration::deserialize")]
    every: Duration,
}

fn parse_schedule_md(path: &Path) -> Result<ScheduleEntry, String> {
    let raw = fs::read_to_string(path).map_err(|e| format!("read: {e}"))?;
    let body = raw.trim_start().strip_prefix("---\n").ok_or("missing `---` frontmatter")?;
    let (meta_yaml, prompt) = body.split_once("\n---\n").ok_or("missing closing `---`")?;
    let meta: ScheduleMeta = Config::builder()
        .add_source(File::from_str(meta_yaml, FileFormat::Yaml))
        .build()
        .and_then(|c| c.try_deserialize())
        .map_err(|e| format!("frontmatter: {e}"))?;
    Ok(ScheduleEntry {
        name: meta.name,
        cadence: meta.every,
        prompt: prompt.trim().to_string(),
    })
}
