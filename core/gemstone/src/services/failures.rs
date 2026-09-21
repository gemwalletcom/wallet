use std::future::Future;

use crate::services::error::GemServiceError;

pub trait StepFailure {
    type Step;

    fn new(step: Self::Step, message: String) -> Self;
}

pub async fn record<T, F>(failures: &mut Vec<T>, step: T::Step, future: F)
where
    T: StepFailure,
    F: Future<Output = Result<(), GemServiceError>>,
{
    record_result(failures, step, future.await);
}

pub fn record_result<T>(failures: &mut Vec<T>, step: T::Step, result: Result<(), GemServiceError>)
where
    T: StepFailure,
{
    if let Err(error) = result {
        failures.push(T::new(step, error.to_string()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct Failure {
        step: u8,
        message: String,
    }

    impl StepFailure for Failure {
        type Step = u8;

        fn new(step: u8, message: String) -> Self {
            Self { step, message }
        }
    }

    #[test]
    fn test_record_collects_failures_and_continues() {
        let failures = futures::executor::block_on(async {
            let mut failures: Vec<Failure> = Vec::new();
            record(&mut failures, 1, async { Err(GemServiceError::Gateway { msg: "offline".to_string() }) }).await;
            record(&mut failures, 2, async { Ok(()) }).await;
            record(&mut failures, 3, async { Err(GemServiceError::Cancelled) }).await;
            failures
        });

        assert_eq!(failures, vec![Failure { step: 1, message: "offline".to_string() }, Failure { step: 3, message: "cancelled".to_string() },]);
    }
}
