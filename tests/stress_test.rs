//! Stress tests for the Dark HyperCore matching engine.
//!
//! These tests verify:
//! 1. Performance targets are met (>100k orders/sec)
//! 2. System remains stable under high load
//! 3. Determinism is preserved across runs
//! 4. Memory usage is reasonable
//!
//! ## Running Stress Tests
//!
//! ```bash
//! # Run all stress tests (release mode recommended)
//! cargo test --release --test stress_test -- --nocapture
//!
//! # Run specific test
//! cargo test --release --test stress_test stress_1m_orders -- --nocapture
//! ```

use std::time::Instant;

use dark_hypercore::{CLOB, MatchingEngine, Order, Side};

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

// ============================================================================
// TEST CONSTANTS
// ============================================================================

/// Number of orders for the 1M stress test
const STRESS_ORDER_COUNT: usize = 1_000_000;

/// Target throughput (orders per second)
const TARGET_THROUGHPUT: f64 = 100_000.0;

/// Maximum allowed time for 1M orders (seconds)
const MAX_TIME_SECONDS: f64 = 10.0;

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Generate deterministic orders for stress testing.
///
/// Uses a seeded RNG for reproducibility. Same seed = same orders.
fn generate_deterministic_orders(count: usize, seed: u64) -> Vec<Order> {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut orders = Vec::with_capacity(count);
    
    // Base price: 50000.00000000 (in fixed-point, 10^8 scale)
    let base_price: u64 = 5_000_000_000_000;
    
    for i in 0..count {
        let is_buy = rng.gen_bool(0.5);
        
        // Price variation: ±1000.00000000 (in fixed-point)
        // This ensures meaningful price spread for matching
        let price_offset: i64 = rng.gen_range(-100_000_000_000i64..=100_000_000_000i64);
        let price = (base_price as i64 + price_offset) as u64;
        
        // Quantity: 0.001 to 1.0 (in fixed-point)
        let quantity: u64 = rng.gen_range(100_000..=100_000_000);
        
        let user_id: u64 = rng.gen_range(1..=10_000);
        
        let order = Order::new(
            (i + 1) as u64,  // Order ID
            user_id,
            if is_buy { Side::Buy } else { Side::Sell },
            price,
            quantity,
            i as u64,  // Timestamp = sequence number
        );
        
        orders.push(order);
    }
    
    orders
}

/// Run a deterministic order sequence and return the final state root.
fn run_deterministic_sequence(seed: u64, count: usize) -> [u8; 32] {
    let orders = generate_deterministic_orders(count, seed);
    
    let mut clob = CLOB::with_capacity(count * 2);
    let mut engine = MatchingEngine::new();
    
    for order in orders {
        engine.match_order(&mut clob, order, 0);
    }
    
    clob.compute_state_root()
}

// ============================================================================
// STRESS TESTS
// ============================================================================

