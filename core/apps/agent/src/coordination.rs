use std::collections::HashMap;
use std::future::Future;
use std::sync::Mutex;

use gem_tracing::tracing::debug;
use tokio::sync::watch;

#[derive(Default)]
pub struct ConversationJobs {
    jobs: Mutex<HashMap<String, watch::Sender<bool>>>,
}

impl ConversationJobs {
    pub async fn run<F>(&self, key: &str, work: F) -> crate::Result<()>
    where
        F: Future<Output = crate::Result<()>>,
    {
        let mut superseded = self.begin(key);
        tokio::select! {
            _ = superseded.changed() => {
                debug!(conversation = %key, "superseded by a newer message; cancelling this job");
                Ok(())
            }
            result = work => result,
        }
    }

    fn begin(&self, key: &str) -> watch::Receiver<bool> {
        let (sender, receiver) = watch::channel(false);
        let mut jobs = self.jobs.lock().expect("conversation jobs poisoned");
        if let Some(previous) = jobs.insert(key.to_string(), sender) {
            let _ = previous.send(true);
        }
        receiver
    }
}
