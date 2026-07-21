use std::future::Future;

pub async fn try_in_order<I, F, T, E>(operations: I) -> Result<Option<T>, E>
where
    I: IntoIterator<Item = F>,
    F: Future<Output = Result<T, E>>,
{
    let mut last_error = None;
    for operation in operations {
        match operation.await {
            Ok(value) => return Ok(Some(value)),
            Err(error) => last_error = Some(error),
        }
    }
    match last_error {
        Some(error) => Err(error),
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use std::future::{Future, Ready, ready};
    use std::pin::pin;
    use std::task::{Context, Poll, Waker};

    use super::*;

    fn poll_ready<F: Future>(future: F) -> F::Output {
        let mut future = pin!(future);
        match future.as_mut().poll(&mut Context::from_waker(Waker::noop())) {
            Poll::Ready(output) => output,
            Poll::Pending => panic!("future should be ready"),
        }
    }

    #[test]
    fn test_try_in_order() {
        assert_eq!(poll_ready(try_in_order([ready(Err("first")), ready(Ok(42)), ready(Ok(43))])), Ok(Some(42)));
        assert_eq!(poll_ready(try_in_order([ready(Err::<i32, _>("first")), ready(Err("last"))])), Err("last"));
        assert_eq!(poll_ready(try_in_order(Vec::<Ready<Result<i32, &str>>>::new())), Ok(None));
    }
}
