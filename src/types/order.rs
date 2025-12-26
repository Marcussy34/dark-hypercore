//! Order types for the Dark HyperCore matching engine.
//!
//! ## SSZ Serialization
//!
//! All types derive `SimpleSerialize` from ssz_rs for deterministic encoding.
//! Per the SSZ spec (ethereum.org):
//! - Basic types (u64, bool): Direct little-endian encoding
//! - Fixed-size composites: Concatenated little-endian fields
//!
//! ## Fixed-Point Representation
//!
//! Prices and quantities are stored as u64 scaled by 10^8 (SCALE constant).
//! This provides 8 decimal places of precision without floating-point errors.

use ssz_rs::prelude::*;

// Note: SCALE constant is defined in price.rs module
// Use: crate::types::price::SCALE

// ============================================================================
// Side enum
// ============================================================================

/// Order side: Buy or Sell
///
/// Represented as u8 for SSZ compatibility:
/// - Buy = 0
/// - Sell = 1
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Side {
    /// Buy order (bid) - wants to purchase the asset
    #[default]
    Buy,
    /// Sell order (ask) - wants to sell the asset
    Sell,
}

impl Side {
    /// Convert to u8 for serialization
    pub fn to_u8(self) -> u8 {
        match self {
            Side::Buy => 0,
            Side::Sell => 1,
        }
    }
    
    /// Convert from u8 for deserialization
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Side::Buy),
            1 => Some(Side::Sell),
            _ => None,
        }
    }
    
    /// Returns the opposite side
    pub fn opposite(self) -> Self {
        match self {
            Side::Buy => Side::Sell,
            Side::Sell => Side::Buy,
        }
    }
}

// ============================================================================
// OrderType enum
// ============================================================================

/// Order type enumeration
///
/// Phase 1 only supports Limit orders.
/// Future phases may add Market, Stop, etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum OrderType {
    /// Limit order - executes at specified price or better
    #[default]
    Limit,
}

impl OrderType {
    /// Convert to u8 for serialization
    pub fn to_u8(self) -> u8 {
        match self {
            OrderType::Limit => 0,
        }
    }
    
    /// Convert from u8 for deserialization
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(OrderType::Limit),
            _ => None,
        }
    }
}

// ============================================================================
// Order struct
// ============================================================================

/// A limit order in the order book.
///
/// ## Fields
///
/// All price/quantity fields use fixed-point representation (scaled by 10^8).
///
/// ## SSZ Layout
///
/// The struct is serialized as a fixed-size container:
/// - Total size: 57 bytes (8+8+1+8+8+8+8+8 = 57)
///
/// ## Example
///
/// ```
/// use dark_hypercore::types::{Order, Side};
///
/// // Create a buy order for 1 BTC at $50,000
/// let order = Order::new(
///     1,                      // id
///     100,                    // user_id
///     Side::Buy,              // side
///     5_000_000_000_000,      // price: 50000.00000000
///     100_000_000,            // quantity: 1.00000000
///     1703577600000,          // timestamp (ms)
/// );
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Default, SimpleSerialize)]
pub struct Order {
    /// Unique order identifier (assigned by the engine)
    pub id: u64,
    
    /// User/account identifier
    pub user_id: u64,
    
    /// Order side as u8 (0=Buy, 1=Sell)
    /// Stored as u8 for SSZ compatibility
    pub side_raw: u8,
    
    /// Price in fixed-point (scaled by 10^8)
    /// Example: 50000.00000000 = 5_000_000_000_000u64
    pub price: u64,
    
    /// Original quantity in fixed-point (scaled by 10^8)
    /// Example: 1.00000000 = 100_000_000u64
    pub quantity: u64,
    
    /// Remaining quantity (for partial fills)
    /// Decremented as the order is matched
    pub remaining: u64,
    
    /// Unix timestamp in milliseconds when order was created
    pub timestamp: u64,
    
    /// Order type as u8 (0=Limit)
    /// Stored as u8 for SSZ compatibility
    pub order_type_raw: u8,
}

impl Order {
    /// Create a new limit order
    ///
    /// # Arguments
    ///
    /// * `id` - Unique order identifier
    /// * `user_id` - User/account identifier
    /// * `side` - Buy or Sell
    /// * `price` - Price in fixed-point (scaled by 10^8)
    /// * `quantity` - Quantity in fixed-point (scaled by 10^8)
    /// * `timestamp` - Unix timestamp in milliseconds
    ///
    /// # Example
    ///
    /// ```
    /// use dark_hypercore::types::{Order, Side};
    ///
    /// let order = Order::new(1, 100, Side::Buy, 5_000_000_000_000, 100_000_000, 0);
    /// assert_eq!(order.side(), Side::Buy);
    /// ```
    pub fn new(
        id: u64,
        user_id: u64,
        side: Side,
        price: u64,
        quantity: u64,
        timestamp: u64,
    ) -> Self {
        Self {
            id,
            user_id,
            side_raw: side.to_u8(),
            price,
            quantity,
            remaining: quantity, // Initially, remaining = quantity
            timestamp,
            order_type_raw: OrderType::Limit.to_u8(),
        }
    }
    
