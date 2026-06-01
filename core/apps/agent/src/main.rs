use agent::{build_runtime, chatwoot, resolve_agent_name, scheduler, slack};
use gem_tracing::tracing::{error, info};

#[tokio::main]
async fn main() -> agent::Result<()> {
    rustls::crypto::ring::default_provider().install_default().ok();
    gem_tracing::init_tracing("agent=debug,info");

    let state = build_runtime(&resolve_agent_name()?).await?;
    info!(
        agent = %state.settings.agent_name,
        bot_user_id = %state.bot_user_id,
        chatwoot = state.chatwoot.is_some(),
        "gemmy starting"
    );

    scheduler::spawn_all(state.clone());
    let slack_task = tokio::spawn(slack::socket::run_forever(state.clone()));
    let webhook_enabled = state.settings.chatwoot.bot.webhook.enabled && state.chatwoot.is_some();
    let chatwoot_task = webhook_enabled.then(|| tokio::spawn(chatwoot::server::run_forever(state.clone())));

    tokio::select! {
        res = slack_task => {
            log_loop_exit("slack", res);
        }
        res = optional_join(chatwoot_task), if webhook_enabled => {
            log_loop_exit("chatwoot", res);
        }
        _ = tokio::signal::ctrl_c() => info!("shutdown signal received"),
    }
    Ok(())
}

async fn optional_join(handle: Option<tokio::task::JoinHandle<agent::Result<()>>>) -> Result<agent::Result<()>, tokio::task::JoinError> {
    match handle {
        Some(h) => h.await,
        None => std::future::pending().await,
    }
}

fn log_loop_exit(name: &str, res: Result<agent::Result<()>, tokio::task::JoinError>) {
    match res {
        Ok(Ok(())) => info!(loop = name, "loop exited cleanly"),
        Ok(Err(e)) => error!(loop = name, error = %e, "loop failed"),
        Err(e) => error!(loop = name, error = %e, "loop task panicked"),
    }
}
