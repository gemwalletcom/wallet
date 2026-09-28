mod parser_options;

pub use parser_options::ParserOptions;

use std::{
    error::Error,
    sync::Arc,
    time::{Duration, Instant},
};

use crate::health::HealthState;
use crate::metrics::parser::ParserMetrics;
use crate::reporters::parser::ParserReporter;
use chain_traits::ChainTraits;
use gem_tracing::{DurationMs, error_with_fields, info_with_fields};
use primitives::Chain;
use services::Services;
use settings::Settings;
use streamer::{StreamProducer, StreamProducerQueue, TransactionsPayload};

use crate::shutdown::{self, ShutdownReceiver};
use services::transactions::{BlockPlan, BlockPlanKind, ParserState, ParserStateService, plan_next_block, should_reload_catchup, timeout_for_state};

pub struct Parser {
    chain: Chain,
    provider: Box<dyn ChainTraits>,
    stream_producer: StreamProducer,
    state_service: ParserStateService,
    reporter: ParserReporter,
    options: ParserOptions,
    shutdown_rx: ShutdownReceiver,
}

impl Parser {
    pub fn new(provider: Box<dyn ChainTraits>, stream_producer: StreamProducer, state_service: ParserStateService, parser_metrics: Arc<ParserMetrics>, options: ParserOptions, shutdown_rx: ShutdownReceiver) -> Self {
        let chain = provider.get_chain();
        let reporter = ParserReporter::new(chain, parser_metrics);
        Self {
            chain,
            provider,
            stream_producer,
            state_service,
            reporter,
            options,
            shutdown_rx,
        }
    }

    fn is_shutdown(&self) -> bool {
        *self.shutdown_rx.borrow()
    }

    async fn sleep_or_shutdown(&self, duration: Duration) -> bool {
        shutdown::sleep_or_shutdown(duration, &self.shutdown_rx).await
    }

    async fn wait_if_disabled(&self, state: &ParserState, timeout: Duration) -> bool {
        if state.is_enabled {
            true
        } else {
            self.sleep_or_shutdown(timeout).await;
            false
        }
    }

    async fn get_latest_block(&self, state: &ParserState) -> Result<i64, Box<dyn Error + Send + Sync>> {
        let latest_block = self.provider.get_block_latest_number().await? as i64;
        let _ = self.state_service.set_latest_block(latest_block).await;

        if state.current_block == 0 {
            let _ = self.state_service.set_current_block(latest_block).await;
        }

        Ok(latest_block)
    }

    async fn execute_plan(&self, plan: BlockPlan, state: &ParserState, timeout: Duration) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let start = Instant::now();
        let blocks_desc = format!("{:?}", plan.range.blocks);

        match plan.kind {
            BlockPlanKind::Enqueue => {
                self.stream_producer.publish_blocks(self.chain, &plan.range.blocks).await?;
                let _ = self.state_service.set_current_block(plan.range.end_block).await;

                info_with_fields!(
                    "block add to queue",
                    chain = self.chain.as_ref(),
                    blocks = blocks_desc,
                    remaining = plan.range.remaining,
                    duration = DurationMs(start.elapsed())
                );
                return Ok(true);
            }
            BlockPlanKind::Parse => {}
        }

        match self.parse_blocks(plan.range.blocks).await {
            Ok(result) => {
                let _ = self.state_service.set_current_block(plan.range.end_block).await;

                info_with_fields!(
                    "block complete",
                    chain = self.chain.as_ref(),
                    blocks = blocks_desc,
                    transactions = result,
                    remaining = plan.range.remaining,
                    duration = DurationMs(start.elapsed())
                );
            }
            Err(err) => {
                error_with_fields!("parser parse_block", &*err, chain = self.chain.as_ref(), blocks = blocks_desc);
                self.sleep_or_shutdown(timeout).await;
                return Ok(false);
            }
        }

        if should_reload_catchup(plan.range.remaining, self.options.catchup_reload_interval) {
            return Ok(false);
        }
        if state.timeout_between_blocks > 0 && self.sleep_or_shutdown(Duration::from_millis(state.timeout_between_blocks as u64)).await {
            return Ok(false);
        }

