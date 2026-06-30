use super::{AlienError, AlienProvider, AlienResponse, AlienTarget};

use async_trait::async_trait;
use futures::channel::oneshot;
use primitives::Chain;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, MutexGuard},
};

type RequestWaiters = Vec<oneshot::Sender<Result<AlienResponse, AlienError>>>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct RequestKey {
    method: String,
    url: String,
    body: Option<Vec<u8>>,
}

impl RequestKey {
    fn new(target: &AlienTarget) -> Self {
        Self {
            method: String::from(target.method),
            url: target.url.clone(),
            body: target.body.clone(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct CoalescingAlienProvider {
    provider: Arc<dyn AlienProvider>,
    requests: Mutex<HashMap<RequestKey, RequestWaiters>>,
}

impl CoalescingAlienProvider {
    pub(crate) fn new(provider: Arc<dyn AlienProvider>) -> Self {
        Self {
            provider,
            requests: Mutex::new(HashMap::new()),
        }
    }
}

struct RequestGuard<'a> {
    requests: &'a Mutex<HashMap<RequestKey, RequestWaiters>>,
    key: RequestKey,
    active: bool,
}

impl<'a> RequestGuard<'a> {
    fn new(requests: &'a Mutex<HashMap<RequestKey, RequestWaiters>>, key: RequestKey) -> Self {
        Self { requests, key, active: true }
    }

    fn finish(&mut self) -> RequestWaiters {
        self.active = false;
        requests(self.requests).remove(&self.key).unwrap_or_default()
    }
}

impl Drop for RequestGuard<'_> {
    fn drop(&mut self) {
        if self.active {
            for waiter in requests(self.requests).remove(&self.key).unwrap_or_default() {
                let _ = waiter.send(Err(AlienError::request_error("Coalesced request was canceled")));
            }
        }
    }
}

fn requests(requests: &Mutex<HashMap<RequestKey, RequestWaiters>>) -> MutexGuard<'_, HashMap<RequestKey, RequestWaiters>> {
    match requests.lock() {
        Ok(requests) => requests,
        Err(poisoned) => poisoned.into_inner(),
    }
}

#[async_trait]
impl AlienProvider for CoalescingAlienProvider {
    async fn request(&self, target: AlienTarget) -> Result<AlienResponse, AlienError> {
        let key = RequestKey::new(&target);
        let receiver = {
            let mut requests = requests(&self.requests);
            if let Some(waiters) = requests.get_mut(&key) {
                let (sender, receiver) = oneshot::channel();
                waiters.push(sender);
                Some(receiver)
            } else {
                requests.insert(key.clone(), Vec::new());
                None
            }
        };

        if let Some(receiver) = receiver {
            return receiver.await.unwrap_or_else(|_| Err(AlienError::request_error("Coalesced request was canceled")));
        }

        let mut guard = RequestGuard::new(&self.requests, key);
        let result = self.provider.request(target).await;
        for waiter in guard.finish() {
            let _ = waiter.send(result.clone());
        }
        result
    }

    fn get_endpoint(&self, chain: Chain) -> Result<String, AlienError> {
        self.provider.get_endpoint(chain)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alien::AlienHttpMethod;
    use gem_client::{CONTENT_TYPE, ContentType};
    use std::{
        future::Future,
        sync::{
            Barrier,
            atomic::{AtomicUsize, Ordering},
        },
        task::{Context, Poll},
        thread,
        time::Duration,
    };

    #[derive(Debug)]
    struct MockProvider {
        count: AtomicUsize,
    }

    #[derive(Debug)]
    struct CancelProvider {
        count: AtomicUsize,
    }

    #[async_trait]
    impl AlienProvider for MockProvider {
        async fn request(&self, _target: AlienTarget) -> Result<AlienResponse, AlienError> {
            self.count.fetch_add(1, Ordering::SeqCst);
            thread::sleep(Duration::from_millis(100));
            Ok(AlienResponse {
                status: Some(200),
                data: b"{}".to_vec(),
            })
        }

        fn get_endpoint(&self, _chain: Chain) -> Result<String, AlienError> {
            Ok("https://example.com".to_string())
        }
    }

    #[async_trait]
    impl AlienProvider for CancelProvider {
        async fn request(&self, _target: AlienTarget) -> Result<AlienResponse, AlienError> {
            let count = self.count.fetch_add(1, Ordering::SeqCst);
            if count == 0 {
                futures::future::pending().await
            } else {
                Ok(AlienResponse {
                    status: Some(200),
                    data: b"{}".to_vec(),
                })
            }
        }

        fn get_endpoint(&self, _chain: Chain) -> Result<String, AlienError> {
            Ok("https://example.com".to_string())
        }
    }

    fn target(request_type: &str) -> AlienTarget {
        AlienTarget {
            url: "https://example.com/info".to_string(),
            method: AlienHttpMethod::Post,
            headers: Some(HashMap::from([(CONTENT_TYPE.to_string(), ContentType::ApplicationJson.as_str().to_string())])),
            body: Some(serde_json::to_vec(&serde_json::json!({ "type": request_type })).unwrap()),
        }
    }

    fn spawn_request(provider: Arc<CoalescingAlienProvider>, barrier: Arc<Barrier>, request_type: &'static str) -> thread::JoinHandle<AlienResponse> {
        thread::spawn(move || {
            barrier.wait();
            futures::executor::block_on(provider.request(target(request_type))).unwrap()
        })
    }

    fn concurrent_request_count(first_type: &'static str, second_type: &'static str) -> usize {
        let mock = Arc::new(MockProvider { count: AtomicUsize::new(0) });
        let provider = Arc::new(CoalescingAlienProvider::new(mock.clone()));
        let barrier = Arc::new(Barrier::new(3));
        let first = spawn_request(provider.clone(), barrier.clone(), first_type);
        let second = spawn_request(provider, barrier.clone(), second_type);
        barrier.wait();

        first.join().unwrap();
        second.join().unwrap();

        mock.count.load(Ordering::SeqCst)
    }

    #[test]
    fn test_request_coalescing() {
        assert_eq!(concurrent_request_count("spotClearinghouseState", "spotClearinghouseState"), 1);
        assert_eq!(concurrent_request_count("spotClearinghouseState", "delegatorSummary"), 2);
    }

    #[test]
    fn test_canceled_leader_clears_request() {
        let mock = Arc::new(CancelProvider { count: AtomicUsize::new(0) });
        let provider = Arc::new(CoalescingAlienProvider::new(mock.clone()));
        let mut request = provider.request(target("spotClearinghouseState"));
        let waker = futures::task::noop_waker();
        let mut context = Context::from_waker(&waker);

        assert!(matches!(Future::poll(request.as_mut(), &mut context), Poll::Pending));
        drop(request);

        futures::executor::block_on(provider.request(target("spotClearinghouseState"))).unwrap();

        assert_eq!(mock.count.load(Ordering::SeqCst), 2);
    }
}
