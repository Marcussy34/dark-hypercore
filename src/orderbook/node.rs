//! Order node for slab-based storage.
//!
//! ## Design
//!
//! `OrderNode` wraps an `Order` with doubly-linked list pointers for
//! efficient removal from price levels. This allows O(1) removal when
//! we have the slab key.
//!
//! ## Slab Integration
//!
//! Per official slab docs (https://docs.rs/slab/0.4.11):
//! - Keys are `usize` values returned by `slab.insert()`
//! - Keys may be reused after `slab.remove()`
//! - O(1) insert, remove, and lookup
//!
//! ## Linked List
//!
//! Orders at the same price level form a doubly-linked list:
//! - `next`: Points to the next order (newer) in the price level
//! - `prev`: Points to the previous order (older) in the price level
//!
//! This allows O(1) removal from anywhere in the list.

use crate::types::Order;

/// Order node stored in the slab.
///
/// Contains the order data plus linked-list pointers for the price level queue.
/// The pointers are slab keys (`usize`), not direct references.
///
/// ## Memory Layout
///
/// ```text
/// OrderNode {
///     order: Order (50 bytes SSZ)
///     next: Option<usize> (16 bytes with alignment)
///     prev: Option<usize> (16 bytes with alignment)
/// }
/// ```
#[derive(Debug, Clone)]
pub struct OrderNode {
    /// The actual order data
    pub order: Order,
    
    /// Next order in the price level queue (slab key)
    /// None if this is the tail (newest order)
    pub next: Option<usize>,
    
    /// Previous order in the price level queue (slab key)
    /// None if this is the head (oldest order)
    pub prev: Option<usize>,
}

impl OrderNode {
    /// Create a new order node (not yet linked)
    ///
    /// # Arguments
    ///
    /// * `order` - The order to wrap
    ///
    /// # Example
    ///
    /// ```
    /// use dark_hypercore::orderbook::OrderNode;
    /// use dark_hypercore::types::{Order, Side};
    ///
    /// let order = Order::new(1, 100, Side::Buy, 5_000_000_000_000, 100_000_000, 0);
    /// let node = OrderNode::new(order);
    ///
    /// assert!(node.next.is_none());
    /// assert!(node.prev.is_none());
    /// ```
    #[inline]
    pub fn new(order: Order) -> Self {
        Self {
            order,
            next: None,
            prev: None,
        }
    }
    
    /// Check if this node is unlinked (not part of any price level)
    #[inline]
    pub fn is_unlinked(&self) -> bool {
        self.next.is_none() && self.prev.is_none()
    }
    
    /// Get the order ID
    #[inline]
    pub fn order_id(&self) -> u64 {
        self.order.id
    }
    
    /// Get the order price
    #[inline]
    pub fn price(&self) -> u64 {
        self.order.price
    }
    
    /// Get the remaining quantity
    #[inline]
    pub fn remaining(&self) -> u64 {
        self.order.remaining
    }
    
    /// Fill a portion of this order
    ///
    /// # Returns
    ///
    /// The actual quantity filled (may be less than requested)
    #[inline]
    pub fn fill(&mut self, quantity: u64) -> u64 {
        self.order.fill(quantity)
    }
    
    /// Check if the order is fully filled
    #[inline]
    pub fn is_filled(&self) -> bool {
        self.order.is_filled()
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Side;
    
    fn create_test_order(id: u64, price: u64, quantity: u64) -> Order {
        Order::new(id, 100, Side::Buy, price, quantity, 0)
    }
    
    #[test]
    fn test_order_node_new() {
        let order = create_test_order(1, 5_000_000_000_000, 100_000_000);
        let node = OrderNode::new(order.clone());
        
        assert_eq!(node.order, order);
        assert!(node.next.is_none());
        assert!(node.prev.is_none());
        assert!(node.is_unlinked());
    }
    
    #[test]
    fn test_order_node_accessors() {
        let order = create_test_order(42, 5_000_000_000_000, 100_000_000);
        let node = OrderNode::new(order);
        
        assert_eq!(node.order_id(), 42);
        assert_eq!(node.price(), 5_000_000_000_000);
        assert_eq!(node.remaining(), 100_000_000);
        assert!(!node.is_filled());
    }
    
    #[test]
    fn test_order_node_fill() {
        let order = create_test_order(1, 5_000_000_000_000, 100_000_000);
        let mut node = OrderNode::new(order);
        
        // Partial fill
        let filled = node.fill(30_000_000);
        assert_eq!(filled, 30_000_000);
        assert_eq!(node.remaining(), 70_000_000);
        assert!(!node.is_filled());
        
        // Complete fill
        let filled = node.fill(70_000_000);
        assert_eq!(filled, 70_000_000);
        assert_eq!(node.remaining(), 0);
        assert!(node.is_filled());
    }
    
    #[test]
    fn test_order_node_linking() {
        let order = create_test_order(1, 5_000_000_000_000, 100_000_000);
        let mut node = OrderNode::new(order);
        
        assert!(node.is_unlinked());
        
        // Link to other nodes
        node.next = Some(2);
        assert!(!node.is_unlinked());
        
        node.prev = Some(0);
        assert!(!node.is_unlinked());
        
        // Only one link
        node.next = None;
        assert!(!node.is_unlinked());
    }
}

