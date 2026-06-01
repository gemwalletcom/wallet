use std::io::{BufRead, Write};

use agent::{Result, build_runtime, resolve_agent_name};

#[tokio::main]
async fn main() -> Result<()> {
    gem_tracing::init_tracing("agent=info,warn");

    let state = build_runtime(&resolve_agent_name()?).await?;

    eprintln!("gemmy REPL — agent `{}` — type a message, get a reply. Ctrl+D to quit.", state.settings.agent_name);
    eprintln!("(faking a DM from an admin — admin privileges active)\n");

    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    let mut lines = stdin.lock().lines();

    loop {
        write!(out, "you> ")?;
        out.flush()?;
        let Some(line) = lines.next() else { break };
        let msg = line?.trim().to_string();
        if msg.is_empty() {
            continue;
        }

        let header = "[Slack — channel: DM, channel_id: REPL, message_ts: 0.0, \
                      user_id: UADMIN0000, addressed: addressed]";
        let prompt = format!("{header}\n\n{msg}");

        match state.agent.prompt(&prompt).await {
            Ok(reply) => writeln!(out, "\ngemmy> {reply}\n")?,
            Err(e) => writeln!(out, "\nerror: {e}\n")?,
        }
    }

    eprintln!("\nbye");
    Ok(())
}
