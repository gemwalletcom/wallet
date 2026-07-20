mod chains;
mod checker;
mod factory;
mod fixtures;
mod service;

use std::{error::Error, process::ExitCode};

use clap::Parser;
use factory::new_checker;
use gem_tracing::error_with_fields;
use primitives::Chain;

use crate::{fixtures::fixture, service::NodeCheckService};

#[derive(Parser)]
#[command(name = "nodecheck", about = "Check blockchain node capabilities and data")]
struct Args {
    chain: Chain,
    url: String,
    #[arg(short = 'H', long = "header", value_name = "NAME:VALUE")]
    headers: Vec<String>,
}

impl Args {
    async fn run(&self) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let fixture = fixture(self.chain).ok_or_else(|| format!("node fixtures are not configured for {}", self.chain))?;
        let checker = new_checker(self.chain, self.url.clone(), &self.headers)?;
        Ok(NodeCheckService::new(self.chain, checker).run(fixture).await)
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
