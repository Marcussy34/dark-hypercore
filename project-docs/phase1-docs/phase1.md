# Phase 1: The Dark Kernel - Implementation Plan

**Project:** Dark HyperCore  
**Phase:** 1 - The Dark Kernel  
**Status:** In Progress  
**Created:** December 26, 2025  
**Last Updated:** December 26, 2025

---

## Overview

Phase 1 focuses on building the **single-threaded deterministic matching engine** - the core of Dark HyperCore's CLOB (Central Limit Order Book). This phase establishes the performance baseline in standard userspace before TEE integration in Phase 2.

### Why Userspace First?
- Establish a raw performance baseline (target: >100k TPS on raw CPU)
- Isolate matching engine logic from TEE overhead
- Critical for measuring TEE performance cost in Phase 2
- Faster iteration and debugging in standard environment

---

## Technical Stack (Phase 1)

| Component | Technology | Version | Purpose |
|-----------|------------|---------|---------|
| Language | Rust | Stable (latest) | Core implementation |
| Serialization | `ssz_rs` | 0.9.0 | Ethereum SSZ format for consensus-ready data |
| Memory | `slab` | 0.4.11 | Pre-allocated O(1) order storage |
| Math | `rust_decimal` | Latest | Fixed-point arithmetic (no floating point) |

### Library References