    /// Get the order side
    pub fn side(&self) -> Side {
        Side::from_u8(self.side_raw).unwrap_or(Side::Buy)
    }
    
    /// Set the order side
    pub fn set_side(&mut self, side: Side) {
        self.side_raw = side.to_u8();
    }
    
    /// Get the order type
    pub fn order_type(&self) -> OrderType {
        OrderType::from_u8(self.order_type_raw).unwrap_or(OrderType::Limit)
    }
    
    /// Check if the order is fully filled
    pub fn is_filled(&self) -> bool {
        self.remaining == 0
    }
    
    /// Get the filled quantity
    pub fn filled_quantity(&self) -> u64 {
        self.quantity.saturating_sub(self.remaining)
    }
    
    /// Fill a portion of this order
    ///
    /// # Arguments
    ///
    /// * `fill_qty` - Quantity to fill (in fixed-point)
    ///
    /// # Returns
    ///
    /// The actual quantity filled (may be less if order doesn't have enough remaining)
    pub fn fill(&mut self, fill_qty: u64) -> u64 {
        let actual_fill = fill_qty.min(self.remaining);
        self.remaining = self.remaining.saturating_sub(actual_fill);
        actual_fill
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_side_conversion() {
        assert_eq!(Side::Buy.to_u8(), 0);
        assert_eq!(Side::Sell.to_u8(), 1);
        assert_eq!(Side::from_u8(0), Some(Side::Buy));
        assert_eq!(Side::from_u8(1), Some(Side::Sell));
        assert_eq!(Side::from_u8(2), None);
    }
    
    #[test]
    fn test_side_opposite() {
        assert_eq!(Side::Buy.opposite(), Side::Sell);
        assert_eq!(Side::Sell.opposite(), Side::Buy);
    }
    
    #[test]
    fn test_order_type_conversion() {
        assert_eq!(OrderType::Limit.to_u8(), 0);
        assert_eq!(OrderType::from_u8(0), Some(OrderType::Limit));
        assert_eq!(OrderType::from_u8(1), None);
    }
    
    #[test]
    fn test_order_new() {
        let order = Order::new(
            1,
            100,
            Side::Buy,
            5_000_000_000_000, // 50000.00000000
            100_000_000,       // 1.00000000
            1703577600000,
        );
        
        assert_eq!(order.id, 1);
        assert_eq!(order.user_id, 100);
        assert_eq!(order.side(), Side::Buy);
        assert_eq!(order.price, 5_000_000_000_000);
        assert_eq!(order.quantity, 100_000_000);
        assert_eq!(order.remaining, 100_000_000);
        assert_eq!(order.order_type(), OrderType::Limit);
        assert!(!order.is_filled());
    }
    
    #[test]
    fn test_order_fill() {
        let mut order = Order::new(1, 100, Side::Buy, 5_000_000_000_000, 100_000_000, 0);
        
        // Partial fill
        let filled = order.fill(30_000_000);
        assert_eq!(filled, 30_000_000);
        assert_eq!(order.remaining, 70_000_000);
        assert_eq!(order.filled_quantity(), 30_000_000);
        assert!(!order.is_filled());
        
        // Fill the rest
        let filled = order.fill(70_000_000);
        assert_eq!(filled, 70_000_000);
        assert_eq!(order.remaining, 0);
        assert!(order.is_filled());
    }
    
    #[test]
    fn test_order_overfill() {
        let mut order = Order::new(1, 100, Side::Buy, 5_000_000_000_000, 100_000_000, 0);
        
        // Try to fill more than available
        let filled = order.fill(200_000_000);
        assert_eq!(filled, 100_000_000); // Only fills what's available
        assert_eq!(order.remaining, 0);
        assert!(order.is_filled());
    }
    
    #[test]
    fn test_order_ssz_roundtrip() {
        let order = Order::new(
            1,
            100,
            Side::Buy,
            5_000_000_000_000,
            100_000_000,
            1703577600000,
        );
        
        // Serialize
        let serialized = ssz_rs::serialize(&order).expect("Failed to serialize");
        
        // Deserialize
        let deserialized: Order = ssz_rs::deserialize(&serialized).expect("Failed to deserialize");
        
        // Verify roundtrip
        assert_eq!(order, deserialized);
    }
    
    #[test]
    fn test_order_deterministic_serialization() {
        // Same order should always produce identical bytes
        let order = Order::new(1, 100, Side::Buy, 5_000_000_000_000, 100_000_000, 1703577600000);
        
        let bytes1 = ssz_rs::serialize(&order).expect("Failed to serialize");
        let bytes2 = ssz_rs::serialize(&order).expect("Failed to serialize");
        
        assert_eq!(bytes1, bytes2, "SSZ serialization must be deterministic");
    }
    
    #[test]
    fn test_order_ssz_size() {
        let order = Order::new(1, 100, Side::Buy, 5_000_000_000_000, 100_000_000, 0);
        let bytes = ssz_rs::serialize(&order).expect("Failed to serialize");
        
        // Expected size: 8+8+1+8+8+8+8+1 = 50 bytes
        // (id + user_id + side_raw + price + quantity + remaining + timestamp + order_type_raw)
        assert_eq!(bytes.len(), 50, "Order should serialize to 50 bytes");
    }
}

