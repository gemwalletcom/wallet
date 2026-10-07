mod client;
mod gasflow;
mod helius;
mod jito;
mod solana_client;

use clap::{Parser, ValueEnum};
use std::error::Error;
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::time::interval;

use crate::jito::{format_micro_lamports, lamports_to_sol, priority_fee_to_lamports};
use crate::{
    client::{GemstoneClient, GemstoneFeeData},
    gasflow::GasflowClient,
    helius::{HeliusClient, HeliusPriorityFees},
    jito::{JitoClient, JitoTipFloor},
    solana_client::{SolanaFeeData, SolanaGasClient},
};
use etherscan::EtherscanClient;
use gem_evm::ether_conv::EtherConv;
use gem_evm::fee_calculator::{get_fee_history_blocks, get_reward_percentiles};
use gem_solana::JUPITER_PROGRAM_ID;
use gemstone::alien::reqwest_provider::NativeProvider;
use primitives::EVMChain;
use primitives::fee::FeePriority;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum ChainMode {
    Ethereum,
    Solana,
}

#[derive(Debug, Clone)]
struct SourceFeeDetail {
    source_name: String,
    base_fee: String,
    gas_used_ratio: Option<String>,
    normal_fee: String,
    fast_fee: String,
}

#[derive(Parser, Debug)]
#[clap(
    author,
    version,
    about,
    long_about = "A CLI tool to benchmark gas/priority fees from multiple sources.\n\
It periodically fetches fee data and displays comparative tables.\n\n\
For Ethereum: compares Core fee rates with Etherscan and Gasflow.\n\
For Solana: compares Core fee rates with RPC samples, Helius, and the Jito tip floor API."
)]
struct Cli {
    /// Chain to benchmark
    #[clap(short, long, value_enum, default_value_t = ChainMode::Ethereum)]
    chain: ChainMode,

    /// Enable debug logging
    #[arg(long, short, action = clap::ArgAction::SetTrue)]
    debug: bool,

    /// The Etherscan API key (Ethereum only, optional: keyless requests are rate limited)
    #[clap(long, env = "ETHERSCAN_API_KEY")]
    etherscan_api_key: Option<String>,

    /// Compute units for priority fee calculation (Solana only)
    #[clap(long, default_value_t = 200_000)]
    compute_units: u64,

    /// Skip Jito API comparison (Solana only)
    #[clap(long, action = clap::ArgAction::SetTrue)]
    skip_jito: bool,

    /// Helius API key for getPriorityFeeEstimate (Solana only)
    #[clap(long, env = "HELIUS_API_KEY")]
    helius_api_key: Option<String>,
}

