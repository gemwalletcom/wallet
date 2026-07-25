mod factory;
mod service;

use std::{error::Error, process::ExitCode};

use clap::Parser;
use factory::new_provider;
use gem_tracing::error_with_fields;
use primitives::{Chain, NodeCheckProfile};
use settings_chain::node_check_request;

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
}

impl Args {
    async fn run(&self) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let request = node_check_request(self.chain, self.profile);
        Ok(NodeCheckService::new(request, new_provider(self.chain, &self.url, &self.headers)?).run().await)
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
