use std::future::Future;
use std::time::Duration;

use reqwest::{StatusCode, retry};

#[cfg(feature = "reqwest")]
use tokio::time::sleep;

pub fn retry_policy<S>(host: S, max_retries: u32) -> retry::Builder
where
    S: for<'a> PartialEq<&'a str> + Send + Sync + 'static,
{
    retry::for_host(host).max_retries_per_request(max_retries).classify_fn(|req_rep| {
        match req_rep.status() {
            Some(StatusCode::TOO_MANY_REQUESTS) | Some(StatusCode::INTERNAL_SERVER_ERROR) | Some(StatusCode::BAD_GATEWAY) | Some(StatusCode::SERVICE_UNAVAILABLE) | Some(StatusCode::GATEWAY_TIMEOUT) => req_rep.retryable(),
            None => req_rep.retryable(), // Network errors
            _ => req_rep.success(),
        }
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
            Err(err) => {
                if attempt < max_retries && should_retry(&err) {
                    attempt += 1;
                    // Exponential backoff: 2^attempt seconds (2s, 4s, 8s, ...) with max cap
                    let delay = Duration::from_secs(2_u64.saturating_pow(attempt).min(1800)); // Cap at 30 minutes

                    #[cfg(feature = "reqwest")]
                    sleep(delay).await;

                    #[cfg(not(feature = "reqwest"))]
                    std::thread::sleep(delay);

                    continue;
                }

                return Err(err);
            }
        }
    }
}

/// Default retry predicate for clearly transient errors
///
/// Retries on:
/// - 401 (Unauthorized - some APIs use this for rate limits)
/// - 429 (Too Many Requests)
/// - 502 (Bad Gateway)
/// - 503 (Service Unavailable)
/// - 504 (Gateway Timeout)
/// - "too many requests", "throttled", and "limited" messages
pub fn default_should_retry<E: std::fmt::Display>(error: &E) -> bool {
    let error_str = error.to_string().to_lowercase();

    error_str.contains("401") ||                    // Unauthorized (rate limit on some APIs)
    error_str.contains("429") ||                    // Too Many Requests
    error_str.contains("502") ||                    // Bad Gateway
    error_str.contains("503") ||                    // Service Unavailable
    error_str.contains("504") ||                    // Gateway Timeout
    error_str.contains("too many requests") ||      // Rate limiting messages
    error_str.contains("throttled") ||              // Throttling messages
    error_str.contains("request is limited") // CoinGecko rate limit message
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::retry;

    #[tokio::test]
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
