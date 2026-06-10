use std::sync::{Mutex, MutexGuard, OnceLock};

use crate::KeystoreError;

static QUEUE: OnceLock<Mutex<()>> = OnceLock::new();

pub(super) fn lock() -> Result<MutexGuard<'static, ()>, KeystoreError> {
    QUEUE.get_or_init(|| Mutex::new(())).lock().map_err(|_| KeystoreError::io("keystore queue poisoned"))
}
