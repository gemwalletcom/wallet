use std::process::Stdio;
use std::time::Duration;

use gem_tracing::tracing::debug;
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::io::AsyncWriteExt;

use super::ToolFailure;

#[derive(Clone)]
pub struct ShellTool {
    pub workdir: String,
    pub timeout_secs: u64,
    pub allow: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShellArgs {
    pub argv: Vec<String>,
    #[serde(default)]
    pub stdin: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ShellOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

impl Tool for ShellTool {
    const NAME: &'static str = "shell";
    type Error = ToolFailure;
    type Args = ShellArgs;
    type Output = ShellOutput;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Run an allowlisted binary directly (no shell). Pass `argv` as an array \
                where `argv[0]` is the program (must be in `agent.shell.allow`) and the rest are \
                its arguments. There is no shell interpretation — no pipes, no redirects, no \
                `&&`/`||`/`;`, no `$VAR` expansion, no globs. To chain or pipe, call the tool \
                multiple times and process output in-context. Returns stdout, stderr, and exit \
                code. Be specific — broad commands waste tokens."
                .to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "argv": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Program + args, no shell. e.g. [\"git\", \"log\", \"--grep=foo\", \"--since=3 months ago\"]."
                    },
                    "stdin": {
                        "type": "string",
                        "description": "Optional stdin payload."
                    }
                },
                "required": ["argv"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let program = args.argv.first().ok_or_else(|| ToolFailure::other("argv must be non-empty"))?.clone();
        debug!(program = %program, argc = args.argv.len(), "shell tool");
        if !self.allow.iter().any(|a| a == &program) {
            return Err(ToolFailure::not_allowed(format!("command not allowed: `{program}` (not in agent.shell.allow)")));
        }

        let mut cmd = tokio::process::Command::new(&program);
        cmd.args(&args.argv[1..])
            .current_dir(&self.workdir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd.spawn()?;
        if let (Some(stdin_payload), Some(mut child_stdin)) = (args.stdin, child.stdin.take()) {
            child_stdin.write_all(stdin_payload.as_bytes()).await?;
            child_stdin.shutdown().await?;
        }

        let output = tokio::time::timeout(Duration::from_secs(self.timeout_secs), child.wait_with_output())
            .await
            .map_err(|_| ToolFailure::other(format!("timed out after {}s", self.timeout_secs)))??;
        Ok(ShellOutput {
            stdout: truncate(&String::from_utf8_lossy(&output.stdout), 60_000),
            stderr: truncate(&String::from_utf8_lossy(&output.stderr), 8_000),
            exit_code: output.status.code().unwrap_or(-1),
        })
    }
}

fn truncate(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        let head: String = s.chars().take(max_chars).collect();
        let total = s.chars().count();
        format!("{head}\n…[truncated, {total} chars total]")
    }
}
