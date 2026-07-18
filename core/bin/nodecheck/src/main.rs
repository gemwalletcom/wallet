mod chains;
mod checker;
mod factory;
mod fixtures;

use std::{error::Error, process::ExitCode};

use clap::Parser;
use factory::new_checker;
use gem_tracing::{error_with_fields, info_with_fields};
use primitives::Chain;

use crate::fixtures::fixture;

#[derive(Parser)]
#[command(name = "nodecheck", about = "Check blockchain node capabilities and data")]
struct Args {
    chain: Chain,
    url: String,
    #[arg(short = 'H', long = "header", value_name = "NAME:VALUE")]
    headers: Vec<String>,
    #[arg(long)]
    archival: bool,
}

impl Args {
    async fn run(&self) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let checker = new_checker(self.chain, self.url.clone(), &self.headers)?;
        let fixture = fixture(self.chain).ok_or_else(|| format!("node fixtures are not configured for {}", self.chain))?;
        info_with_fields!("---------------- load_balancer ----------------", chain = self.chain);
        let load_balancer = self.log_section_result("load_balancer", checker.check_load_balancer().await);
        info_with_fields!("---------------- indexer ----------------", chain = self.chain);
        let indexer = self.log_section_result("indexer", checker.check_indexer(fixture, self.archival).await);
        let summary = format!(
            "node check summary\nsection          status\n---------------  ------\nload_balancer    {}\nindexer          {}",
            if load_balancer { "passed" } else { "failed" },
            if indexer { "passed" } else { "failed" }
        );
        info_with_fields!(&summary);
        Ok(load_balancer && indexer)
    }

    fn log_section_result(&self, section: &str, result: Result<(), Box<dyn Error + Send + Sync>>) -> bool {
        match result {
            Ok(()) => {
                info_with_fields!("node check completed", chain = self.chain, section = section);
                true
            }
            Err(error) => {
                error_with_fields!("node check failed", error.as_ref(), chain = self.chain, section = section);
                false
            }
        }
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
