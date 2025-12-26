//! Order book module for the Dark HyperCore matching engine.
//!
//! ## Architecture
//!
//! The order book is implemented as a Central Limit Order Book (CLOB) with:
//!
//! - **Slab-based storage**: O(1) order insertion, removal, and lookup
//! - **Price levels**: Orders grouped by price using BTreeMap
//! - **Price-time priority**: FIFO ordering at each price level
//!
//! ## Components
//!
//! - [`OrderNode`]: Wrapper around `Order` with linked-list pointers for price level
//! - [`PriceLevel`]: Collection of orders at a single price point
//! - [`CLOB`]: Main order book with bid/ask sides
//!
//! ## Performance
//!
//! | Operation | Complexity |
//! |-----------|------------|
//! | Add order | O(log n) |
//! | Remove order by key | O(1) |
//! | Cancel order by ID | O(1) |
//! | Best bid/ask | O(1)* |
//! | Match order | O(k log n) |
//!
//! *After initial lookup, cached at price level head
//!
//! ## Example
//!
//! ```
//! use dark_hypercore::orderbook::CLOB;
//! use dark_hypercore::types::{Order, Side};
//!
//! let mut clob = CLOB::with_capacity(10_000);
//!
//! // Add a buy order at $50,000
//! let order = Order::new(1, 100, Side::Buy, 5_000_000_000_000, 100_000_000, 0);
//! clob.add_order(order);
//!
//! assert_eq!(clob.best_bid(), Some(5_000_000_000_000));
//! ```

pub mod node;
pub mod level;
pub mod clob;

pub use node::OrderNode;
pub use level::PriceLevel;
pub use clob::CLOB;

