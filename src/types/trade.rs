//! Trade type representing an executed match between two orders.
//!
//! ## SSZ Serialization
//!
//! Trades are serialized using SSZ for deterministic encoding.
//! This ensures identical state roots across all validators.

use ssz_rs::prelude::*;

/// A trade represents a single match between a maker and taker order.
///
/// ## Terminology
///
/// - **Maker**: The resting order that was already in the book
/// - **Taker**: The incoming order that triggered the match
///
/// ## Price Discovery
///
/// The trade always executes at the maker's price (the resting order's price).
/// This is standard price-time priority behavior.
///
/// ## Example
///
/// ```
/// use dark_hypercore::types::Trade;
///
/// let trade = Trade::new(
///     1,                      // trade_id
///     100,                    // maker_order_id
///     200,                    // taker_order_id
///     100,                    // maker_user_id
///     200,                    // taker_user_id
///     5_000_000_000_000,      // price: 50000.00000000
///     50_000_000,             // quantity: 0.50000000
///     1703577600000,          // timestamp
/// );
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Default, SimpleSerialize)]
pub struct Trade {
    /// Unique trade identifier (assigned by the engine)
    pub id: u64,
    
    /// Maker order ID (the resting order)
    pub maker_order_id: u64,
    
    /// Taker order ID (the incoming order)
    pub taker_order_id: u64,
    
    /// Maker user/account ID
    pub maker_user_id: u64,
    
    /// Taker user/account ID
    pub taker_user_id: u64,
    
    /// Execution price in fixed-point (scaled by 10^8)
    /// Always the maker's price
    pub price: u64,
    
    /// Executed quantity in fixed-point (scaled by 10^8)
    pub quantity: u64,
    
    /// Execution timestamp in milliseconds
    pub timestamp: u64,
}

impl Trade {
    /// Create a new trade
    ///
    /// # Arguments
    ///
    /// * `id` - Unique trade identifier
    /// * `maker_order_id` - ID of the resting (maker) order
    /// * `taker_order_id` - ID of the incoming (taker) order
    /// * `maker_user_id` - User ID of the maker
    /// * `taker_user_id` - User ID of the taker
    /// * `price` - Execution price (fixed-point, scaled by 10^8)
    /// * `quantity` - Execution quantity (fixed-point, scaled by 10^8)
    /// * `timestamp` - Execution timestamp in milliseconds
    pub fn new(
        id: u64,
        maker_order_id: u64,
        taker_order_id: u64,
        maker_user_id: u64,
        taker_user_id: u64,
        price: u64,
        quantity: u64,
        timestamp: u64,
    ) -> Self {
        Self {
            id,
            maker_order_id,
            taker_order_id,
            maker_user_id,
            taker_user_id,
            price,
            quantity,
            timestamp,
        }
    }
    
    /// Calculate the notional value of this trade (price * quantity)
    ///
    /// Note: This returns the value in fixed-point representation.
    /// The result is scaled by 10^16 (10^8 * 10^8).
    /// To get the actual notional, divide by SCALE (10^8).
    pub fn notional_raw(&self) -> u128 {
        (self.price as u128) * (self.quantity as u128)
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_trade_new() {
        let trade = Trade::new(
            1,
            100,
            200,
            10,
            20,
            5_000_000_000_000, // 50000.00000000
            50_000_000,        // 0.50000000
            1703577600000,
        );
        
        assert_eq!(trade.id, 1);
        assert_eq!(trade.maker_order_id, 100);
        assert_eq!(trade.taker_order_id, 200);
        assert_eq!(trade.maker_user_id, 10);
        assert_eq!(trade.taker_user_id, 20);
        assert_eq!(trade.price, 5_000_000_000_000);
        assert_eq!(trade.quantity, 50_000_000);
        assert_eq!(trade.timestamp, 1703577600000);
    }
    
    #[test]
    fn test_trade_notional() {
        let trade = Trade::new(
            1, 100, 200, 10, 20,
            5_000_000_000_000, // 50000.00000000
            100_000_000,       // 1.00000000
            0,
        );
        
        // Notional = 50000 * 1 = 50000
        // In raw form: 5_000_000_000_000 * 100_000_000 = 500_000_000_000_000_000_000
        let expected = 5_000_000_000_000u128 * 100_000_000u128;
        assert_eq!(trade.notional_raw(), expected);
    }
    
    #[test]
    fn test_trade_ssz_roundtrip() {
        let trade = Trade::new(
            1, 100, 200, 10, 20,
            5_000_000_000_000,
            50_000_000,
            1703577600000,
        );
        
        // Serialize
        let serialized = ssz_rs::serialize(&trade).expect("Failed to serialize");
        
        // Deserialize
        let deserialized: Trade = ssz_rs::deserialize(&serialized).expect("Failed to deserialize");
        
        // Verify roundtrip
        assert_eq!(trade, deserialized);
    }
    
    #[test]
    fn test_trade_deterministic_serialization() {
        let trade = Trade::new(1, 100, 200, 10, 20, 5_000_000_000_000, 50_000_000, 1703577600000);
        
        let bytes1 = ssz_rs::serialize(&trade).expect("Failed to serialize");
        let bytes2 = ssz_rs::serialize(&trade).expect("Failed to serialize");
        
        assert_eq!(bytes1, bytes2, "SSZ serialization must be deterministic");
    }
    
    #[test]
    fn test_trade_ssz_size() {
        let trade = Trade::new(1, 100, 200, 10, 20, 5_000_000_000_000, 50_000_000, 0);
        let bytes = ssz_rs::serialize(&trade).expect("Failed to serialize");
        
        // Expected size: 8 fields * 8 bytes = 64 bytes
        assert_eq!(bytes.len(), 64, "Trade should serialize to 64 bytes");
    }
}

