// 64
use crate::sync::RawMutex;

// 74
pub(super) static GPIO_LOCK: RawMutex = RawMutex::new();
