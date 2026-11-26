mod factory;
mod rate_limit;
mod result_table;
mod service;

use std::{error::Error, process::ExitCode};

use chain_traits::node_check::NodeCheckProfile;
use factory::new_provider;
use gem_args::Args;
use gem_tracing::error_with_fields;
use node_probes::node_check_request;
use primitives::Chain;

use crate::service::NodeCheckService;

const USAGE: &str = "Check blockchain node capabilities and data\n\nUsage: nodecheck --profile <wallet|parser> [--header <NAME:VALUE>] [--rate-limit <PROFILE_RUNS_PER_SECOND>] <chain> <url>";

struct NodeCheck {
    chain: Chain,
    url: String,
    profile: NodeCheckProfile,
    headers: Vec<String>,
    rate_limit: Option<u32>,
}

impl NodeCheck {
    fn new(mut args: Args) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let profile = args.required(&["--profile", "-p"])?.parse()?;
        let headers = args.options(&["--header", "-H"])?;
        let rate_limit = args.option(&["--rate-limit"])?.map(|value| value.parse()).transpose()?;
        if rate_limit.is_some_and(|value| !(1..=10_000).contains(&value)) {
            return Err("rate limit must be between 1 and 10000".into());
        }
        let chain = args.positional("chain")?.parse()?;
        let url = args.positional("url")?;
        args.finish()?;
        Ok(Self { chain, url, profile, headers, rate_limit })
    }

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
    let mut args = Args::from_env();
    if args.flag(&["--help", "-h"]) {
        println!("{USAGE}");
        return ExitCode::SUCCESS;
    }
    let node_check = match NodeCheck::new(args) {
        Ok(node_check) => node_check,
        Err(error) => {
            eprintln!("{error}\n{USAGE}");
            return ExitCode::from(2);
        }
    };
    match node_check.run().await {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(error) => {
            error_with_fields!("node check failed", error.as_ref(), chain = node_check.chain);
            ExitCode::FAILURE
        }
    }
}
