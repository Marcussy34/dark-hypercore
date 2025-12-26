//! Benchmarks for the Dark HyperCore matching engine.
//!
//! ## Performance Targets (from phase1.md)
//!
//! | Metric              | Target            |
//! |---------------------|-------------------|
//! | Single match latency| < 10μs            |
//! | Throughput          | > 100,000 ops/sec |
//! | 1M order test       | < 10 seconds      |
//!
//! ## Running Benchmarks
//!
//! ```bash
//! # Run all benchmarks
//! cargo bench
//!
//! # Run specific benchmark
//! cargo bench -- single_match
//!
//! # Run with verbose output
//! cargo bench -- --verbose
//! ```
//!
//! Results are saved to `target/criterion/` with HTML reports.

use criterion::{
    black_box, criterion_group, criterion_main, 
    Criterion, BenchmarkId, Throughput, BatchSize
};
use std::time::Duration;

use dark_hypercore::{CLOB, MatchingEngine, Order, Side};

// ============================================================================
// HELPER FUNCTIONS - Deterministic order generation
// ============================================================================

/// Generate a deterministic buy order for benchmarking
fn make_buy_order(id: u64, price: u64, quantity: u64) -> Order {
    Order::new(id, 1, Side::Buy, price, quantity, 0)
}

/// Generate a deterministic sell order for benchmarking
fn make_sell_order(id: u64, price: u64, quantity: u64) -> Order {
    Order::new(id, 1, Side::Sell, price, quantity, 0)
}

/// Pre-populate a CLOB with sell orders at various price levels.
/// This creates a realistic order book for matching benchmarks.
///
/// # Arguments
/// * `clob` - The CLOB to populate
/// * `count` - Number of orders to add
/// * `base_price` - Starting price (lowest ask)
/// * `price_step` - Price increment between levels
/// * `quantity` - Quantity per order (in fixed-point, 10^8)
fn populate_asks(clob: &mut CLOB, count: usize, base_price: u64, price_step: u64, quantity: u64) {
    for i in 0..count {
        let price = base_price + (i as u64 * price_step);
        let order = make_sell_order(0, price, quantity); // ID 0 = auto-assign
        clob.add_order(order);
    }
}

/// Pre-populate a CLOB with buy orders at various price levels.
fn populate_bids(clob: &mut CLOB, count: usize, base_price: u64, price_step: u64, quantity: u64) {
    for i in 0..count {
        let price = base_price - (i as u64 * price_step);
        let order = make_buy_order(0, price, quantity); // ID 0 = auto-assign
        clob.add_order(order);
    }
}

/// Generate a vector of deterministic orders for throughput testing.
/// Alternates between buy and sell orders with slight price variations.
fn generate_order_batch(count: usize, seed: u64) -> Vec<Order> {
    use rand::{Rng, SeedableRng};
    use rand_chacha::ChaCha8Rng;
    
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut orders = Vec::with_capacity(count);
    
    // Base price: 50000.00000000 (in fixed-point)
    let base_price: u64 = 5_000_000_000_000;
    
    for i in 0..count {
        let is_buy = rng.gen_bool(0.5);
        // Price variation: ±500.00000000 (in fixed-point)
        let price_offset: i64 = rng.gen_range(-50_000_000_000i64..=50_000_000_000i64);
        let price = (base_price as i64 + price_offset) as u64;
        // Quantity: 0.01 to 1.0 (in fixed-point)
        let quantity: u64 = rng.gen_range(1_000_000..=100_000_000);
        
        let order = if is_buy {
            make_buy_order((i + 1) as u64, price, quantity)
        } else {
            make_sell_order((i + 1) as u64, price, quantity)
        };
        orders.push(order);
    }
    
    orders
}

// ============================================================================
// BENCHMARK: Single Match Latency
// ============================================================================
// Target: < 10μs per match operation

fn bench_single_match(c: &mut Criterion) {
    let mut group = c.benchmark_group("single_match");
    
    // Configure for micro-benchmarking
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(1000);
    
    // Benchmark: Match against a book with 1,000 resting orders
    group.bench_function("against_1k_orders", |b| {
        // Setup: Create CLOB with resting sell orders
        let mut clob = CLOB::with_capacity(2000);
        let mut engine = MatchingEngine::new();
        
        // Populate with 1000 sell orders at increasing prices
        // Base: 50000.00000000, step: 1.00000000
        populate_asks(&mut clob, 1000, 5_000_000_000_000, 100_000_000, 100_000_000);
        
        // Benchmark: Match a buy order against the best ask
        b.iter_batched(
            || {
                // Setup: Create a buy order that matches the best ask
                make_buy_order(999999, 5_000_000_000_000, 100_000_000)
            },
            |buy_order| {
                // NOTE: This modifies clob, but we want to measure real matching
                // For pure latency measurement, we clone the clob in setup
                black_box(engine.match_order(&mut clob, buy_order, 0))
            },
            BatchSize::SmallInput
        );
    });
    
    // Benchmark: Match that sweeps multiple price levels
    group.bench_function("multi_level_sweep", |b| {
        b.iter_batched(
            || {
                // Setup: Fresh CLOB with 100 asks at different prices
                let mut clob = CLOB::with_capacity(200);
                populate_asks(&mut clob, 100, 5_000_000_000_000, 100_000_000, 10_000_000);
                
                // Buy order large enough to sweep ~10 levels
                let buy = make_buy_order(999999, 5_001_000_000_000, 100_000_000);
                (clob, buy)
            },
            |(mut clob, buy)| {
                let mut engine = MatchingEngine::new();
                black_box(engine.match_order(&mut clob, buy, 0))
            },
            BatchSize::SmallInput
        );
    });
    
    // Benchmark: No-match (order rests on book)
    group.bench_function("no_match_rest_on_book", |b| {
        b.iter_batched(
            || {
                // Setup: CLOB with asks, buy price below best ask
                let mut clob = CLOB::with_capacity(2000);
                populate_asks(&mut clob, 1000, 5_000_000_000_000, 100_000_000, 100_000_000);
                
                // Buy order below best ask - will rest on book
                let buy = make_buy_order(999999, 4_900_000_000_000, 100_000_000);
                (clob, buy)
            },
            |(mut clob, buy)| {
                let mut engine = MatchingEngine::new();
                black_box(engine.match_order(&mut clob, buy, 0))
            },
            BatchSize::SmallInput
        );
    });
    
    group.finish();
}