async fn run_ethereum(args: Cli) -> Result<(), Box<dyn Error + Send + Sync>> {
    let chain = EVMChain::Ethereum;
    let mut ticker = interval(Duration::from_secs(6));
    let native_provider = Arc::new(NativeProvider::new().set_debug(args.debug));
    let gemstone_client = GemstoneClient::new(native_provider, chain);
    let etherscan_client = EtherscanClient::new_with_reqwest_client(gem_client::reqwest_client(), args.etherscan_api_key);
    let gasflow_client = GasflowClient::new();

    let mut last_printed_block_opt: Option<u64> = None;
    let mut block_data: HashMap<u64, Vec<SourceFeeDetail>> = HashMap::new();
    println!(
        "gas-bench [Ethereum]: Core fee history blocks: {}, reward percentiles: {:?}, min priority fee: {} Gwei",
        get_fee_history_blocks(chain),
        get_reward_percentiles(),
        EtherConv::to_gwei(&chain.min_priority_fee().into())
    );

    loop {
        ticker.tick().await;
        if args.debug {
            eprintln!("gas-bench: fetching new gas fee data...");
        }

        let (gemstone_res, etherscan_res, gasflow_res) = tokio::join!(gemstone_client.get_fee_data(), etherscan_client.get_gas_oracle(chain), gasflow_client.get_prediction());

        if args.debug {
            eprintln!("gas-bench: processing new fetch cycle, block_data currently has {} entries.", block_data.len());
        }

        let process_fee_data = |source_name: &str, data: &GemstoneFeeData| -> SourceFeeDetail {
            let mut normal = "N/A".to_string();
            let mut fast = "N/A".to_string();
            for fee_record in &data.priority_fees {
                match fee_record.priority {
                    FeePriority::Normal => normal = EtherConv::to_gwei(&fee_record.value),
                    FeePriority::Fast => fast = EtherConv::to_gwei(&fee_record.value),
                }
            }
            SourceFeeDetail {
                source_name: source_name.to_string(),
                base_fee: data.suggest_base_fee.clone(),
                gas_used_ratio: data.gas_used_ratio.clone(),
                normal_fee: normal,
                fast_fee: fast,
            }
        };

        if let Ok(data) = gemstone_res {
            let entry = block_data.entry(data.latest_block).or_default();
            if !entry.iter().any(|d| d.source_name == "Gemstone") {
                entry.push(process_fee_data("Gemstone", &data));
            }
        } else if let Err(error) = gemstone_res
            && args.debug
        {
            eprintln!("gas-bench: Error fetching Gemstone data: {error:?}");
        }

        if let Ok(oracle) = etherscan_res {
            let fee_data = GemstoneFeeData::from(oracle);
            let entry = block_data.entry(fee_data.latest_block).or_default();
            if !entry.iter().any(|d| d.source_name == "Etherscan") {
                entry.push(process_fee_data("Etherscan", &fee_data));
            }
        } else if let Err(error) = etherscan_res
            && args.debug
        {
            eprintln!("Error fetching Etherscan data: {error:?}");
        }

        if let Ok(data) = gasflow_res {
            let fee_data = data.fee_data();
            let entry = block_data.entry(fee_data.latest_block).or_default();
            if !entry.iter().any(|d| d.source_name == "Gasflow") {
                entry.push(process_fee_data("Gasflow", &fee_data));
            }
        } else if let Err(error) = gasflow_res
            && args.debug
        {
            eprintln!("Error fetching Gasflow data: {error:?}");
        }

        if args.debug {
            eprintln!("Debug: Aggregated block_data summary:");
            let mut sorted_debug_keys: Vec<_> = block_data.keys().collect();
            sorted_debug_keys.sort();
            for block_num in sorted_debug_keys {
                if let Some(details) = block_data.get(block_num) {
                    let sources: Vec<&str> = details.iter().map(|d| d.source_name.as_str()).collect();
                    eprintln!("  Block {block_num}: {sources:?}");
                }
            }
        }

        let mut sorted_blocks_in_map: Vec<u64> = block_data.keys().cloned().collect();
        sorted_blocks_in_map.sort_unstable();

        let block_to_print_this_iteration: Option<u64> = match last_printed_block_opt {
            Some(last_printed) => sorted_blocks_in_map
                .into_iter()
                .find(|&block_num| block_num > last_printed && block_data.get(&block_num).is_some_and(|details| details.len() >= 2)),
            None => sorted_blocks_in_map.iter().find(|&&b_num| block_data.get(&b_num).is_some_and(|details| details.len() >= 2)).cloned(),
        };

        if args.debug {
            eprintln!("Debug: last_printed_block_opt: {last_printed_block_opt:?}");
            eprintln!("Debug: block_to_print_this_iteration: {block_to_print_this_iteration:?}");
        }

        if let Some(current_block_to_print) = block_to_print_this_iteration {
            if args.debug {
                eprintln!("Debug: Attempting to print table for block: {current_block_to_print}");
            }
            if let Some(details_for_block) = block_data.get(&current_block_to_print)
                && details_for_block.len() >= 2
            {
                println!("\n--- Block: {current_block_to_print} ---");
                let headers = ["Source", "Base Fee (Gwei)", "Used Gas (%)", "Normal (Gwei)", "Fast (Gwei)"];
                let rows = details_for_block
                    .iter()
                    .map(|detail| {
                        vec![
                            detail.source_name.as_str(),
                            detail.base_fee.as_str(),
                            detail.gas_used_ratio.as_deref().unwrap_or("N/A"),
                            detail.normal_fee.as_str(),
                            detail.fast_fee.as_str(),
                        ]
                    })
                    .collect::<Vec<_>>();
                print_aligned_table(&headers, &rows);
                last_printed_block_opt = Some(current_block_to_print);
            }
        }
    }
}

