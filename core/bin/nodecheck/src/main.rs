mod factory;
mod rate_limit;
mod result_table;
mod service;

use std::{error::Error, process::ExitCode};

use chain_traits::node_check::NodeCheckProfile;
use factory::new_provider;
use gem_args::{Args, ArgsError};
use gem_tracing::error_with_fields;
use node_probes::node_check_request;
use primitives::Chain;

use crate::service::NodeCheckService;

struct NodeCheck {
    chain: Chain,
    url: String,
    profile: NodeCheckProfile,
    headers: Vec<String>,
    rate_limit: Option<u32>,
}

impl NodeCheck {
    fn new(mut args: Args) -> Result<Self, ArgsError> {
        let profile = args.required(&["--profile", "-p"], "wallet|parser", "Checks to run");
        let headers = args.options(&["--header", "-H"], "NAME:VALUE", "Header sent with each request, repeatable");
        let rate_limit = args.option(&["--rate-limit"], "PROFILE_RUNS_PER_SECOND", "Also run the profile at this rate, 1 to 10000");
        let chain = args.positional("chain", "Chain the node serves");
        let url = args.positional("url", "Node URL");
        args.finish("Check blockchain node capabilities and data")?;
        let rate_limit = rate_limit.map(|value| value.parse()).transpose().map_err(ArgsError::invalid)?;
        if rate_limit.is_some_and(|value| !(1..=10_000).contains(&value)) {
            return Err(ArgsError::invalid("rate limit must be between 1 and 10000"));
        }
        Ok(Self {
            chain: chain.parse().map_err(ArgsError::invalid)?,
            url,
            profile: profile.parse().map_err(ArgsError::invalid)?,
            headers,
            rate_limit,
        })
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
    let node_check = match NodeCheck::new(Args::from_env()) {
        Ok(node_check) => node_check,
        Err(error) => return error.report(),
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
