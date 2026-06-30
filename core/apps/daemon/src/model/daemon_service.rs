use std::str::FromStr;

use primitives::Chain;
use strum::AsRefStr;

use super::{ConsumerOptions, ConsumerService, IndexerConsumer, WorkerOptions, WorkerService};

#[derive(Debug, Clone, AsRefStr)]
#[strum(serialize_all = "snake_case")]
pub enum DaemonService {
    Setup,
    SetupDev,
    #[strum(serialize = "worker")]
    Worker(WorkerOptions),
    #[strum(serialize = "parser")]
    Parser(Option<Chain>),
    #[strum(serialize = "consumer")]
    Consumer(ConsumerOptions),
}

impl DaemonService {
    pub fn name(&self) -> String {
        match self {
            DaemonService::Setup | DaemonService::SetupDev => self.as_ref().to_owned(),
            DaemonService::Worker(opts) => match (opts.service, opts.job.as_deref()) {
                (Some(service), Some(job)) => format!("worker {} {}", service.as_ref(), job),
                (Some(service), None) => format!("worker {}", service.as_ref()),
                (None, _) => "worker all".to_owned(),
            },
            DaemonService::Parser(chain) => match chain {
                Some(chain) => format!("parser {}", chain.as_ref()),
                None => "parser".to_owned(),
            },
            DaemonService::Consumer(opts) => match (&opts.service, opts.indexer) {
                (Some(service), Some(indexer)) => format!("consumer {} {}", service.as_ref(), indexer.as_ref()),
                (Some(service), None) => format!("consumer {}", service.as_ref()),
                (None, _) => "consumer all".to_owned(),
            },
        }
    }
}

impl FromStr for DaemonService {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        let name = parts.first().copied().ok_or_else(|| "Empty service name".to_string())?;

        match name {
            name if name == DaemonService::Setup.as_ref() => Ok(DaemonService::Setup),
            name if name == DaemonService::SetupDev.as_ref() => Ok(DaemonService::SetupDev),
            "worker" => {
                let service = parts.get(1).map(|s| WorkerService::from_str(s).map_err(|_| format!("Invalid worker: {s}"))).transpose()?;
                let job = parts.get(2).map(|s| (*s).to_owned());
                Ok(DaemonService::Worker(WorkerOptions { service, job }))
            }
            "parser" => {
                let chain = parts.get(1).map(|s| Chain::from_str(s).map_err(|_| format!("Invalid chain: {s}"))).transpose()?;
                Ok(DaemonService::Parser(chain))
            }
            "consumer" => {
                let service = parts
                    .get(1)
                    .map(|s| ConsumerService::from_str(s).map_err(|_| format!("Invalid consumer: {s}")))
                    .transpose()?;
                let indexer = if matches!(service, Some(ConsumerService::Indexer)) {
                    parts
                        .get(2)
                        .map(|s| IndexerConsumer::from_str(s).map_err(|_| format!("Invalid indexer consumer: {s}")))
                        .transpose()?
                } else {
                    None
                };
                Ok(DaemonService::Consumer(ConsumerOptions { service, indexer }))
            }
            _ => Err(format!("Unknown service: {name}")),
        }
    }
}
