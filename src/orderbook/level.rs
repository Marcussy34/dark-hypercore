//! Price level management for orders at the same price.
//!
//! ## Design
//!
//! A `PriceLevel` represents all orders at a single price point.
//! Orders are maintained in a doubly-linked list for FIFO ordering
//! (price-time priority).
//!
//! ## Queue Structure
//!
//! ```text
//! head (oldest) <-> order2 <-> order3 <-> tail (newest)
//! ```
//!
//! - New orders are appended at the tail
//! - Matching consumes orders from the head
//! - Any order can be removed in O(1) using the slab key

use slab::Slab;
use crate::orderbook::OrderNode;

/// A price level containing orders at a single price.
///
/// Orders are stored in a FIFO queue (doubly-linked list).
/// The actual order data lives in the slab; this struct only
/// holds the queue metadata.
#[derive(Debug, Clone)]
pub struct PriceLevel {
    /// Price for this level (fixed-point, scaled by 10^8)
    pub price: u64,
    
    /// Total remaining quantity at this level
    /// Updated when orders are added/removed/filled
    pub total_quantity: u64,
    
    /// Head of the order queue (oldest order, slab key)
    /// This is the first order to be matched
    pub head: Option<usize>,
    
    /// Tail of the order queue (newest order, slab key)
    /// New orders are appended here
    pub tail: Option<usize>,
    
    /// Number of orders at this price level
    pub order_count: usize,
}

impl PriceLevel {
    /// Create a new empty price level
    ///
    /// # Arguments
    ///
    /// * `price` - The price for this level (fixed-point)
    pub fn new(price: u64) -> Self {
        Self {
            price,
            total_quantity: 0,
            head: None,
            tail: None,
            order_count: 0,
        }
    }
    
