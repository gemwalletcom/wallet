use std::{error::Error, future::Future, pin::Pin, time::Duration};

pub type AccessTokenFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, Box<dyn Error + Send + Sync>>> + Send + 'a>>;

pub trait AccessTokenCacher: Send + Sync {
    fn get_or_refresh<'a>(&'a self, refresh: AccessTokenFuture<'a, (String, Duration)>) -> AccessTokenFuture<'a, String>;
}
