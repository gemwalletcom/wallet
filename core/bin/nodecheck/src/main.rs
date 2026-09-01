mod factory;
mod rate_limit;
mod result_table;
mod service;

use std::{error::Error, process::ExitCode};

use clap::Parser;
use factory::new_provider;
use gem_tracing::error_with_fields;
use primitives::node_check_request;
use primitives::{Chain, NodeCheckProfile};

use crate::service::NodeCheckService;

#[derive(Parser)]
#[command(name = "nodecheck", about = "Check blockchain node capabilities and data")]
struct Args {
    chain: Chain,
    url: String,
    #[arg(short, long)]
    profile: NodeCheckProfile,
    #[arg(short = 'H', long = "header", value_name = "NAME:VALUE")]
    headers: Vec<String>,
    #[arg(long, value_name = "PROFILE_RUNS_PER_SECOND", value_parser = clap::value_parser!(u32).range(1..=10_000))]
    rate_limit: Option<u32>,
}

impl Args {
    async fn run(&self) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let request = node_check_request(self.chain, self.profile);
        let service = NodeCheckService::new(request, new_provider(self.chain, &self.url, &self.headers)?);
        let passed = service.run().await;
        Ok(match self.rate_limit {
            Some(profile_runs_per_second) => service.run_rate_limit(profile_runs_per_second).await && passed,
            None => passed,
        })
    }
}

#[tokio::main]
async fn main() -> ExitCode {
    let args = Args::parse();
    match args.run().await {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(error) => {
            error_with_fields!("node check failed", error.as_ref(), chain = args.chain);
            ExitCode::FAILURE
        }
    }
}
