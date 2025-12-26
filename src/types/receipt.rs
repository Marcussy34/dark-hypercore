//! Execution receipt for batch processing results.
//!
//! The ExecutionReceipt provides a summary of a batch of order operations,
//! including the state root for verification.

use ssz_rs::prelude::*;
use sha2::{Sha256, Digest};

/// Execution receipt summarizing a batch of processed orders.
///
/// ## Purpose
///
/// The receipt serves as proof of execution for a batch of orders.
/// The state_root can be used to verify the order book state.
///
/// ## State Root
///
/// The 32-byte state root is a SHA-256 hash of the order book state.
/// This enables verification without revealing order details.
///
/// ## Example
///
/// ```
/// use dark_hypercore::types::ExecutionReceipt;
///
/// let receipt = ExecutionReceipt::new(
///     1,                      // batch_id
///     1000,                   // orders_processed
///     500,                    // trades_executed
///     [0u8; 32],              // state_root (would be computed)
///     1703577600000,          // timestamp
/// );
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Default, SimpleSerialize)]
pub struct ExecutionReceipt {
    /// Batch sequence number
    pub batch_id: u64,
    
    /// Number of orders processed in this batch
    pub orders_processed: u64,
    
    /// Number of trades executed in this batch
    pub trades_executed: u64,
    
    /// State root after execution (SHA-256 hash, 32 bytes)
    /// This is a merkle root of the order book state
    pub state_root: [u8; 32],
    
    /// Batch completion timestamp in milliseconds
    pub timestamp: u64,
}

impl ExecutionReceipt {
    /// Create a new execution receipt
    ///
    /// # Arguments
    ///
    /// * `batch_id` - Sequence number for this batch
    /// * `orders_processed` - Count of orders processed
    /// * `trades_executed` - Count of trades executed
    /// * `state_root` - 32-byte hash of the order book state
    /// * `timestamp` - Completion timestamp in milliseconds
    pub fn new(
        batch_id: u64,
        orders_processed: u64,
        trades_executed: u64,
        state_root: [u8; 32],
        timestamp: u64,
    ) -> Self {
        Self {
            batch_id,
            orders_processed,
            trades_executed,
            state_root,
            timestamp,
        }
    }
    
    /// Create a receipt with a computed state root from arbitrary data
    ///
    /// This is a convenience method for creating receipts during development.
    /// In production, the state root should be computed from the actual order book state.
    pub fn with_computed_root(
        batch_id: u64,
        orders_processed: u64,
        trades_executed: u64,
        state_data: &[u8],
        timestamp: u64,
    ) -> Self {
        let state_root = Self::compute_hash(state_data);
        Self::new(batch_id, orders_processed, trades_executed, state_root, timestamp)
    }
    
    /// Compute SHA-256 hash of the given data
    ///
    /// Returns a 32-byte array suitable for use as a state root.
    pub fn compute_hash(data: &[u8]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }
    
    /// Get the state root as a hex string
    pub fn state_root_hex(&self) -> String {
        hex::encode(self.state_root)
    }
    
    /// Check if this receipt represents an empty batch (no orders processed)
    pub fn is_empty(&self) -> bool {
        self.orders_processed == 0
    }
    
    /// Calculate the fill rate (trades / orders)
    ///
    /// Returns None if no orders were processed.
    pub fn fill_rate(&self) -> Option<f64> {
        if self.orders_processed == 0 {
            None
        } else {
            Some(self.trades_executed as f64 / self.orders_processed as f64)
        }
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_receipt_new() {
        let state_root = [1u8; 32];
        let receipt = ExecutionReceipt::new(
            1,
            1000,
            500,
            state_root,
            1703577600000,
        );
        
        assert_eq!(receipt.batch_id, 1);
        assert_eq!(receipt.orders_processed, 1000);
        assert_eq!(receipt.trades_executed, 500);
        assert_eq!(receipt.state_root, state_root);
        assert_eq!(receipt.timestamp, 1703577600000);
    }
    
    #[test]
    fn test_receipt_computed_root() {
        let receipt = ExecutionReceipt::with_computed_root(
            1,
            100,
            50,
            b"test state data",
            0,
        );
        
        // Verify the hash was computed
        assert_ne!(receipt.state_root, [0u8; 32]);
        
        // Verify it's deterministic
        let expected_hash = ExecutionReceipt::compute_hash(b"test state data");
        assert_eq!(receipt.state_root, expected_hash);
    }
    
    #[test]
    fn test_receipt_hash_determinism() {
        // Same input should always produce same hash
        let hash1 = ExecutionReceipt::compute_hash(b"test data");
        let hash2 = ExecutionReceipt::compute_hash(b"test data");
        assert_eq!(hash1, hash2);
        
        // Different input should produce different hash
        let hash3 = ExecutionReceipt::compute_hash(b"different data");
        assert_ne!(hash1, hash3);
    }
    
    #[test]
    fn test_receipt_state_root_hex() {
        let state_root = [0xAB; 32];
        let receipt = ExecutionReceipt::new(1, 0, 0, state_root, 0);
        
        let hex = receipt.state_root_hex();
        assert_eq!(hex.len(), 64); // 32 bytes * 2 hex chars
        assert!(hex.chars().all(|c| c.is_ascii_hexdigit()));
    }
    
    #[test]
    fn test_receipt_is_empty() {
        let empty = ExecutionReceipt::new(1, 0, 0, [0u8; 32], 0);
        assert!(empty.is_empty());
        
        let not_empty = ExecutionReceipt::new(1, 1, 0, [0u8; 32], 0);
        assert!(!not_empty.is_empty());
    }
    
    #[test]
    fn test_receipt_fill_rate() {
        let receipt = ExecutionReceipt::new(1, 100, 50, [0u8; 32], 0);
        assert_eq!(receipt.fill_rate(), Some(0.5));
        
        let empty = ExecutionReceipt::new(1, 0, 0, [0u8; 32], 0);
        assert_eq!(empty.fill_rate(), None);
    }
    
    #[test]
    fn test_receipt_ssz_roundtrip() {
        let receipt = ExecutionReceipt::new(
            1,
            1000,
            500,
            [0xAB; 32],
            1703577600000,
        );
        
        // Serialize
        let serialized = ssz_rs::serialize(&receipt).expect("Failed to serialize");
        
        // Deserialize
        let deserialized: ExecutionReceipt = ssz_rs::deserialize(&serialized)
            .expect("Failed to deserialize");
        
        // Verify roundtrip
        assert_eq!(receipt, deserialized);
    }
    
    #[test]
    fn test_receipt_deterministic_serialization() {
        let receipt = ExecutionReceipt::new(1, 1000, 500, [0xAB; 32], 1703577600000);
        
        let bytes1 = ssz_rs::serialize(&receipt).expect("Failed to serialize");
        let bytes2 = ssz_rs::serialize(&receipt).expect("Failed to serialize");
        
        assert_eq!(bytes1, bytes2, "SSZ serialization must be deterministic");
    }
    
    #[test]
    fn test_receipt_ssz_size() {
        let receipt = ExecutionReceipt::new(1, 0, 0, [0u8; 32], 0);
        let bytes = ssz_rs::serialize(&receipt).expect("Failed to serialize");
        
        // Expected size: 8 + 8 + 8 + 32 + 8 = 64 bytes
        assert_eq!(bytes.len(), 64, "ExecutionReceipt should serialize to 64 bytes");
    }
    
    #[test]
    fn test_receipt_state_root_is_32_bytes() {
        let receipt = ExecutionReceipt::default();
        assert_eq!(receipt.state_root.len(), 32, "State root must be exactly 32 bytes");
    }
}

