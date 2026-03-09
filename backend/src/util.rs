use std::time::{SystemTime, UNIX_EPOCH};

pub fn now_u64() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Do you have a time machine?")
        .as_millis() as u64
}
