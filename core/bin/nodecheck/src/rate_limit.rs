use std::{
    collections::BTreeMap,
    sync::Arc,
    time::{Duration, Instant},
};

use chain_traits::ChainTraits;
use primitives::{NodeCheckRequest, NodeCheckStatus};
use tokio::{
    task::JoinSet,
    time::{MissedTickBehavior, interval},
};

use crate::result_table::{ResultStatus, ResultTable};

const TEST_DURATION: Duration = Duration::from_secs(1);

#[derive(Default)]
struct CheckResults {
    passed: u32,
    warnings: u32,
    failed: u32,
}

#[derive(Default)]
struct MethodResults {
    checks: CheckResults,
    first_warning: Option<String>,
    first_error: Option<String>,
}

impl MethodResults {
    fn record(&mut self, status: NodeCheckStatus) {
        match status {
            NodeCheckStatus::Passed { .. } => self.checks.passed += 1,
            NodeCheckStatus::Warning { warning } => {
                self.checks.warnings += 1;
                self.first_warning.get_or_insert(warning);
            }
            NodeCheckStatus::Failed { error } => {
                self.checks.failed += 1;
                self.first_error.get_or_insert(error);
            }
        }
    }
}

pub(crate) async fn run(request: Arc<NodeCheckRequest>, provider: Arc<dyn ChainTraits>, profile_runs_per_second: u32) -> bool {
    let title = format!("{} / {} / rate limit: {} profile runs/s", provider.get_chain(), request.profile(), profile_runs_per_second);
    let table = ResultTable::start(&title, "metric / method", false);

    let request_interval = TEST_DURATION.div_f64(f64::from(profile_runs_per_second));
    let mut ticker = interval(request_interval);
    ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);
    ticker.tick().await;
    let mut requests = JoinSet::new();
    let started = Instant::now();

    for _ in 0..profile_runs_per_second {
        let provider = Arc::clone(&provider);
        let request = Arc::clone(&request);
        requests.spawn(async move {
            let status_started = Instant::now();
            let status = provider.get_node_status().await.map_err(|error| error.to_string())?;
            Ok(provider.check_node(request.as_ref(), &status, status_started.elapsed()).await)
        });
        ticker.tick().await;
    }

    let mut profile_results = CheckResults::default();
    let mut method_results = BTreeMap::<String, MethodResults>::new();
    while let Some(result) = requests.join_next().await {
        match result {
            Ok(Ok(report)) => {
                if report.is_healthy() {
                    profile_results.passed += 1;
                } else {
                    profile_results.failed += 1;
                }
                for (method, result) in report.checks {
                    method_results.entry(method).or_default().record(result.status);
                }
            }
            Ok(Err(error)) => {
                profile_results.failed += 1;
                method_results.entry("node_status".to_string()).or_default().record(NodeCheckStatus::Failed { error });
            }
            Err(error) => {
                profile_results.failed += 1;
                method_results
                    .entry("task".to_string())
                    .or_default()
                    .record(NodeCheckStatus::Failed { error: error.to_string() });
            }
        }
    }

    let passed = profile_results.failed == 0;
    print_results(&table, started.elapsed(), profile_results, method_results);
    passed
}

fn print_results(table: &ResultTable, elapsed: Duration, profiles: CheckResults, method_results: BTreeMap<String, MethodResults>) {
    let totals = method_results.values().fold(CheckResults::default(), |totals, method| CheckResults {
        passed: totals.passed + method.checks.passed,
        warnings: totals.warnings + method.checks.warnings,
        failed: totals.failed + method.checks.failed,
    });
    let elapsed_seconds = elapsed.as_secs_f64();
    let check_throughput = f64::from(totals.passed + totals.warnings + totals.failed) / elapsed_seconds;
    let estimated_rpc = f64::from(totals.passed) / elapsed_seconds;
    let profile_status = ResultStatus::from_counts(profiles.warnings, profiles.failed);

    table.row(profile_status, "profiles", None, &format!("passed: {}, failed: {}", profiles.passed, profiles.failed));
    table.row(
        ResultStatus::from_counts(totals.warnings, totals.failed),
        "checks",
        None,
        &format!("passed: {}, warnings: {}, failed: {}", totals.passed, totals.warnings, totals.failed),
    );
    let (throughput_status, throughput) = if totals.failed == 0 {
        (
            profile_status,
            format!("{check_throughput:.2} checks/s; at least {estimated_rpc:.2} successful RPC/s; {elapsed_seconds:.2}s; no failures"),
        )
    } else {
        let useful_checks = totals.passed + totals.failed;
        let failed_percent = f64::from(totals.failed) / f64::from(useful_checks) * 100.0;
        (
            ResultStatus::Failed,
            format!(
                "{check_throughput:.2} checks/s; ~{estimated_rpc:.2} successful RPC/s; {elapsed_seconds:.2}s; {}/{useful_checks} failed ({failed_percent:.1}%)",
                totals.failed
            ),
        )
    };
    table.row(throughput_status, "throughput", None, &throughput);
    for (method, results) in method_results {
        if let Some(error) = results.first_error {
            table.row(ResultStatus::Failed, &method, None, &format!("failed: {}, {error}", results.checks.failed));
        }
        if let Some(warning) = results.first_warning {
            table.row(ResultStatus::Warning, &method, None, &format!("warnings: {}, {warning}", results.checks.warnings));
        }
    }
    table.finish(profiles.failed == 0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_method_results_records_outcomes() {
        let mut results = MethodResults::default();
        results.record(NodeCheckStatus::Passed { result: "ok".to_string() });
        results.record(NodeCheckStatus::Warning {
            warning: "unsupported".to_string(),
        });
        results.record(NodeCheckStatus::Failed {
            error: "HTTP error: status 429 (-32900)".to_string(),
        });
        results.record(NodeCheckStatus::Failed { error: "other error".to_string() });

        assert_eq!(results.checks.passed, 1);
        assert_eq!(results.checks.warnings, 1);
        assert_eq!(results.checks.failed, 2);
        assert_eq!(results.first_warning.as_deref(), Some("unsupported"));
        assert_eq!(results.first_error.as_deref(), Some("HTTP error: status 429 (-32900)"));
    }
}