async fn run_solana(args: Cli) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut ticker = interval(Duration::from_secs(6));
    let native_provider = Arc::new(NativeProvider::new().set_debug(args.debug));
    let solana_client = SolanaGasClient::new(native_provider);
    let jito_client = if args.skip_jito { None } else { Some(JitoClient::new()) };
    let helius_client = args.helius_api_key.as_ref().map(|key| HeliusClient::new(key));

    let mut last_printed_slot: Option<u64> = None;

    println!("gas-bench [Solana]: monitoring priority fees and Jito tips");
    println!("  Compute units for fee calculation: {}", args.compute_units);
    println!("  Jito comparison: {}", if jito_client.is_some() { "enabled" } else { "disabled" });
    println!("  Helius comparison: {}", if helius_client.is_some() { "enabled" } else { "disabled (set HELIUS_API_KEY)" });
    println!();

    loop {
        ticker.tick().await;

        if args.debug {
            eprintln!("gas-bench: fetching Solana fee data...");
        }

        let solana_future = solana_client.get_fee_data();
        let jito_future = async {
            match &jito_client {
                Some(client) => Some(client.get_tip_floor().await),
                None => None,
            }
        };
        let helius_future = async {
            match &helius_client {
                Some(client) => Some(client.get_priority_fee_estimate(Some(vec![JUPITER_PROGRAM_ID.to_string()])).await),
                None => None,
            }
        };

        let (solana_res, jito_res, helius_res) = tokio::join!(solana_future, jito_future, helius_future);

        match solana_res {
            Ok(fee_data) => {
                if last_printed_slot.is_some_and(|s| s >= fee_data.slot) {
                    continue;
                }

                print_solana_fee_data(&fee_data, &jito_res, &helius_res, args.compute_units);
                last_printed_slot = Some(fee_data.slot);
            }
            Err(error) => {
                if args.debug {
                    eprintln!("gas-bench: Error fetching Solana data: {error:?}");
                }
            }
        }
    }
}

