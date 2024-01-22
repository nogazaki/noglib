//! Frequently used macros

/// Choose
macro_rules! choose {
    ($x:expr, $y:expr, $z:expr) => {
        ($x & $y) ^ (!$x & $z)
    };
}
pub(crate) use choose;

/// Majority
macro_rules! majority {
    ($x:expr, $y:expr, $z:expr) => {
        ($x & $y) ^ ($x & $z) ^ ($y & $z)
    };
}
pub(crate) use majority;
