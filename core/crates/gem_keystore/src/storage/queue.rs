use std::sync::{Mutex, MutexGuard, OnceLock};

use crate::KeystoreError;

static QUEUE: OnceLock<Mutex<()>> = OnceLock::new();

pub(super) fn lock() -> Result<MutexGuard<'static, ()>, KeystoreError> {
    Ok(QUEUE.get_or_init(|| Mutex::new(())).lock().unwrap_or_else(std::sync::PoisonError::into_inner))
}