fn print_solana_fee_data(fee_data: &SolanaFeeData, jito_res: &Option<Result<JitoTipFloor, Box<dyn Error + Send + Sync>>>, helius_res: &Option<Result<HeliusPriorityFees, Box<dyn Error + Send + Sync>>>, compute_units: u64) {
    println!("\n--- Slot: {} ---", fee_data.slot);

    let accounts = [("Jupiter", &fee_data.account_fees.jupiter), ("Orca", &fee_data.account_fees.orca), ("USDC", &fee_data.account_fees.usdc)];
    let active_accounts: Vec<&str> = accounts.iter().filter(|(_, data)| data.as_ref().is_some_and(|d| d.count > 0)).map(|(name, _)| *name).collect();

    let jito_available = jito_res.as_ref().is_some_and(Result::is_ok);
    let helius_data = helius_res.as_ref().and_then(|r| r.as_ref().ok());

    if !active_accounts.is_empty() {
        print!("Sampling: {} (avg: {} µL/CU)", active_accounts.join(", "), fee_data.raw_fees.avg);
    }
    if let Some(helius) = helius_data {
        print!(" | Helius: normal={} fast={}", format_micro_lamports(helius.medium), format_micro_lamports(helius.high));
    }
    println!();

    let core_rows = fee_data
        .core_rates
        .iter()
        .map(|rate| {
            vec![
                rate.transfer.to_string(),
                format!("{:?}", rate.priority),
                format!("{} µL/CU", format_micro_lamports(rate.unit_price)),
                rate.priority_fee.to_string(),
                rate.total_fee.to_string(),
            ]
        })
        .collect::<Vec<_>>();
    print_aligned_table(&["Core", "Level", "Unit Price", "Priority (lamports)", "Total (lamports)"], &core_rows);

    let mut headers = vec!["Level", "Priority (70%)", "Jito Tip (30%)", "Total"];
    if jito_available {
        headers.push("Jito Floor");
    }

    let levels = [("Normal", fee_data.priority_fees.normal, fee_data.jito_tips.normal), ("Fast", fee_data.priority_fees.fast, fee_data.jito_tips.fast)];

    let jito_data = jito_res.as_ref().and_then(|r| r.as_ref().ok());

    let mut rows = Vec::new();
    for (level, priority_fee, jito_tip) in levels.iter() {
        let priority_lamports = priority_fee_to_lamports(*priority_fee, compute_units);
        let total_lamports = priority_lamports + jito_tip;
        let priority_display = format!("{} ({})", format_micro_lamports(*priority_fee), lamports_to_sol(priority_lamports));
        let jito_tip_display = lamports_to_sol(*jito_tip);
        let total_display = lamports_to_sol(total_lamports);

        let mut row = vec![level.to_string(), priority_display, jito_tip_display, total_display];

        if let Some(jito) = jito_data {
            let jito_floor = match *level {
                "Normal" => jito.p50_lamports,
                "Fast" => jito.p75_lamports,
                _ => 0,
            };
            row.push(lamports_to_sol(jito_floor));
        }

        rows.push(row);
    }

    print_aligned_table(&headers, &rows);

    if let Some(Err(e)) = jito_res {
        println!("  (Jito API error: {})", e);
    }
    if let Some(Err(e)) = helius_res {
        println!("  (Helius API error: {})", e);
    }
}

fn print_aligned_table<T: AsRef<str>>(headers: &[&str], rows: &[Vec<T>]) {
    for line in aligned_table_lines(headers, rows) {
        println!("{line}");
    }
}

fn aligned_table_lines<T: AsRef<str>>(headers: &[&str], rows: &[Vec<T>]) -> Vec<String> {
    let mut widths = headers.iter().map(|header| header.chars().count()).collect::<Vec<_>>();
    for row in rows {
        for (width, cell) in widths.iter_mut().zip(row) {
            *width = (*width).max(cell.as_ref().chars().count());
        }
    }

    std::iter::once(format_table_row(headers.iter().copied(), &widths))
        .chain(rows.iter().map(|row| format_table_row(row.iter().map(AsRef::as_ref), &widths)))
        .collect()
}

fn format_table_row<'a>(cells: impl Iterator<Item = &'a str>, widths: &[usize]) -> String {
    cells.zip(widths).map(|(cell, width)| format!("{cell:<width$}")).collect::<Vec<_>>().join("  ").trim_end().to_string()
}

async fn run(args: Cli) -> Result<(), Box<dyn Error + Send + Sync>> {
    match args.chain {
        ChainMode::Ethereum => run_ethereum(args).await,
        ChainMode::Solana => run_solana(args).await,
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let args = Cli::parse();
    if args.debug {
        eprintln!("gas-bench: debug mode enabled by CLI flag.");
    }

    if let Err(error) = run(args).await {
        eprintln!("gas-bench: run error: {error}");
        std::process::exit(1);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::aligned_table_lines;

    #[test]
    fn test_aligned_table_lines() {
        let headers = ["Level", "Priority"];
        let rows = vec![vec!["Normal", "12 µL/CU"], vec!["Fast", "100 µL/CU"]];

        assert_eq!(aligned_table_lines(&headers, &rows), vec!["Level   Priority", "Normal  12 µL/CU", "Fast    100 µL/CU"]);
    }
}