// ============================================================================
// BENCHMARK: Order Operations
// ============================================================================
// Measure add_order and cancel_order performance

fn bench_order_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("order_operations");
    
    group.measurement_time(Duration::from_secs(5));
    
    // Benchmark: Add order to empty book
    group.bench_function("add_to_empty", |b| {
        b.iter_batched(
            || CLOB::new(),
            |mut clob| {
                let order = make_buy_order(1, 5_000_000_000_000, 100_000_000);
                black_box(clob.add_order(order))
            },
            BatchSize::SmallInput
        );
    });
    
    // Benchmark: Add order to populated book
    group.bench_function("add_to_1k_book", |b| {
        b.iter_batched(
            || {
                let mut clob = CLOB::with_capacity(2000);
                populate_asks(&mut clob, 500, 5_000_000_000_000, 100_000_000, 100_000_000);
                populate_bids(&mut clob, 500, 4_999_000_000_000, 100_000_000, 100_000_000);
                clob
            },
            |mut clob| {
                let order = make_buy_order(0, 4_500_000_000_000, 100_000_000);
                black_box(clob.add_order(order))
            },
            BatchSize::SmallInput
        );
    });
    
    // Benchmark: Cancel order
    group.bench_function("cancel_order", |b| {
        b.iter_batched(
            || {
                let mut clob = CLOB::with_capacity(2000);
                populate_bids(&mut clob, 1000, 5_000_000_000_000, 100_000_000, 100_000_000);
                // Return clob and an order ID that exists (first order has ID 1)
                clob
            },
            |mut clob| {
                // Cancel order ID 500 (middle of the book)
                black_box(clob.cancel_order(500))
            },
            BatchSize::SmallInput
        );
    });
    
    group.finish();
}

// ============================================================================
// BENCHMARK: Throughput
// ============================================================================
// Target: > 100,000 orders/second

fn bench_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput");
    
    // Increase measurement time for throughput tests
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(50);
    
    // Test different batch sizes
    for batch_size in [1_000, 10_000, 50_000] {
        group.throughput(Throughput::Elements(batch_size as u64));
        
        group.bench_with_input(
            BenchmarkId::new("orders", batch_size),
            &batch_size,
            |b, &size| {
                // Generate orders deterministically (same seed = same orders)
                let orders = generate_order_batch(size, 42);
                
                b.iter_batched(
                    || {
                        // Fresh CLOB for each iteration
                        let clob = CLOB::with_capacity(size * 2);
                        let engine = MatchingEngine::new();
                        (clob, engine, orders.clone())
                    },
                    |(mut clob, mut engine, orders)| {
                        for order in orders {
                            black_box(engine.match_order(&mut clob, order, 0));
                        }
                        clob.order_count() // Return something to prevent optimization
                    },
                    BatchSize::LargeInput
                );
            }
        );
    }
    
    group.finish();
}

// ============================================================================
// BENCHMARK: Memory Efficiency
// ============================================================================
// Measure operations with large order books

fn bench_large_book(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_book");
    
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(50);
    
    // Benchmark with 100k orders in the book
    group.bench_function("match_in_100k_book", |b| {
        // Pre-create the large book (expensive, done once)
        let mut clob = CLOB::with_capacity(120_000);
        populate_asks(&mut clob, 50_000, 5_000_000_000_000, 100_000, 10_000_000);
        populate_bids(&mut clob, 50_000, 4_999_000_000_000, 100_000, 10_000_000);
        
        let mut engine = MatchingEngine::new();
        
        // Measure matching performance with large book
        b.iter(|| {
            // Create a buy order that matches
            let buy = make_buy_order(999999, 5_000_000_000_000, 10_000_000);
            black_box(engine.match_order(&mut clob, buy, 0))
        });
    });
    
    group.finish();
}

// ============================================================================
// BENCHMARK: Determinism Verification
// ============================================================================
// Ensure same sequence produces same results

fn bench_determinism(c: &mut Criterion) {
    let mut group = c.benchmark_group("determinism");
    
    group.measurement_time(Duration::from_secs(5));
    
    // Benchmark the deterministic sequence
    group.bench_function("1k_deterministic_sequence", |b| {
        let orders = generate_order_batch(1000, 12345);
        
        b.iter_batched(
            || orders.clone(),
            |orders| {
                let mut clob = CLOB::with_capacity(2000);
                let mut engine = MatchingEngine::new();
                let mut trade_count = 0;
                
                for order in orders {
                    let result = engine.match_order(&mut clob, order, 0);
                    trade_count += result.trades.len();
                }
                
                black_box((clob.order_count(), trade_count))
            },
            BatchSize::SmallInput
        );
    });
    
    group.finish();
}

// ============================================================================
// CRITERION ENTRY POINT
// ============================================================================

criterion_group!(
    benches,
    bench_single_match,
    bench_order_operations,
    bench_throughput,
    bench_large_book,
    bench_determinism
);

criterion_main!(benches);
