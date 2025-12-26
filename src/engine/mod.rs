//! Matching engine module for Dark HyperCore.
//!
//! ## Design Principles
//!
//! The matching engine is designed for:
//!
//! 1. **Determinism**: Same input always produces same output
//! 2. **Fixed-Point Math**: No floating-point operations
//! 3. **Synchronous Execution**: No async/await in hot path
//! 4. **Price-Time Priority**: Best price first, then FIFO
//!
//! ## Matching Rules
//!
//! - **Buy orders** match against asks (lowest price first)
//! - **Sell orders** match against bids (highest price first)
//! - **Partial fills** are supported
//! - **Unfilled quantity** rests on the book
//!
//! ## Example
//!
//! ```
//! use dark_hypercore::engine::MatchingEngine;
//! use dark_hypercore::orderbook::CLOB;
//! use dark_hypercore::types::{Order, Side};
//!
//! let mut clob = CLOB::with_capacity(1000);
//! let mut engine = MatchingEngine::new();
//!
//! // Add resting sell order
//! let sell = Order::new(1, 100, Side::Sell, 5_000_000_000_000, 100_000_000, 0);
//! clob.add_order(sell);
//!
//! // Incoming buy order should match
//! let buy = Order::new(2, 101, Side::Buy, 5_000_000_000_000, 100_000_000, 0);
//! let result = engine.match_order(&mut clob, buy, 1000);
//!
//! assert!(result.fully_filled);
//! assert_eq!(result.trades.len(), 1);
//! ```

pub mod matcher;

pub use matcher::{MatchingEngine, MatchResult};

