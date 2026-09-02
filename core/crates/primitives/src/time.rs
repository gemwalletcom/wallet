use std::time::{SystemTime, SystemTimeError, UNIX_EPOCH};

pub fn unix_seconds() -> Result<u64, SystemTimeError> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())
}

pub fn unix_milliseconds() -> Result<u64, SystemTimeError> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as u64)
}

pub fn unix_timestamp() -> u64 {
    unix_seconds().expect("Time went backwards")
}
