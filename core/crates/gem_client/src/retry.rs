use std::future::Future;
use std::time::Duration;

use reqwest::{StatusCode, retry};

#[cfg(feature = "reqwest")]
use tokio::time::sleep;

pub fn retry_policy<S>(host: S, max_retries: u32) -> retry::Builder
where
    S: for<'a> PartialEq<&'a str> + Send + Sync + 'static,
{
    retry::for_host(host).max_retries_per_request(max_retries).classify_fn(|req_rep| match req_rep.status() {
        Some(StatusCode::TOO_MANY_REQUESTS) | Some(StatusCode::INTERNAL_SERVER_ERROR) | Some(StatusCode::BAD_GATEWAY) | Some(StatusCode::SERVICE_UNAVAILABLE) | Some(StatusCode::GATEWAY_TIMEOUT) => req_rep.retryable(),
        None => req_rep.retryable(),
        _ => req_rep.success(),
    })
}

pub async fn retry<T, E, F, Fut, P>(operation: F, max_retries: u32, should_retry: P) -> Result<T, E>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    P: Fn(&E) -> bool,
{
    let mut attempt = 0;

    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(error) => {
                if attempt < max_retries && should_retry(&error) {
                    attempt += 1;
                    let delay = Duration::from_secs(2_u64.saturating_pow(attempt).min(1800));

                    #[cfg(feature = "reqwest")]
                    sleep(delay).await;

                    #[cfg(not(feature = "reqwest"))]
                    std::thread::sleep(delay);

                    continue;
                }

                return Err(error);
            }
        }
    }
}

pub fn default_should_retry<E: std::fmt::Display>(error: &E) -> bool {
    let error_str = error.to_string().to_lowercase();

    error_str.contains("401")
        || error_str.contains("429")
        || error_str.contains("502")
        || error_str.contains("503")
        || error_str.contains("504")
        || error_str.contains("too many requests")
        || error_str.contains("throttled")
        || error_str.contains("request is limited")
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::retry;

    #[tokio::test(start_paused = true)]
    async fn test_retry_respects_predicate_and_limit() {
        for (error, expected_attempts) in [(7, 2), (8, 1)] {
            let attempts = AtomicUsize::new(0);
            let result: Result<(), u8> = retry(
                || async {
                    attempts.fetch_add(1, Ordering::SeqCst);
                    Err(error)
                },
                1,
                |error| *error == 7,
            )
            .await;
            assert_eq!(result, Err(error));
            assert_eq!(attempts.load(Ordering::SeqCst), expected_attempts);
        }
    }
}