/// Main stress test: Process 1 million orders.
///
/// # Performance Targets
/// - Throughput: >100,000 orders/second
/// - Total time: <10 seconds
///
/// # Verification
/// - No panics during execution
/// - State root is computed correctly
/// - Trade count is positive (some matching occurred)
#[test]
fn stress_1m_orders() {
    println!("\n=== STRESS TEST: 1 Million Orders ===\n");
    
    // Setup
    println!("Generating {} deterministic orders (seed=42)...", STRESS_ORDER_COUNT);
    let gen_start = Instant::now();
    let orders = generate_deterministic_orders(STRESS_ORDER_COUNT, 42);
    let gen_time = gen_start.elapsed();
    println!("  Generated in {:.2?}", gen_time);
    
    println!("\nInitializing CLOB with capacity {}...", STRESS_ORDER_COUNT * 2);
    let mut clob = CLOB::with_capacity(STRESS_ORDER_COUNT * 2);
    let mut engine = MatchingEngine::new();
    
    // Run stress test
    println!("\nProcessing orders...");
    let start = Instant::now();
    
    let mut trade_count = 0;
    for order in orders {
        let result = engine.match_order(&mut clob, order, 0);
        trade_count += result.trades.len();
    }
    
    let elapsed = start.elapsed();
    let elapsed_secs = elapsed.as_secs_f64();
    let throughput = STRESS_ORDER_COUNT as f64 / elapsed_secs;
    let avg_latency_us = elapsed.as_micros() as f64 / STRESS_ORDER_COUNT as f64;
    
    // Compute final state
    println!("\nComputing state root...");
    let state_root = clob.compute_state_root();
    
    // Print results
    println!("\n=== RESULTS ===");
    println!("  Orders processed:  {:>12}", STRESS_ORDER_COUNT);
    println!("  Trades generated:  {:>12}", trade_count);
    println!("  Final book size:   {:>12}", clob.order_count());
    println!("  Bid count:         {:>12}", clob.bid_count());
    println!("  Ask count:         {:>12}", clob.ask_count());
    println!();
    println!("  Elapsed time:      {:>12.2?}", elapsed);
    println!("  Throughput:        {:>12.0} orders/sec", throughput);
    println!("  Avg latency:       {:>12.2} μs/order", avg_latency_us);
    println!();
    println!("  State root:        {}", hex::encode(state_root));
    
    // Verify performance targets
    println!("\n=== PERFORMANCE CHECK ===");
    
    let throughput_ok = throughput >= TARGET_THROUGHPUT;
    let time_ok = elapsed_secs <= MAX_TIME_SECONDS;
    
    println!("  Throughput >= {:.0}/sec: {} ({:.0} actual)",
        TARGET_THROUGHPUT,
        if throughput_ok { "PASS ✓" } else { "FAIL ✗" },
        throughput
    );
    println!("  Time <= {:.1}s:         {} ({:.2}s actual)",
        MAX_TIME_SECONDS,
        if time_ok { "PASS ✓" } else { "FAIL ✗" },
        elapsed_secs
    );
    
    // Assertions
    assert!(throughput_ok, 
        "Throughput {:.0} orders/sec below target {:.0}", 
        throughput, TARGET_THROUGHPUT);
    assert!(time_ok, 
        "Elapsed time {:.2}s exceeds maximum {:.1}s", 
        elapsed_secs, MAX_TIME_SECONDS);
    assert!(trade_count > 0, "Expected some trades to occur");
    
    println!("\n=== STRESS TEST PASSED ===\n");
}

/// Verify determinism: Same sequence produces identical state root.
///
/// This is critical for consensus - all nodes must produce the same
/// final state given the same input sequence.
#[test]
fn verify_determinism() {
    println!("\n=== DETERMINISM TEST ===\n");
    
    const TEST_COUNT: usize = 10_000;  // Smaller for faster test
    const SEED: u64 = 12345;
    
    println!("Running sequence with {} orders (seed={})...", TEST_COUNT, SEED);
    
    // Run sequence twice
    let root1 = run_deterministic_sequence(SEED, TEST_COUNT);
    let root2 = run_deterministic_sequence(SEED, TEST_COUNT);
    
    println!("  Run 1 state root: {}", hex::encode(root1));
    println!("  Run 2 state root: {}", hex::encode(root2));
    
    // Verify identical
    assert_eq!(root1, root2, "State roots must match for determinism");
    
    // Also verify different seeds produce different roots
    let root3 = run_deterministic_sequence(SEED + 1, TEST_COUNT);
    println!("  Different seed:   {}", hex::encode(root3));
    assert_ne!(root1, root3, "Different seeds should produce different roots");
    
    println!("\n=== DETERMINISM VERIFIED ===\n");
}

/// Test varying load sizes to ensure consistent performance.
#[test]
fn stress_scaling() {
    println!("\n=== SCALING TEST ===\n");
    
    let test_sizes = [1_000, 10_000, 100_000, 500_000];
    
    println!("{:>12} {:>12} {:>12} {:>12}", "Orders", "Time", "Throughput", "Latency");
    println!("{:-<12} {:-<12} {:-<12} {:-<12}", "", "", "", "");
    
    for &size in &test_sizes {
        let orders = generate_deterministic_orders(size, 42);
        let mut clob = CLOB::with_capacity(size * 2);
        let mut engine = MatchingEngine::new();
        
        let start = Instant::now();
        for order in orders {
            engine.match_order(&mut clob, order, 0);
        }
        let elapsed = start.elapsed();
        
        let throughput = size as f64 / elapsed.as_secs_f64();
        let latency_us = elapsed.as_micros() as f64 / size as f64;
        
        println!("{:>12} {:>12.2?} {:>12.0} {:>12.2}μs", 
            size, elapsed, throughput, latency_us);
    }
    
    println!("\n=== SCALING TEST COMPLETE ===\n");
}