        Ok(true)
    }

    async fn process_blocks(&self, timeout: Duration) -> Result<(), Box<dyn Error + Send + Sync>> {
        loop {
            if self.is_shutdown() {
                break;
            }

            let state = self.state_service.get_state().await?;

            let Some(plan) = plan_next_block(&state, state.current_block, state.latest_block) else {
                break;
            };

            if !self.execute_plan(plan, &state, timeout).await? {
                break;
            }
        }

        Ok(())
    }

    pub async fn start(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        loop {
            if self.is_shutdown() {
                info_with_fields!("shutdown requested", chain = self.chain.as_ref());
                break;
            }

            let state = self.state_service.get_state().await?;
            self.reporter.update_state(state.current_block, state.latest_block, state.is_enabled);
            let timeout = timeout_for_state(&state, self.options.min_check, self.options.max_check);

            if !self.wait_if_disabled(&state, timeout).await {
                continue;
            }

            let latest_block = match self.get_latest_block(&state).await {
                Ok(block) => block,
                Err(err) => {
                    error_with_fields!("parser latest_block", &*err, chain = self.chain.as_ref());
                    self.sleep_or_shutdown(self.options.error_interval).await;
                    continue;
                }
            };

            if state.current_block + state.await_blocks as i64 >= latest_block {
                info_with_fields!(
                    "parser ahead",
                    chain = self.chain.as_ref(),
                    current_block = state.current_block,
                    latest_block = latest_block,
                    await_blocks = state.await_blocks,
                    next_check = DurationMs(timeout)
                );
                self.sleep_or_shutdown(timeout).await;
                continue;
            }

            self.process_blocks(timeout).await?;
        }

        info_with_fields!("parser stopped", chain = self.chain.as_ref());

        Ok(())
    }

    async fn parse_blocks(&self, blocks: Vec<u64>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let transactions = self.provider.get_transactions_in_blocks(blocks.clone()).await?;
        self.reporter.record_transactions(&transactions);
        let payload = TransactionsPayload::new_with_notify(self.chain, blocks, transactions);
        self.stream_producer.publish_transactions(payload).await
    }
}

pub async fn run(settings: Settings, chain: Option<Chain>, health_state: Arc<HealthState>, parser_metrics: Arc<ParserMetrics>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let services = Services::new(Arc::new(settings.clone()))?;

    let config = services.config();
    let catchup_reload_interval = config.get_i64(config_keys::ConfigKey::ParserCatchupReloadInterval).await?;
    let min_check = config.get_duration(config_keys::ConfigKey::ParserMinCheckInterval).await?;
    let max_check = config.get_duration(config_keys::ConfigKey::ParserMaxCheckInterval).await?;
    let error_interval = config.get_duration(config_keys::ConfigKey::ParserErrorInterval).await?;

    let chains: Vec<Chain> = if let Some(chain) = chain { vec![chain] } else { services.parser_chains().await? };

    let chain_names = chains.iter().map(Chain::as_ref).collect::<Vec<_>>().join(",");
    let checks = format!("{}..{}", gem_tracing::human_duration(min_check), gem_tracing::human_duration(max_check));
    info_with_fields!(
        "parser start",
        chains = chain_names,
        checks = checks,
        retry = DurationMs(settings.parser.timeout),
        error_retry = DurationMs(error_interval),
        catchup_reload = catchup_reload_interval
    );

    let (shutdown_tx, shutdown_rx) = shutdown::channel();
    let shutdown_timeout = settings.parser.shutdown.timeout;

    let signal_handle = shutdown::spawn_signal_handler(shutdown_tx);

    let mut handles = Vec::new();

    for chain in chains {
        let state_service = services.parser_state(chain);
        let parser_metrics = parser_metrics.clone();
        let shutdown_rx = shutdown_rx.clone();
        let settings = settings.clone();

        let provider = chain_providers::ProviderFactory::new_from_settings_with_user_agent(chain, &settings, &settings::service_user_agent("parser", None));

        let stream_producer = services.stream_producer(format!("parser_{chain}").as_str(), shutdown_rx.clone()).await?;

        let options = ParserOptions {
            timeout: settings.parser.timeout,
            catchup_reload_interval,
            min_check,
            max_check,
            error_interval,
        };

        handles.push(tokio::spawn(async move {
            run_parser(state_service, parser_metrics, stream_producer, provider, options, shutdown_rx).await;
        }));
    }

    health_state.set_ready();
    info_with_fields!("parsers ready", chains = handles.len());

    signal_handle.await.ok();
    info_with_fields!("waiting for parser shutdown", tasks = handles.len());
    let _ = shutdown::wait_with_timeout(handles, shutdown_timeout).await;

    info_with_fields!("all parsers stopped", status = "ok");
    Ok(())
}

async fn run_parser(state_service: ParserStateService, parser_metrics: Arc<ParserMetrics>, stream_producer: StreamProducer, provider: Box<dyn ChainTraits>, options: ParserOptions, shutdown_rx: ShutdownReceiver) {
    let chain = provider.get_chain();
    let timeout = options.timeout;

    let parser = Parser::new(provider, stream_producer, state_service, parser_metrics, options, shutdown_rx.clone());

    loop {
        if *shutdown_rx.borrow() {
            break;
        }

        if let Err(e) = parser.start().await {
            error_with_fields!("parser error", &*e, chain = chain.as_ref());

            if shutdown::sleep_or_shutdown(timeout, &shutdown_rx).await {
                break;
            }
        }
    }
}
