use std::fs;
use std::path::{Path, PathBuf};

use crate::Result;
use gem_tracing::human_duration;

use crate::config::Settings;

pub struct PreambleFile {
    pub source: String,
    pub content: String,
}

pub fn files(settings: &Settings) -> Result<Vec<PreambleFile>> {
    let mut out = Vec::new();
    let root = PathBuf::from(&settings.defaults.dir);

    read_md_files(&root.join("context"), "context", &settings.agent.include_context, &mut out)?;

    let agent_dir = settings.agent_dir();
    let agent_prefix = format!("agents/{}", settings.agent_name);
    read_md_dir(&agent_dir, &agent_prefix, &mut out)?;
    read_md_dir(&agent_dir.join("memory"), &format!("{agent_prefix}/memory"), &mut out)?;

    Ok(out)
}

pub fn indexable_files(settings: &Settings) -> Result<Vec<PreambleFile>> {
    let mut out = Vec::new();
    let agent_dir = settings.agent_dir();
    let prefix = format!("agents/{}/memory", settings.agent_name);
    read_md_dir(&agent_dir.join("memory"), &prefix, &mut out)?;
    Ok(out)
}

pub fn render(settings: &Settings) -> Result<String> {
    let mut out = String::new();
    for f in files(settings)? {
        out.push_str(&format!("\n--- {} ---\n", f.source));
        out.push_str(&f.content);
        out.push('\n');
    }
    out.push_str(&schedules_summary(settings));
    Ok(out)
}

fn schedules_summary(settings: &Settings) -> String {
    let entries = crate::scheduler::load_for(settings);
    if entries.is_empty() {
        return String::new();
    }
    let mut out = String::from("\n--- scheduled tasks ---\nThe following tasks fire automatically on this agent — no human prompts them.\n");
    for e in &entries {
        out.push_str(&format!("- `{}` every {}\n", e.name, human_duration(e.cadence)));
    }
    out
}

fn read_md_files(dir: &Path, prefix: &str, include: &[String], out: &mut Vec<PreambleFile>) -> Result<()> {
    for name in include {
        let p = dir.join(name);
        let content = fs::read_to_string(&p).map_err(|e| format!("include_context: {}: {e}", p.display()))?;
        out.push(PreambleFile {
            source: format!("{prefix}/{name}"),
            content,
        });
    }
    Ok(())
}

fn read_md_dir(dir: &Path, prefix: &str, out: &mut Vec<PreambleFile>) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    let mut entries: Vec<_> = fs::read_dir(dir)?.filter_map(|e| e.ok()).collect();
    entries.sort_by_key(|e| e.path());
    for entry in entries {
        let p = entry.path();
        if !p.is_file() {
            continue;
        }
        if p.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }
        let name = p.file_name().and_then(|s| s.to_str()).unwrap_or("").to_string();
        out.push(PreambleFile {
            source: format!("{prefix}/{name}"),
            content: fs::read_to_string(&p)?,
        });
    }
    Ok(())
}