**ssz_rs** (https://docs.rs/ssz_rs)
- Ethereum's Simple Serialize scheme
- Deterministic serialization for consensus
- Derive macro: `#[derive(SimpleSerialize)]`

**slab** (https://docs.rs/slab)
- Pre-allocated storage for uniform data types
- O(1) insert, remove, and access
- `Slab::with_capacity(n)` for pre-allocation

**rust_decimal** (https://docs.rs/rust_decimal)
- 128-bit decimal numbers
- `checked_*` methods for overflow protection
- No floating-point errors

---

## Phase 1 Sub-Phases

Phase 1 is divided into **5 sequential sub-phases**. Each must be completed and verified before proceeding.

---

## Sub-Phase 1.1: Project Setup & Dependencies

### Objective
Initialize the Rust project with all required dependencies and establish project structure.

### Scope
- Create Cargo project with workspace structure
- Configure dependencies with exact versions
- Set up basic project layout
- Configure compiler settings for performance

### Deliverables

| Deliverable | Description | File(s) |
|-------------|-------------|---------|
| Cargo.toml | Project configuration with dependencies | `/Cargo.toml` |
| Project structure | Module organization | `/src/lib.rs`, `/src/main.rs` |
| Build configuration | Release optimization flags | `.cargo/config.toml` |
| README update | Build instructions | `/README.md` |

### Implementation Details

```toml
# Cargo.toml
[package]
name = "dark-hypercore"
version = "0.1.0"
edition = "2021"

[dependencies]
# SSZ serialization - Ethereum consensus format
ssz_rs = "0.9.0"

# Pre-allocated order storage - O(1) operations
slab = "0.4.11"

# Fixed-point decimal math - no floating point
rust_decimal = "1.33"
rust_decimal_macros = "1.33"

# Additional utilities
thiserror = "1.0"    # Error handling
sha2 = "0.10"        # Hashing for state roots

[dev-dependencies]
criterion = "0.5"    # Benchmarking
rand = "0.8"         # Test data generation

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
```

### Test/Acceptance Criteria
- [ ] `cargo build` completes without errors
- [ ] `cargo test` runs (empty tests pass)
- [ ] All dependencies resolve correctly
- [ ] Project compiles in release mode with optimizations

### Owner
Lead Developer

### Estimated Time
1 day

---

## Sub-Phase 1.2: Core Data Structures

### Objective
Implement the fundamental data structures with SSZ serialization support.

### Scope
- Define `Order` struct with all required fields
- Define `Trade` struct for execution records
- Define `ExecutionReceipt` for batch results
- Define `OrderNode` for slab storage
- Implement SSZ serialization for all types
- Implement fixed-point price/quantity types

### Deliverables

| Deliverable | Description | File(s) |
|-------------|-------------|---------|
| Order types | Order, Side, OrderType enums | `/src/types/order.rs` |
| Trade type | Trade execution record | `/src/types/trade.rs` |
| Receipt type | Execution batch receipt | `/src/types/receipt.rs` |
| Price type | Fixed-point price wrapper | `/src/types/price.rs` |
| Module exports | Public API | `/src/types/mod.rs` |

### Data Structure Specifications

```rust
// Order Side - Buy or Sell
#[derive(Debug, Clone, Copy, PartialEq, Eq, SimpleSerialize)]
pub enum Side {
    Buy = 0,
    Sell = 1,
}

// Order Type - Limit orders only for Phase 1
#[derive(Debug, Clone, Copy, PartialEq, Eq, SimpleSerialize)]
pub enum OrderType {
    Limit = 0,
}

// Core Order struct
// Uses SSZ serialization for deterministic encoding
#[derive(Debug, Clone, PartialEq, Eq, SimpleSerialize)]
pub struct Order {
    /// Unique order identifier
    pub id: u64,
    
    /// User/account identifier
    pub user_id: u64,
    
    /// Buy or Sell
    pub side: Side,
    
    /// Price in fixed-point (scaled by 10^8)
    /// Example: 50000.00000000 BTC = 5_000_000_000_000u64
    pub price: u64,
    
    /// Quantity in fixed-point (scaled by 10^8)
    pub quantity: u64,
    
    /// Remaining quantity (for partial fills)
    pub remaining: u64,
    
    /// Unix timestamp in milliseconds
    pub timestamp: u64,
    
    /// Order type (Limit only in Phase 1)
    pub order_type: OrderType,
}

// Trade execution record
#[derive(Debug, Clone, PartialEq, Eq, SimpleSerialize)]
pub struct Trade {
    /// Unique trade identifier
    pub id: u64,
    
    /// Maker order ID (resting order)
    pub maker_order_id: u64,
    
    /// Taker order ID (incoming order)
    pub taker_order_id: u64,
    
    /// Execution price
    pub price: u64,
    
    /// Executed quantity
    pub quantity: u64,
    
    /// Execution timestamp
    pub timestamp: u64,
}

// Batch execution receipt
#[derive(Debug, Clone, PartialEq, Eq, SimpleSerialize)]
pub struct ExecutionReceipt {
    /// Batch sequence number
    pub batch_id: u64,
    
    /// Number of orders processed
    pub orders_processed: u64,
    
    /// Number of trades executed
    pub trades_executed: u64,
    
    /// State root after execution (32-byte hash)
    pub state_root: [u8; 32],
    
    /// Batch timestamp
    pub timestamp: u64,
}
```

### SSZ Serialization Notes

Per official SSZ spec (ethereum.org):
- **Basic types** (u64, bool): Direct little-endian encoding
- **Fixed-size composites**: Concatenated little-endian fields
- **Variable-length types**: Offset values in fixed section, data in heap
- All serialization is **deterministic** - same input always produces same output

### Test/Acceptance Criteria
- [ ] All structs compile with `#[derive(SimpleSerialize)]`
- [ ] Serialization round-trip test passes (serialize → deserialize → compare)
- [ ] `Order` serializes to expected byte length
- [ ] `Trade` serializes to expected byte length
- [ ] `ExecutionReceipt` state_root is exactly 32 bytes
- [ ] All fixed-point conversions are correct (10^8 scaling)

### Unit Tests Required

```rust
#[test]
fn test_order_ssz_roundtrip() {
    let order = Order {
        id: 1,
        user_id: 100,
        side: Side::Buy,
        price: 5_000_000_000_000, // 50000.00000000
        quantity: 100_000_000,    // 1.00000000
        remaining: 100_000_000,
        timestamp: 1703577600000,
        order_type: OrderType::Limit,
    };
    
    let serialized = ssz_rs::serialize(&order).unwrap();
    let deserialized: Order = ssz_rs::deserialize(&serialized).unwrap();
    
    assert_eq!(order, deserialized);
}

#[test]
fn test_order_deterministic_serialization() {
    // Same order should always produce identical bytes
    let order = create_test_order();
    let bytes1 = ssz_rs::serialize(&order).unwrap();
    let bytes2 = ssz_rs::serialize(&order).unwrap();
    assert_eq!(bytes1, bytes2);
}
```

### Owner
Lead Developer

### Estimated Time
2 days

---

## Sub-Phase 1.3: Order Book Data Structure (CLOB)

### Objective
Implement the Central Limit Order Book using slab-based memory allocation.

### Scope
- Implement `OrderNode` for slab storage
- Implement price-level organization (price → orders mapping)
- Implement `CLOB` struct with bid/ask sides
- Implement O(1) order insertion and removal
- Implement price-time priority ordering

### Deliverables

| Deliverable | Description | File(s) |
|-------------|-------------|---------|
| OrderNode | Slab-compatible order wrapper | `/src/orderbook/node.rs` |
| PriceLevel | Orders at a single price | `/src/orderbook/level.rs` |
| CLOB | Main order book structure | `/src/orderbook/clob.rs` |
| Module exports | Public API | `/src/orderbook/mod.rs` |

### Data Structure Specifications

```rust
use slab::Slab;
use std::collections::BTreeMap;

/// Order node stored in the slab
/// Contains order data plus linked-list pointers for price level
#[derive(Debug, Clone)]
pub struct OrderNode {
    /// The actual order
    pub order: Order,
    
    /// Next order in price level (slab key, None if tail)
    pub next: Option<usize>,
    
    /// Previous order in price level (slab key, None if head)
    pub prev: Option<usize>,
}

/// Orders at a single price level
/// Uses doubly-linked list for O(1) removal
#[derive(Debug)]
pub struct PriceLevel {
    /// Price for this level
    pub price: u64,
    
    /// Total quantity at this level
    pub total_quantity: u64,
    
    /// Head of the order queue (slab key)
    pub head: Option<usize>,
    
    /// Tail of the order queue (slab key)
    pub tail: Option<usize>,
    
    /// Number of orders at this level
    pub order_count: usize,
}

/// Central Limit Order Book
pub struct CLOB {
    /// Pre-allocated order storage
    /// Key: slab index, Value: OrderNode
    orders: Slab<OrderNode>,
    
    /// Bid price levels (sorted high to low)
    /// Key: price (negated for reverse sort), Value: PriceLevel
    bids: BTreeMap<std::cmp::Reverse<u64>, PriceLevel>,
    
    /// Ask price levels (sorted low to high)
    /// Key: price, Value: PriceLevel
    asks: BTreeMap<u64, PriceLevel>,
    
    /// Order ID to slab key mapping (for O(1) cancel)
    order_index: std::collections::HashMap<u64, usize>,
    
    /// Next order ID
    next_order_id: u64,
    
    /// Next trade ID
    next_trade_id: u64,
}
```

### Slab Usage Notes

Per official slab documentation (docs.rs/slab):
- `Slab::with_capacity(n)` - Pre-allocate n slots
- `slab.insert(value)` - Returns key, O(1)
- `slab.remove(key)` - Returns value, O(1)
- `slab.get(key)` / `slab.get_mut(key)` - O(1) access
- `slab.vacant_entry()` - Get entry before inserting

### CLOB Operations

```rust
impl CLOB {
    /// Create new CLOB with pre-allocated capacity
    pub fn with_capacity(order_capacity: usize) -> Self;
    
    /// Add order to the book (returns slab key)
    pub fn add_order(&mut self, order: Order) -> usize;
    
    /// Remove order by slab key
    pub fn remove_order(&mut self, key: usize) -> Option<Order>;
    
    /// Cancel order by order ID
    pub fn cancel_order(&mut self, order_id: u64) -> Option<Order>;
    
    /// Get best bid price
    pub fn best_bid(&self) -> Option<u64>;
    
    /// Get best ask price
    pub fn best_ask(&self) -> Option<u64>;
    
    /// Get spread (best_ask - best_bid)
    pub fn spread(&self) -> Option<u64>;
    
    /// Get order count
    pub fn order_count(&self) -> usize;
}
```

### Test/Acceptance Criteria
- [ ] CLOB initializes with pre-allocated capacity
- [ ] Order insertion is O(1) average case
- [ ] Order removal is O(1)
- [ ] Order cancel by ID is O(1)
- [ ] Best bid/ask retrieval is O(log n)
- [ ] Price-time priority is maintained (FIFO at same price)
- [ ] Bid side sorted high-to-low (best bid first)
- [ ] Ask side sorted low-to-high (best ask first)

### Unit Tests Required

```rust
#[test]
fn test_clob_add_order() {
    let mut clob = CLOB::with_capacity(1000);
    
    let order = Order::new_limit(Side::Buy, 50000_00000000, 1_00000000);
    let key = clob.add_order(order);
    
    assert!(clob.orders.contains(key));
    assert_eq!(clob.order_count(), 1);
}

#[test]
fn test_clob_price_priority() {
    let mut clob = CLOB::with_capacity(1000);
    
    // Add bids at different prices
    clob.add_order(Order::new_limit(Side::Buy, 49000_00000000, 1_00000000));
    clob.add_order(Order::new_limit(Side::Buy, 51000_00000000, 1_00000000));
    clob.add_order(Order::new_limit(Side::Buy, 50000_00000000, 1_00000000));
    
    // Best bid should be highest price
    assert_eq!(clob.best_bid(), Some(51000_00000000));
}

#[test]
fn test_clob_time_priority() {
    let mut clob = CLOB::with_capacity(1000);
    
    // Add orders at same price
    let key1 = clob.add_order(Order::new_limit(Side::Buy, 50000_00000000, 1_00000000));
    let key2 = clob.add_order(Order::new_limit(Side::Buy, 50000_00000000, 2_00000000));
    
    // First order should be at head of queue
    // (verified by matching behavior in next phase)
}
```

### Owner
Lead Developer

### Estimated Time
3 days

---

## Sub-Phase 1.4: Matching Engine

### Objective
Implement the synchronous, deterministic order matching algorithm.

### Scope
- Implement price-time priority matching
- Implement fixed-point arithmetic for quantity calculations
- Implement partial fill handling
- Implement trade generation
- Ensure 100% deterministic execution

### Deliverables

| Deliverable | Description | File(s) |
|-------------|-------------|---------|
| Matching engine | Core matching logic | `/src/engine/matcher.rs` |
| Trade collector | Trade aggregation | `/src/engine/trades.rs` |
| State calculator | State root computation | `/src/engine/state.rs` |
| Engine interface | Public API | `/src/engine/mod.rs` |

### Matching Algorithm

```rust
/// Match result for a single order
#[derive(Debug)]
pub struct MatchResult {
    /// Original order (may be partially filled)
    pub order: Order,
    
    /// Trades generated
    pub trades: Vec<Trade>,
    
    /// Whether order was fully filled
    pub fully_filled: bool,
    
    /// Remaining quantity (0 if fully filled)
    pub remaining: u64,
}

impl CLOB {
    /// Process incoming order and match against book
    /// 
    /// # Matching Rules
    /// 1. Buy orders match against asks (low to high)
    /// 2. Sell orders match against bids (high to low)
    /// 3. Price-time priority (FIFO at same price)
    /// 4. Partial fills allowed
    /// 5. Unfilled quantity rests on book
    /// 
    /// # Determinism
    /// - Uses fixed-point arithmetic only
    /// - No floating point operations
    /// - Same input always produces same output
    pub fn match_order(&mut self, incoming: Order) -> MatchResult {
        let mut trades = Vec::new();
        let mut remaining = incoming.remaining;
        
        // Determine which side to match against
        let opposite_levels = match incoming.side {
            Side::Buy => &mut self.asks,
            Side::Sell => &mut self.bids,
        };
        
        // Match while there are compatible prices
        while remaining > 0 {
            // Get best opposite price level
            let best_price = match incoming.side {
                Side::Buy => opposite_levels.keys().next().copied(),
                Side::Sell => opposite_levels.keys().next().map(|r| r.0),
            };
            
            let best_price = match best_price {
                Some(p) if is_price_compatible(incoming.side, incoming.price, p) => p,
                _ => break, // No compatible prices
            };
            
            // Match against orders at this price level
            // (Implementation details...)
        }
        
        // If remaining quantity, add to book
        if remaining > 0 && incoming.order_type == OrderType::Limit {
            let mut resting = incoming.clone();
            resting.remaining = remaining;
            self.add_order(resting);
        }
        
        MatchResult {
            order: incoming,
            trades,
            fully_filled: remaining == 0,
            remaining,
        }
    }
}

/// Check if prices are compatible for matching
fn is_price_compatible(incoming_side: Side, incoming_price: u64, book_price: u64) -> bool {
    match incoming_side {
        // Buy order matches if book price <= incoming price
        Side::Buy => book_price <= incoming_price,
        // Sell order matches if book price >= incoming price
        Side::Sell => book_price >= incoming_price,
    }
}
```

### Fixed-Point Arithmetic

Using `rust_decimal` for safe calculations:

```rust
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// Fixed-point scaling factor (10^8)
const SCALE: u64 = 100_000_000;

/// Convert u64 to Decimal
fn to_decimal(value: u64) -> Decimal {
    Decimal::from(value) / Decimal::from(SCALE)
}

/// Convert Decimal back to u64
fn from_decimal(value: Decimal) -> u64 {
    (value * Decimal::from(SCALE))
        .round_dp(0)
        .to_u64()
        .expect("Overflow in decimal conversion")
}

/// Safe multiplication with overflow check
fn checked_mul(a: u64, b: u64) -> Option<u64> {
    let da = to_decimal(a);
    let db = to_decimal(b);
    let result = da.checked_mul(db)?;
    Some(from_decimal(result))
}
```

### Determinism Requirements

**CRITICAL:** The matching engine MUST be 100% deterministic:

1. **No floating point** - All calculations use fixed-point
2. **No random numbers** - Deterministic ID generation
3. **No timestamps from system** - Timestamps passed in
4. **Deterministic iteration** - BTreeMap for ordered traversal
5. **No async/await** - Synchronous execution only

### Test/Acceptance Criteria
- [ ] Single order matches correctly against empty book (rests)
- [ ] Buy order matches against best ask (lowest price first)
- [ ] Sell order matches against best bid (highest price first)
- [ ] Partial fills work correctly
- [ ] Full fills remove orders from book
- [ ] Trade records are generated correctly
- [ ] Price-time priority is respected
- [ ] Same input sequence always produces same output (determinism test)
- [ ] Fixed-point calculations are accurate to 8 decimal places

### Unit Tests Required

```rust
#[test]
fn test_simple_match() {
    let mut clob = CLOB::with_capacity(1000);
    
    // Add resting sell order
    clob.add_order(Order::new_limit(Side::Sell, 50000_00000000, 1_00000000));
    
    // Incoming buy order should match
    let result = clob.match_order(
        Order::new_limit(Side::Buy, 50000_00000000, 1_00000000)
    );
    
    assert!(result.fully_filled);
    assert_eq!(result.trades.len(), 1);
    assert_eq!(result.trades[0].price, 50000_00000000);
    assert_eq!(result.trades[0].quantity, 1_00000000);
}

#[test]
fn test_partial_fill() {
    let mut clob = CLOB::with_capacity(1000);
    
    // Add resting sell order for 1 BTC
    clob.add_order(Order::new_limit(Side::Sell, 50000_00000000, 1_00000000));
    
    // Incoming buy order for 2 BTC
    let result = clob.match_order(
        Order::new_limit(Side::Buy, 50000_00000000, 2_00000000)
    );
    
    assert!(!result.fully_filled);
    assert_eq!(result.remaining, 1_00000000);
    assert_eq!(clob.order_count(), 1); // Remaining rests on book
}

#[test]
fn test_determinism() {
    // Run same sequence twice
    let trades1 = run_order_sequence();
    let trades2 = run_order_sequence();
    
    // Results must be identical
    assert_eq!(trades1, trades2);
}

#[test]
fn test_price_priority() {
    let mut clob = CLOB::with_capacity(1000);
    
    // Add asks at different prices
    clob.add_order(Order::new_limit(Side::Sell, 51000_00000000, 1_00000000));
    clob.add_order(Order::new_limit(Side::Sell, 50000_00000000, 1_00000000));
    clob.add_order(Order::new_limit(Side::Sell, 52000_00000000, 1_00000000));
    
    // Buy should match against lowest ask first
    let result = clob.match_order(
        Order::new_limit(Side::Buy, 52000_00000000, 1_00000000)
    );
    
    assert_eq!(result.trades[0].price, 50000_00000000);
}
```

### Owner
Lead Developer

### Estimated Time
4 days

---

## Sub-Phase 1.5: Benchmarking & Stress Testing

### Objective
Validate performance targets and establish baseline metrics for TEE comparison.

### Scope
- Implement benchmarking harness with Criterion
- Run 1M order stress test
- Measure and document TPS, latency, memory usage
- Verify deterministic state roots
- Create performance baseline report

### Deliverables

| Deliverable | Description | File(s) |
|-------------|-------------|---------|
| Benchmarks | Criterion benchmark suite | `/benches/matching.rs` |
| Stress test | 1M order test | `/tests/stress_test.rs` |
| Performance report | Documented metrics | `/docs/phase1-performance.md` |

### Performance Targets

| Metric | Target | Measurement Method |
|--------|--------|-------------------|
| Throughput | >100,000 orders/sec | Orders processed / elapsed time |
| Single match latency | <10μs | Criterion micro-benchmark |
| 1M order test | <10 seconds | Total elapsed time |
| Memory per order | <200 bytes | Peak memory / order count |
| Deterministic roots | 100% match | Compare roots across runs |

### Benchmark Implementation

```rust
// benches/matching.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

fn bench_single_match(c: &mut Criterion) {
    let mut clob = CLOB::with_capacity(100_000);
    
    // Pre-populate with orders
    for i in 0..1000 {
        clob.add_order(Order::new_limit(
            Side::Sell,
            50000_00000000 + (i * 1_00000000),
            1_00000000,
        ));
    }
    
    c.bench_function("single_match", |b| {
        b.iter(|| {
            let order = Order::new_limit(Side::Buy, 50000_00000000, 1_00000000);
            black_box(clob.match_order(order))
        })
    });
}

fn bench_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput");
    
    for size in [1000, 10_000, 100_000].iter() {
        group.bench_with_input(
            BenchmarkId::new("orders", size),
            size,
            |b, &size| {
                let mut clob = CLOB::with_capacity(size * 2);
                let orders = generate_random_orders(size);
                
                b.iter(|| {
                    for order in orders.iter().cloned() {
                        black_box(clob.match_order(order));
                    }
                });
            },
        );
    }
    
    group.finish();
}

criterion_group!(benches, bench_single_match, bench_throughput);
criterion_main!(benches);
```

### Stress Test Implementation

```rust
// tests/stress_test.rs
use std::time::Instant;
use sha2::{Sha256, Digest};

const ORDER_COUNT: usize = 1_000_000;

#[test]
fn stress_test_1m_orders() {
    println!("Initializing CLOB with capacity for {} orders...", ORDER_COUNT);
    let mut clob = CLOB::with_capacity(ORDER_COUNT);
    
    println!("Generating {} random orders...", ORDER_COUNT);
    let orders = generate_deterministic_orders(ORDER_COUNT, 42); // Seed for reproducibility
    
    println!("Starting stress test...");
    let start = Instant::now();
    
    let mut trade_count = 0;
    for order in orders {
        let result = clob.match_order(order);
        trade_count += result.trades.len();
    }
    
    let elapsed = start.elapsed();
    let tps = ORDER_COUNT as f64 / elapsed.as_secs_f64();
    
    println!("=== STRESS TEST RESULTS ===");
    println!("Orders processed: {}", ORDER_COUNT);
    println!("Trades generated: {}", trade_count);
    println!("Elapsed time: {:?}", elapsed);
    println!("Throughput: {:.0} orders/sec", tps);
    println!("Avg latency: {:.2}μs/order", elapsed.as_micros() as f64 / ORDER_COUNT as f64);
    
    // Verify performance target
    assert!(tps > 100_000.0, "Failed to meet 100k TPS target: {:.0}", tps);
    
    // Calculate and print state root
    let state_root = clob.compute_state_root();
    println!("Final state root: {}", hex::encode(state_root));
}

#[test]
fn verify_determinism() {
    // Run same sequence twice
    let state1 = run_deterministic_sequence(42);
    let state2 = run_deterministic_sequence(42);
    
    assert_eq!(state1, state2, "State roots must match for determinism");
    println!("Determinism verified: {}", hex::encode(state1));
}
```

### Test/Acceptance Criteria
- [ ] Single match latency < 10μs
- [ ] 1M order stress test completes in < 10 seconds
- [ ] Throughput exceeds 100,000 orders/second
- [ ] Memory usage is reasonable (< 200MB for 1M orders)
- [ ] State roots are identical across multiple runs
- [ ] No panics or errors during stress test
- [ ] Benchmark results are documented

### Owner
Lead Developer

### Estimated Time
2 days

---

## Summary: Phase 1 Timeline

| Sub-Phase | Description | Duration | Cumulative |
|-----------|-------------|----------|------------|
| 1.1 | Project Setup | 1 day | 1 day |
| 1.2 | Core Data Structures | 2 days | 3 days |
| 1.3 | Order Book (CLOB) | 3 days | 6 days |
| 1.4 | Matching Engine | 4 days | 10 days |
| 1.5 | Benchmarking | 2 days | **12 days** |

**Total Estimated Duration: 12 working days (2.5 weeks)**

---

## Phase 1 Exit Criteria

Before proceeding to Phase 2 (TEE Integration), ALL of the following must be verified:

### Functional Requirements
- [ ] All unit tests pass (`cargo test`)
- [ ] SSZ serialization round-trips correctly
- [ ] Order matching follows price-time priority
- [ ] Partial fills work correctly
- [ ] Cancel orders work correctly

### Performance Requirements
- [ ] >100,000 orders/second throughput
- [ ] <10μs average match latency
- [ ] 1M order stress test passes
- [ ] Memory usage < 200MB for 1M orders

### Determinism Requirements
- [ ] Same order sequence produces same trades
- [ ] State roots match across runs
- [ ] No floating-point operations in hot path
- [ ] No non-deterministic operations

### Documentation Requirements
- [ ] Performance baseline documented
- [ ] API documentation complete
- [ ] Test coverage report generated

---

## Appendix A: File Structure

```
dark-hypercore/
├── Cargo.toml
├── README.md
├── PRD.md
├── phase1.md                    # This document
├── .cargo/
│   └── config.toml              # Build optimizations
├── src/
│   ├── lib.rs                   # Library root
│   ├── main.rs                  # Binary entry point
│   ├── types/
│   │   ├── mod.rs
│   │   ├── order.rs             # Order, Side, OrderType
│   │   ├── trade.rs             # Trade struct
│   │   ├── receipt.rs           # ExecutionReceipt
│   │   └── price.rs             # Fixed-point helpers
│   ├── orderbook/
│   │   ├── mod.rs
│   │   ├── node.rs              # OrderNode
│   │   ├── level.rs             # PriceLevel
│   │   └── clob.rs              # CLOB struct
│   └── engine/
│       ├── mod.rs
│       ├── matcher.rs           # Matching logic
│       ├── trades.rs            # Trade generation
│       └── state.rs             # State root calculation
├── benches/
│   └── matching.rs              # Criterion benchmarks
├── tests/
│   └── stress_test.rs           # 1M order stress test
└── docs/
    └── phase1-performance.md    # Performance report
```

---

## Appendix B: External References

### Official Documentation

| Library | URL | Purpose |
|---------|-----|---------|
| ssz_rs | https://docs.rs/ssz_rs/0.9.0 | SSZ serialization |
| slab | https://docs.rs/slab/0.4.11 | Pre-allocated storage |
| rust_decimal | https://docs.rs/rust_decimal | Fixed-point math |
| SSZ Spec | https://ethereum.org/developers/docs/data-structures-and-encoding/ssz/ | SSZ format specification |
| Criterion | https://docs.rs/criterion | Benchmarking |

### Key Concepts

- **SSZ (Simple Serialize)**: Ethereum's deterministic serialization format
- **Slab allocation**: Pre-allocated memory pool for O(1) operations
- **Fixed-point arithmetic**: Integer-based decimal math (scaled by 10^8)
- **CLOB**: Central Limit Order Book with price-time priority
- **Price-time priority**: Best price first, then earliest order

---

*This document is part of the Dark HyperCore project. Last updated: December 26, 2025*