/// Test cancel operations under load.
#[test]
fn stress_cancellations() {
    println!("\n=== CANCELLATION STRESS TEST ===\n");
    
    const ORDER_COUNT: usize = 100_000;
    const CANCEL_RATE: f64 = 0.3;  // 30% of orders get cancelled
    
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let mut clob = CLOB::with_capacity(ORDER_COUNT * 2);
    let mut engine = MatchingEngine::new();
    
    let mut orders_placed = 0;
    let mut orders_cancelled = 0;
    let mut resting_order_ids: Vec<u64> = Vec::new();
    
    let start = Instant::now();
    
    for i in 0..ORDER_COUNT {
        // Occasionally cancel a resting order
        if !resting_order_ids.is_empty() && rng.gen_bool(CANCEL_RATE) {
            let idx = rng.gen_range(0..resting_order_ids.len());
            let order_id = resting_order_ids.swap_remove(idx);
            if clob.cancel_order(order_id).is_some() {
                orders_cancelled += 1;
            }
        }
        
        // Place new order
        let is_buy = rng.gen_bool(0.5);
        let base_price: u64 = 5_000_000_000_000;
        let price_offset: i64 = rng.gen_range(-100_000_000_000i64..=100_000_000_000i64);
        let price = (base_price as i64 + price_offset) as u64;
        let quantity: u64 = rng.gen_range(100_000..=100_000_000);
        
        let order = Order::new(
            (i + 1) as u64,
            1,
            if is_buy { Side::Buy } else { Side::Sell },
            price,
            quantity,
            i as u64,
        );
        
        let order_id = order.id;
        let result = engine.match_order(&mut clob, order, 0);
        orders_placed += 1;
        
        // Track resting orders for potential cancellation
        if !result.fully_filled {
            resting_order_ids.push(order_id);
        }
    }
    
    let elapsed = start.elapsed();
    let ops_count = orders_placed + orders_cancelled;
    let throughput = ops_count as f64 / elapsed.as_secs_f64();
    
    println!("  Orders placed:     {:>12}", orders_placed);
    println!("  Orders cancelled:  {:>12}", orders_cancelled);
    println!("  Total operations:  {:>12}", ops_count);
    println!("  Final book size:   {:>12}", clob.order_count());
    println!("  Elapsed time:      {:>12.2?}", elapsed);
    println!("  Throughput:        {:>12.0} ops/sec", throughput);
    
    assert!(throughput >= 50_000.0, 
        "Mixed operations throughput too low: {:.0}", throughput);
    
    println!("\n=== CANCELLATION TEST PASSED ===\n");
}

/// Test memory efficiency by checking the book doesn't grow unbounded.
#[test]
fn stress_memory_stability() {
    println!("\n=== MEMORY STABILITY TEST ===\n");
    
    const ITERATIONS: usize = 100_000;
    const MAX_BOOK_SIZE: usize = 50_000;  // Should stabilize below this
    
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let mut clob = CLOB::with_capacity(MAX_BOOK_SIZE);
    let mut engine = MatchingEngine::new();
    
    let mut max_size_seen = 0;
    
    for i in 0..ITERATIONS {
        let is_buy = rng.gen_bool(0.5);
        let base_price: u64 = 5_000_000_000_000;
        // Tighter spread for more matching
        let price_offset: i64 = rng.gen_range(-10_000_000_000i64..=10_000_000_000i64);
        let price = (base_price as i64 + price_offset) as u64;
        let quantity: u64 = rng.gen_range(100_000..=10_000_000);
        
        let order = Order::new(
            (i + 1) as u64,
            1,
            if is_buy { Side::Buy } else { Side::Sell },
            price,
            quantity,
            i as u64,
        );
        
        engine.match_order(&mut clob, order, 0);
        
        let current_size = clob.order_count();
        if current_size > max_size_seen {
            max_size_seen = current_size;
        }
    }
    
    println!("  Iterations:        {:>12}", ITERATIONS);
    println!("  Max book size:     {:>12}", max_size_seen);
    println!("  Final book size:   {:>12}", clob.order_count());
    println!("  Book is bounded:   {}", 
        if max_size_seen < MAX_BOOK_SIZE { "YES ✓" } else { "NO ✗" });
    
    // With balanced buys/sells and overlapping prices, book should stay bounded
    assert!(max_size_seen < MAX_BOOK_SIZE,
        "Book grew too large: {} (max {})", max_size_seen, MAX_BOOK_SIZE);
    
    println!("\n=== MEMORY STABILITY PASSED ===\n");
}

