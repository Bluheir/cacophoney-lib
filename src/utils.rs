use std::time::{SystemTime, UNIX_EPOCH};

/// Gets the current time as millseconds since January 1 1970.
pub fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64
}