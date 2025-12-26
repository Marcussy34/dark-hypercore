//! Core data types for Dark HyperCore
//!
//! All types implement SSZ serialization for deterministic encoding.
//! All numeric values use fixed-point representation (scaled by 10^8).
//!
//! ## Types
//!
//! - [`Order`]: A limit order in the order book
//! - [`Side`]: Buy or Sell
//! - [`OrderType`]: Type of order (Limit only in Phase 1)
//! - [`Trade`]: An executed trade between two orders
//! - [`ExecutionReceipt`]: Batch execution summary
//!
//! ## Fixed-Point Arithmetic
//!
//! All prices and quantities are stored as `u64` scaled by 10^8.
//! Example: 50000.12345678 is stored as 5_000_012_345_678u64

mod order;
mod trade;
mod receipt;
pub mod price;

// Re-export all types at module level
pub use order::{Order, Side, OrderType};
pub use trade::Trade;
pub use receipt::ExecutionReceipt;

