use crate::model::WorkerService;
use crate::worker::jobs::{JobLabel, JobVariant, WorkerJob};
use config_keys::ConfigParamKey;
use job_runner::{JobContext, JobError, JobHandle, JobPlan};
use services::ConfigCacher;
use std::error::Error;
use std::fmt::Debug;
use std::future::Future;
use std::time::Duration;

type PlanResult = Result<JobPlan, Box<dyn Error + Send + Sync>>;
type AddJob = Box<dyn FnOnce(JobPlan, Duration) -> JobPlan + Send>;

struct PendingJob {
    variant: JobVariant,
    interval_param: Option<ConfigParamKey>,
    add: AddJob,
}

pub struct JobPlanBuilder<'a> {
    worker: WorkerService,
    plan: PlanResult,
    pending: Vec<PendingJob>,
    config: Option<&'a ConfigCacher>,
    filter: Option<String>,
}

impl<'a> JobPlanBuilder<'a> {
    pub fn with_config(worker: WorkerService, plan: JobPlan, config: &'a ConfigCacher) -> Self {
        Self {
            worker,
            plan: Ok(plan),
            pending: Vec::new(),
            config: Some(config),
            filter: None,
        }
    }

    pub fn filter(mut self, filter: Option<String>) -> Self {
        self.filter = filter;
        self
    }

    pub fn job<J, F, Fut, R>(mut self, job: J, job_fn: F) -> Self
    where
        J: Into<JobVariant>,
        F: Fn(JobContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<R, JobError>> + Send + 'static,
        R: Debug + Send + Sync + 'static,
    {
        let variant: JobVariant = job.into();
        if self.plan.is_ok() && variant.worker() != self.worker {
            self.plan = Err(format!("job {} belongs to {:?} worker but builder is {:?}", variant.name(), variant.worker(), self.worker).into());
        }
        if self.plan.is_ok() && should_include(&variant.name(), self.filter.as_deref()) {
            self.pending.push(Self::pending_job(variant, None, job_fn));
        }
        self
    }

    pub fn jobs<Items, Item, Builder, F, Fut, R>(self, job: WorkerJob, items: Items, build_job: Builder) -> Self
    where
        Items: IntoIterator<Item = Item>,
        Item: JobLabel + Clone + Send + Sync + 'static,
        Builder: Fn(Item, JobVariant) -> F,
        F: Fn(JobContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<R, JobError>> + Send + 'static,
        R: Debug + Send + Sync + 'static,
    {
        self.build_jobs(job, items, |_| None, build_job)
    }

    pub fn jobs_with_config<Items, Item, K, Builder, F, Fut, R>(self, job: WorkerJob, items: Items, config_key: K, build_job: Builder) -> Self
    where
        Items: IntoIterator<Item = Item>,
        Item: JobLabel + Clone + Send + Sync + 'static,
        K: Fn(Item) -> ConfigParamKey,
        Builder: Fn(Item, JobVariant) -> F,
        F: Fn(JobContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<R, JobError>> + Send + 'static,
        R: Debug + Send + Sync + 'static,
    {
        self.build_jobs(job, items, move |item| Some(config_key(item)), build_job)
    }

    fn build_jobs<Items, Item, P, Builder, F, Fut, R>(mut self, job: WorkerJob, items: Items, interval_param: P, build_job: Builder) -> Self
    where
        Items: IntoIterator<Item = Item>,
        Item: JobLabel + Clone + Send + Sync + 'static,
        P: Fn(Item) -> Option<ConfigParamKey>,
        Builder: Fn(Item, JobVariant) -> F,
        F: Fn(JobContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<R, JobError>> + Send + 'static,
        R: Debug + Send + Sync + 'static,
    {
        if self.plan.is_ok() && job.worker() != self.worker {
            self.plan = Err(format!("job {} belongs to {:?} worker but builder is {:?}", job.as_ref(), job.worker(), self.worker).into());
        }
        if self.plan.is_err() {
            return self;
        }
        for item in items {
            let variant = JobVariant::labeled(job, item.job_label());
            if !should_include(&variant.name(), self.filter.as_deref()) {
                continue;
            }
            let job_fn = build_job(item.clone(), variant.clone());
            self.pending.push(Self::pending_job(variant, interval_param(item), job_fn));
        }
        self
    }

    fn pending_job<F, Fut, R>(variant: JobVariant, interval_param: Option<ConfigParamKey>, job_fn: F) -> PendingJob
    where
        F: Fn(JobContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<R, JobError>> + Send + 'static,
        R: Debug + Send + Sync + 'static,
    {
        let name = variant.name();
        PendingJob {
            variant,
            interval_param,
            add: Box::new(move |plan, interval| plan.job(name, interval, job_fn)),
        }
    }

    pub async fn finish(self) -> Result<Vec<JobHandle>, Box<dyn Error + Send + Sync>> {
        let mut plan = self.plan?;
        for pending in self.pending {
            let variant = match pending.interval_param {
                Some(param) => pending.variant.with_param_duration(self.config.ok_or("ConfigCacher required for jobs_with_config")?, &param).await?,
                None => pending.variant,
            };
            plan = (pending.add)(plan, variant.resolve_interval(self.config).await?);
        }
        Ok(plan.finish())
    }
}

fn should_include(name: &str, filter: Option<&str>) -> bool {
    filter.is_none_or(|f| f.is_empty() || name.contains(f))
}

#[cfg(test)]
mod tests {
    use super::should_include;

    #[test]
    fn test_should_include() {
        assert!(should_include("publish_missing_prices", None));
        assert!(should_include("publish_missing_prices", Some("")));
        assert!(should_include("publish_missing_prices", Some("missing")));
        assert!(should_include("update_prices_top.coingecko", Some("coingecko")));
        assert!(!should_include("update_prices_top.coingecko", Some("pyth")));
        assert!(!should_include("publish_missing_prices", Some("update_prices")));
    }
}
