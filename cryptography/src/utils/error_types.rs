//! Define various error types for various operation

/// Error type returns when output buffer is not large enough for data
#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct InsufficientMemoryError;
