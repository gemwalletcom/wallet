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

pub async fn record_both<T, A, B>(failures: &mut Vec<T>, first: (T::Step, A), second: (T::Step, B))
where
    T: StepFailure,
    A: Future<Output = Result<(), GemServiceError>>,
    B: Future<Output = Result<(), GemServiceError>>,
{
    let (first_result, second_result) = futures::join!(first.1, second.1);
    record_result(failures, first.0, first_result);
    record_result(failures, second.0, second_result);
}

fn record_result<T>(failures: &mut Vec<T>, step: T::Step, result: Result<(), GemServiceError>)
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
    fn test_record_both_runs_the_steps_together_and_keeps_their_order() {
        let (started, wait) = futures::channel::oneshot::channel::<()>();
        let failures = futures::executor::block_on(async {
            let mut failures: Vec<Failure> = Vec::new();
            record_both(
                &mut failures,
                (1u8, async {
                    wait.await.expect("the slow step starts before the fast one finishes");
                    Err(GemServiceError::Gateway { msg: "offline".to_string() })
                }),
                (2u8, async {
                    started.send(()).expect("the fast step runs while the slow one waits");
                    Err(GemServiceError::Cancelled)
                }),
            )
            .await;
            failures
        });

        assert_eq!(
            failures,
            vec![
                Failure {
                    step: 1,
                    message: "offline".to_string()
                },
                Failure {
                    step: 2,
                    message: "cancelled".to_string()
                },
            ],
            "a failing step neither cancels the other nor changes the reported order"
        );
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

        assert_eq!(
            failures,
            vec![
                Failure {
                    step: 1,
                    message: "offline".to_string()
                },
                Failure {
                    step: 3,
                    message: "cancelled".to_string()
                },
            ]
        );
    }
}
