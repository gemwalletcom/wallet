mod checker;
mod fixtures;

use std::process::ExitCode;

use checker::{Checker, NodeCheck, NodeCheckResult};
use clap::Parser;
use gem_tracing::{error_with_fields, info_with_fields};
use primitives::Chain;

#[derive(Parser)]
#[command(name = "nodecheck", about = "Check blockchain node capabilities and data")]
struct Args {
    chain: Chain,
    url: String,
    #[arg(long)]
    archival: bool,
}

#[tokio::main]
async fn main() -> ExitCode {
    let args = Args::parse();
    let result = match Checker::new(args.chain, args.url, args.archival) {
        Ok(checker) => checker.check().await,
        Err(error) => Err(error),
    };
    match result {
        Ok(NodeCheckResult::Evm {
            chain,
            addresses,
            transactions,
            archival,
        }) => {
            let chain = chain.to_chain();
            info_with_fields!(
                "node check completed",
                chain = chain,
                addresses = addresses,
                transactions = transactions,
                archival = archival
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            error_with_fields!("node check failed", error.as_ref(), chain = args.chain);
            ExitCode::FAILURE
        }
    }
}
