//! # Dark HyperCore
//!
//! High-frequency Layer 1 blockchain with TEE-native privacy.
//!
//! ## Architecture
//!
//! The Dark Kernel consists of:
//! - **Types**: Core data structures (Order, Trade, ExecutionReceipt)
//! - **OrderBook**: CLOB with slab-based memory allocation
//! - **Engine**: Deterministic matching engine
//!
//! ## Design Principles
//!
//! 1. **Determinism**: All operations produce identical results for identical inputs
//! 2. **No Floating Point**: All math uses fixed-point arithmetic (10^8 scaling)
//! 3. **Pre-allocated Memory**: Slab allocation for O(1) order operations
//! 4. **Synchronous Execution**: No async in hot path for maximum throughput
//!
//! ## Performance Targets
//!
//! - Throughput: >100,000 orders/second
//! - Latency: <10μs per match operation
//! - Memory: <200 bytes per order

// ============================================================================
// Module declarations
// ============================================================================

/// Core data types: Order, Trade, ExecutionReceipt
pub mod types;

/// Order book: CLOB with slab-based storage
pub mod orderbook;

/// Matching engine: Deterministic order matching
pub mod engine;

// ============================================================================
// Re-exports for convenience
// ============================================================================

pub use types::{Order, OrderType, Side, Trade, ExecutionReceipt};
pub use orderbook::{CLOB, OrderNode, PriceLevel};
pub use engine::{MatchingEngine, MatchResult};

