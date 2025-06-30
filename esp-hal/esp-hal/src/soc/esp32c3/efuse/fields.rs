// 8
use super::EfuseField;

/// MAC address
// 132
pub const MAC0: EfuseField = EfuseField::new(1, 0, 0, 32);
/// MAC address
// 134
pub const MAC1: EfuseField = EfuseField::new(1, 1, 32, 16);
