//! Dark HyperCore - Binary Entry Point
//!
//! This binary will eventually run the matching engine.
//! For now, it serves as a simple verification that the project builds.

use dark_hypercore::types::{Order, Side};

fn main() {
    println!("===========================================");
    println!("  Dark HyperCore - The Dark Kernel");
    println!("===========================================");
    println!();
    
    // Demonstrate basic type creation
    println!("Creating sample order...");
    let order = Order::new(
        1,                      // id
        100,                    // user_id
        Side::Buy,              // side
        5_000_000_000_000,      // price: 50000.00000000 (scaled by 10^8)
        100_000_000,            // quantity: 1.00000000 (scaled by 10^8)
        1703577600000,          // timestamp (ms)
    );
    
    println!("Order created:");
    println!("  ID: {}", order.id);
    println!("  Side: {:?}", order.side());
    println!("  Price: {} (raw)", order.price);
    println!("  Price: {:.8} (human)", order.price as f64 / 100_000_000.0);
    println!("  Quantity: {} (raw)", order.quantity);
    println!("  Quantity: {:.8} (human)", order.quantity as f64 / 100_000_000.0);
    println!();
    
    // Test SSZ serialization
    println!("Testing SSZ serialization...");
    match ssz_rs::serialize(&order) {
        Ok(bytes) => {
            println!("  Serialized to {} bytes", bytes.len());
            println!("  Bytes: {:?}", &bytes[..bytes.len().min(32)]);
            if bytes.len() > 32 {
                println!("  ... ({} more bytes)", bytes.len() - 32);
            }
        }
        Err(e) => {
            println!("  ERROR: Failed to serialize: {:?}", e);
        }
    }
    
    println!();
    println!("Phase 1.1 & 1.2: Project setup complete!");
    println!("Run 'cargo test' to verify all tests pass.");
}