    /// Check if the price level is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.order_count == 0
    }
    
    /// Add an order to the tail of the queue
    ///
    /// This maintains FIFO ordering - oldest orders are matched first.
    ///
    /// # Arguments
    ///
    /// * `key` - The slab key for the order node
    /// * `slab` - The slab containing all order nodes
    ///
    /// # Panics
    ///
    /// Panics if the key doesn't exist in the slab
    pub fn push_back(&mut self, key: usize, slab: &mut Slab<OrderNode>) {
        let node = slab.get_mut(key).expect("Invalid slab key");
        let quantity = node.remaining();
        
        // Update linked list pointers
        node.prev = self.tail;
        node.next = None;
        
        if let Some(tail_key) = self.tail {
            // Link the old tail to the new node
            let tail_node = slab.get_mut(tail_key).expect("Invalid tail key");
            tail_node.next = Some(key);
        } else {
            // Empty list - this is also the head
            self.head = Some(key);
        }
        
        self.tail = Some(key);
        self.order_count += 1;
        self.total_quantity = self.total_quantity.saturating_add(quantity);
    }
    
    /// Remove an order from the queue by slab key
    ///
    /// # Arguments
    ///
    /// * `key` - The slab key for the order node to remove
    /// * `slab` - The slab containing all order nodes
    ///
    /// # Returns
    ///
    /// The remaining quantity of the removed order
    pub fn remove(&mut self, key: usize, slab: &mut Slab<OrderNode>) -> u64 {
        let node = slab.get(key).expect("Invalid slab key");
        let quantity = node.remaining();
        let prev_key = node.prev;
        let next_key = node.next;
        
        // Update the previous node's next pointer
        if let Some(prev) = prev_key {
            let prev_node = slab.get_mut(prev).expect("Invalid prev key");
            prev_node.next = next_key;
        } else {
            // This was the head
            self.head = next_key;
        }
        
        // Update the next node's prev pointer
        if let Some(next) = next_key {
            let next_node = slab.get_mut(next).expect("Invalid next key");
            next_node.prev = prev_key;
        } else {
            // This was the tail
            self.tail = prev_key;
        }
        
        // Clear the removed node's pointers
        let node = slab.get_mut(key).expect("Invalid slab key");
        node.prev = None;
        node.next = None;
        
        self.order_count -= 1;
        self.total_quantity = self.total_quantity.saturating_sub(quantity);
        
        quantity
    }
    
    /// Get the head order's slab key (oldest order)
    ///
    /// This is the first order to be matched at this price level.
    #[inline]
    pub fn peek_head(&self) -> Option<usize> {
        self.head
    }
    
    /// Update the total quantity after a partial fill
    ///
    /// # Arguments
    ///
    /// * `filled_quantity` - Amount that was filled
    pub fn reduce_quantity(&mut self, filled_quantity: u64) {
        self.total_quantity = self.total_quantity.saturating_sub(filled_quantity);
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Order, Side};
    
    fn create_test_node(slab: &mut Slab<OrderNode>, id: u64, quantity: u64) -> usize {
        let order = Order::new(id, 100, Side::Buy, 5_000_000_000_000, quantity, 0);
        slab.insert(OrderNode::new(order))
    }
    
    #[test]
    fn test_price_level_new() {
        let level = PriceLevel::new(5_000_000_000_000);
        
        assert_eq!(level.price, 5_000_000_000_000);
        assert_eq!(level.total_quantity, 0);
        assert!(level.head.is_none());
        assert!(level.tail.is_none());
        assert_eq!(level.order_count, 0);
        assert!(level.is_empty());
    }
    
    #[test]
    fn test_price_level_push_single() {
        let mut slab = Slab::with_capacity(10);
        let mut level = PriceLevel::new(5_000_000_000_000);
        
        let key = create_test_node(&mut slab, 1, 100_000_000);
        level.push_back(key, &mut slab);
        
        assert_eq!(level.order_count, 1);
        assert_eq!(level.total_quantity, 100_000_000);
        assert_eq!(level.head, Some(key));
        assert_eq!(level.tail, Some(key));
        assert!(!level.is_empty());
        
        // Node should have no links (it's the only one)
        let node = slab.get(key).unwrap();
        assert!(node.prev.is_none());
        assert!(node.next.is_none());
    }
    
    #[test]
    fn test_price_level_push_multiple() {
        let mut slab = Slab::with_capacity(10);
        let mut level = PriceLevel::new(5_000_000_000_000);
        
        let key1 = create_test_node(&mut slab, 1, 100_000_000);
        let key2 = create_test_node(&mut slab, 2, 200_000_000);
        let key3 = create_test_node(&mut slab, 3, 300_000_000);
        
        level.push_back(key1, &mut slab);
        level.push_back(key2, &mut slab);
        level.push_back(key3, &mut slab);
        
        assert_eq!(level.order_count, 3);
        assert_eq!(level.total_quantity, 600_000_000);
        assert_eq!(level.head, Some(key1));
        assert_eq!(level.tail, Some(key3));
        
        // Verify linked list structure: key1 <-> key2 <-> key3
        let node1 = slab.get(key1).unwrap();
        assert!(node1.prev.is_none());
        assert_eq!(node1.next, Some(key2));
        
        let node2 = slab.get(key2).unwrap();
        assert_eq!(node2.prev, Some(key1));
        assert_eq!(node2.next, Some(key3));
        
        let node3 = slab.get(key3).unwrap();
        assert_eq!(node3.prev, Some(key2));
        assert!(node3.next.is_none());
    }
    
    #[test]
    fn test_price_level_remove_middle() {
        let mut slab = Slab::with_capacity(10);
        let mut level = PriceLevel::new(5_000_000_000_000);
        
        let key1 = create_test_node(&mut slab, 1, 100_000_000);
        let key2 = create_test_node(&mut slab, 2, 200_000_000);
        let key3 = create_test_node(&mut slab, 3, 300_000_000);
        
        level.push_back(key1, &mut slab);
        level.push_back(key2, &mut slab);
        level.push_back(key3, &mut slab);
        
        // Remove middle node
        let removed_qty = level.remove(key2, &mut slab);
        
        assert_eq!(removed_qty, 200_000_000);
        assert_eq!(level.order_count, 2);
        assert_eq!(level.total_quantity, 400_000_000);
        assert_eq!(level.head, Some(key1));
        assert_eq!(level.tail, Some(key3));
        
        // Verify new linked list: key1 <-> key3
        let node1 = slab.get(key1).unwrap();
        assert!(node1.prev.is_none());
        assert_eq!(node1.next, Some(key3));
        
        let node3 = slab.get(key3).unwrap();
        assert_eq!(node3.prev, Some(key1));
        assert!(node3.next.is_none());
    }
    
    #[test]
    fn test_price_level_remove_head() {
        let mut slab = Slab::with_capacity(10);
        let mut level = PriceLevel::new(5_000_000_000_000);
        
        let key1 = create_test_node(&mut slab, 1, 100_000_000);
        let key2 = create_test_node(&mut slab, 2, 200_000_000);
        
        level.push_back(key1, &mut slab);
        level.push_back(key2, &mut slab);
        
        // Remove head
        level.remove(key1, &mut slab);
        
        assert_eq!(level.order_count, 1);
        assert_eq!(level.head, Some(key2));
        assert_eq!(level.tail, Some(key2));
        
        // key2 should now be unlinked (only element)
        let node2 = slab.get(key2).unwrap();
        assert!(node2.prev.is_none());
        assert!(node2.next.is_none());
    }
    
    #[test]
    fn test_price_level_remove_tail() {
        let mut slab = Slab::with_capacity(10);
        let mut level = PriceLevel::new(5_000_000_000_000);
        
        let key1 = create_test_node(&mut slab, 1, 100_000_000);
        let key2 = create_test_node(&mut slab, 2, 200_000_000);
        
        level.push_back(key1, &mut slab);
        level.push_back(key2, &mut slab);
        
        // Remove tail
        level.remove(key2, &mut slab);
        
        assert_eq!(level.order_count, 1);
        assert_eq!(level.head, Some(key1));
        assert_eq!(level.tail, Some(key1));
    }
    
    #[test]
    fn test_price_level_remove_only() {
        let mut slab = Slab::with_capacity(10);
        let mut level = PriceLevel::new(5_000_000_000_000);
        
        let key = create_test_node(&mut slab, 1, 100_000_000);
        level.push_back(key, &mut slab);
        
        level.remove(key, &mut slab);
        
        assert!(level.is_empty());
        assert_eq!(level.order_count, 0);
        assert_eq!(level.total_quantity, 0);
        assert!(level.head.is_none());
        assert!(level.tail.is_none());
    }
    
    #[test]
    fn test_price_level_reduce_quantity() {
        let mut level = PriceLevel::new(5_000_000_000_000);
        level.total_quantity = 1_000_000_000;
        
        level.reduce_quantity(300_000_000);
        assert_eq!(level.total_quantity, 700_000_000);
        
        // Saturating subtraction prevents underflow
        level.reduce_quantity(1_000_000_000);
        assert_eq!(level.total_quantity, 0);
    }
    
    #[test]
    fn test_price_level_peek_head() {
        let mut slab = Slab::with_capacity(10);
        let mut level = PriceLevel::new(5_000_000_000_000);
        
        assert!(level.peek_head().is_none());
        
        let key = create_test_node(&mut slab, 1, 100_000_000);
        level.push_back(key, &mut slab);
        
        assert_eq!(level.peek_head(), Some(key));
    }
}

