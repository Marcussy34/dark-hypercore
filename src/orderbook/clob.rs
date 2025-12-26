//! Central Limit Order Book (CLOB) implementation.
//!
//! ## Architecture
//!
//! The CLOB uses a hybrid data structure for optimal performance:
//!
//! - **Slab**: Pre-allocated storage for O(1) order operations
//! - **BTreeMap**: Sorted price levels for efficient best bid/ask lookup
//! - **HashMap**: Order ID to slab key mapping for O(1) cancel
//!
//! ## Price Ordering
//!
//! - **Bids** (buy orders): Sorted high-to-low (best bid = highest price)
//! - **Asks** (sell orders): Sorted low-to-high (best ask = lowest price)
//!
//! ## Memory Model
//!
//! Per slab docs (https://docs.rs/slab/0.4.11):
//! - `Slab::with_capacity(n)` pre-allocates n slots
//! - Keys are reused after removal
//! - O(1) insert, remove, and lookup
//!
//! ## Example
//!
//! ```
//! use dark_hypercore::orderbook::CLOB;
//! use dark_hypercore::types::{Order, Side};
//!
//! let mut clob = CLOB::with_capacity(10_000);
//!
//! // Add orders
//! let buy_order = Order::new(1, 100, Side::Buy, 5_000_000_000_000, 100_000_000, 0);
//! let sell_order = Order::new(2, 101, Side::Sell, 5_100_000_000_000, 100_000_000, 0);
//!
//! clob.add_order(buy_order);
//! clob.add_order(sell_order);
//!
//! assert_eq!(clob.best_bid(), Some(5_000_000_000_000));
//! assert_eq!(clob.best_ask(), Some(5_100_000_000_000));
//! assert_eq!(clob.spread(), Some(100_000_000_000));
//! ```

use std::cmp::Reverse;
use std::collections::{BTreeMap, HashMap};
use slab::Slab;

use crate::orderbook::{OrderNode, PriceLevel};
use crate::types::{Order, Side};

/// Central Limit Order Book
///
/// A high-performance order book using slab allocation for O(1) operations.
#[derive(Debug)]
pub struct CLOB {
    /// Pre-allocated order storage
    /// Key: slab index, Value: OrderNode
    orders: Slab<OrderNode>,
    
    /// Bid price levels (sorted high to low)
    /// Key: Reverse(price) for descending order
    /// Value: PriceLevel containing order queue
    bids: BTreeMap<Reverse<u64>, PriceLevel>,
    
    /// Ask price levels (sorted low to high)
    /// Key: price for ascending order
    /// Value: PriceLevel containing order queue
    asks: BTreeMap<u64, PriceLevel>,
    
    /// Order ID to slab key mapping (for O(1) cancel)
    order_index: HashMap<u64, usize>,
    
    /// Next order ID (for auto-assignment)
    next_order_id: u64,
    
    /// Next trade ID
    next_trade_id: u64,
    
    /// Total number of bid orders
    bid_count: usize,
    
    /// Total number of ask orders
    ask_count: usize,
}

impl Default for CLOB {
    fn default() -> Self {
        Self::new()
    }
}

impl CLOB {
    /// Create a new empty CLOB
    pub fn new() -> Self {
        Self {
            orders: Slab::new(),
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            order_index: HashMap::new(),
            next_order_id: 1,
            next_trade_id: 1,
            bid_count: 0,
            ask_count: 0,
        }
    }
    
    /// Create a CLOB with pre-allocated capacity
    ///
    /// # Arguments
    ///
    /// * `order_capacity` - Number of orders to pre-allocate
    ///
    /// # Example
    ///
    /// ```
    /// use dark_hypercore::orderbook::CLOB;
    ///
    /// let clob = CLOB::with_capacity(100_000);
    /// assert!(clob.capacity() >= 100_000);
    /// ```
    pub fn with_capacity(order_capacity: usize) -> Self {
        Self {
            orders: Slab::with_capacity(order_capacity),
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            order_index: HashMap::with_capacity(order_capacity),
            next_order_id: 1,
            next_trade_id: 1,
            bid_count: 0,
            ask_count: 0,
        }
    }
    
    // ========================================================================
    // Capacity and Size
    // ========================================================================
    
    /// Get the current capacity (pre-allocated slots)
    #[inline]
    pub fn capacity(&self) -> usize {
        self.orders.capacity()
    }
    
    /// Get the total number of orders in the book
    #[inline]
    pub fn order_count(&self) -> usize {
        self.orders.len()
    }
    
    /// Get the number of bid orders
    #[inline]
    pub fn bid_count(&self) -> usize {
        self.bid_count
    }
    
    /// Get the number of ask orders
    #[inline]
    pub fn ask_count(&self) -> usize {
        self.ask_count
    }
    
    /// Check if the order book is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.orders.is_empty()
    }
    
    /// Get the number of bid price levels
    #[inline]
    pub fn bid_levels(&self) -> usize {
        self.bids.len()
    }
    
    /// Get the number of ask price levels
    #[inline]
    pub fn ask_levels(&self) -> usize {
        self.asks.len()
    }
    
    // ========================================================================
    // Order Management
    // ========================================================================
    
    /// Add an order to the book
    ///
    /// The order is placed at the appropriate price level based on its side.
    ///
    /// # Arguments
    ///
    /// * `order` - The order to add
    ///
    /// # Returns
    ///
    /// The slab key for the added order
    ///
    /// # Example
    ///
    /// ```
    /// use dark_hypercore::orderbook::CLOB;
    /// use dark_hypercore::types::{Order, Side};
    ///
    /// let mut clob = CLOB::with_capacity(100);
    /// let order = Order::new(1, 100, Side::Buy, 5_000_000_000_000, 100_000_000, 0);
    /// let key = clob.add_order(order);
    ///
    /// assert_eq!(clob.order_count(), 1);
    /// ```
    pub fn add_order(&mut self, mut order: Order) -> usize {
        // Auto-assign order ID if not set
        if order.id == 0 {
            order.id = self.next_order_id;
            self.next_order_id += 1;
        }
        
        let order_id = order.id;
        let price = order.price;
        let side = order.side();
        
        // Create and insert the order node
        let node = OrderNode::new(order);
        let key = self.orders.insert(node);
        
        // Index the order for O(1) cancel
        self.order_index.insert(order_id, key);
        
        // Add to the appropriate price level
        match side {
            Side::Buy => {
                let level = self.bids
                    .entry(Reverse(price))
                    .or_insert_with(|| PriceLevel::new(price));
                level.push_back(key, &mut self.orders);
                self.bid_count += 1;
            }
            Side::Sell => {
                let level = self.asks
                    .entry(price)
                    .or_insert_with(|| PriceLevel::new(price));
                level.push_back(key, &mut self.orders);
                self.ask_count += 1;
            }
        }
        
        key
    }
    
    /// Remove an order by slab key
    ///
    /// # Arguments
    ///
    /// * `key` - The slab key for the order
    ///
    /// # Returns
    ///
    /// The removed order, or None if not found
    pub fn remove_order(&mut self, key: usize) -> Option<Order> {
        // Get order info before removal
        let node = self.orders.get(key)?;
        let order_id = node.order_id();
        let price = node.price();
        let side = node.order.side();
        
        // Remove from price level
        match side {
            Side::Buy => {
                if let Some(level) = self.bids.get_mut(&Reverse(price)) {
                    level.remove(key, &mut self.orders);
                    self.bid_count -= 1;
                    
                    // Remove empty price levels
                    if level.is_empty() {
                        self.bids.remove(&Reverse(price));
                    }
                }
            }
            Side::Sell => {
                if let Some(level) = self.asks.get_mut(&price) {
                    level.remove(key, &mut self.orders);
                    self.ask_count -= 1;
                    
                    // Remove empty price levels
                    if level.is_empty() {
                        self.asks.remove(&price);
                    }
                }
            }
        }
        
        // Remove from index
        self.order_index.remove(&order_id);
        
        // Remove from slab and return the order
        Some(self.orders.remove(key).order)
    }
    
    /// Cancel an order by order ID
    ///
    /// # Arguments
    ///
    /// * `order_id` - The unique order identifier
    ///
    /// # Returns
    ///
    /// The cancelled order, or None if not found
    ///
    /// # Example
    ///
    /// ```
    /// use dark_hypercore::orderbook::CLOB;
    /// use dark_hypercore::types::{Order, Side};
    ///
    /// let mut clob = CLOB::with_capacity(100);
    /// let order = Order::new(42, 100, Side::Buy, 5_000_000_000_000, 100_000_000, 0);
    /// clob.add_order(order);
    ///
    /// let cancelled = clob.cancel_order(42);
    /// assert!(cancelled.is_some());
    /// assert_eq!(clob.order_count(), 0);
    /// ```
    pub fn cancel_order(&mut self, order_id: u64) -> Option<Order> {
        let key = *self.order_index.get(&order_id)?;
        self.remove_order(key)
    }
    
    /// Get a reference to an order by slab key
    #[inline]
    pub fn get_order(&self, key: usize) -> Option<&Order> {
        self.orders.get(key).map(|node| &node.order)
    }
    
    /// Get a mutable reference to an order by slab key
    #[inline]
    pub fn get_order_mut(&mut self, key: usize) -> Option<&mut Order> {
        self.orders.get_mut(key).map(|node| &mut node.order)
    }
    
    /// Get the slab key for an order ID
    #[inline]
    pub fn get_key(&self, order_id: u64) -> Option<usize> {
        self.order_index.get(&order_id).copied()
    }
    
    /// Check if an order exists
    #[inline]
    pub fn contains_order(&self, order_id: u64) -> bool {
        self.order_index.contains_key(&order_id)
    }
    
    // ========================================================================
    // Best Bid/Ask
    // ========================================================================
    
    /// Get the best bid price (highest buy price)
    ///
    /// # Returns
    ///
    /// The best bid price, or None if no bids exist
    #[inline]
    pub fn best_bid(&self) -> Option<u64> {
        self.bids.keys().next().map(|r| r.0)
    }
    
    /// Get the best ask price (lowest sell price)
    ///
    /// # Returns
    ///
    /// The best ask price, or None if no asks exist
    #[inline]
    pub fn best_ask(&self) -> Option<u64> {
        self.asks.keys().next().copied()
    }
    
    /// Get the spread (best_ask - best_bid)
    ///
    /// # Returns
    ///
    /// The spread, or None if either side is empty
    pub fn spread(&self) -> Option<u64> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) if ask >= bid => Some(ask - bid),
            _ => None,
        }
    }
    
    /// Get the best bid price level
    pub fn best_bid_level(&self) -> Option<&PriceLevel> {
        self.bids.values().next()
    }
    
    /// Get the best ask price level
    pub fn best_ask_level(&self) -> Option<&PriceLevel> {
        self.asks.values().next()
    }
    
    /// Get the best bid price level (mutable)
    pub fn best_bid_level_mut(&mut self) -> Option<&mut PriceLevel> {
        self.bids.values_mut().next()
    }
    
    /// Get the best ask price level (mutable)
    pub fn best_ask_level_mut(&mut self) -> Option<&mut PriceLevel> {
        self.asks.values_mut().next()
    }
    
    // ========================================================================
    // Order Book Access (for matching engine)
    // ========================================================================
    
    /// Get a reference to the orders slab
    #[inline]
    pub fn orders(&self) -> &Slab<OrderNode> {
        &self.orders
    }
    
    /// Get a mutable reference to the orders slab
    #[inline]
    pub fn orders_mut(&mut self) -> &mut Slab<OrderNode> {
        &mut self.orders
    }
    
    /// Get a reference to the bids
    #[inline]
    pub fn bids(&self) -> &BTreeMap<Reverse<u64>, PriceLevel> {
        &self.bids
    }
    
    /// Get a mutable reference to the bids
    #[inline]
    pub fn bids_mut(&mut self) -> &mut BTreeMap<Reverse<u64>, PriceLevel> {
        &mut self.bids
    }
    
    /// Get a reference to the asks
    #[inline]
    pub fn asks(&self) -> &BTreeMap<u64, PriceLevel> {
        &self.asks
    }
    
    /// Get a mutable reference to the asks
    #[inline]
    pub fn asks_mut(&mut self) -> &mut BTreeMap<u64, PriceLevel> {
        &mut self.asks
    }
    
    // ========================================================================
    // ID Generation
    // ========================================================================
    
    /// Get the next trade ID and increment the counter
    #[inline]
    pub fn next_trade_id(&mut self) -> u64 {
        let id = self.next_trade_id;
        self.next_trade_id += 1;
        id
    }
    
    /// Get the current next order ID (without incrementing)
    #[inline]
    pub fn peek_next_order_id(&self) -> u64 {
        self.next_order_id
    }
    
    // ========================================================================
    // Cleanup Helpers
    // ========================================================================
    
    /// Remove an order from the slab (after it's already unlinked from price level)
    ///
    /// This is used by the matching engine after filling an order.
    #[inline]
    pub fn remove_from_slab(&mut self, key: usize) -> OrderNode {
        self.orders.remove(key)
    }
    
    /// Remove an empty bid price level
    pub fn remove_bid_level(&mut self, price: u64) {
        self.bids.remove(&Reverse(price));
    }
    
    /// Remove an empty ask price level
    pub fn remove_ask_level(&mut self, price: u64) {
        self.asks.remove(&price);
    }
    
    /// Remove order from index (used by matching engine)
    pub fn remove_from_index(&mut self, order_id: u64) {
        self.order_index.remove(&order_id);
    }
    
    /// Decrement bid count
    pub fn decrement_bid_count(&mut self) {
        self.bid_count = self.bid_count.saturating_sub(1);
    }
    
    /// Decrement ask count
    pub fn decrement_ask_count(&mut self) {
        self.ask_count = self.ask_count.saturating_sub(1);
    }
    
    /// Clear all orders from the book
    pub fn clear(&mut self) {
        self.orders.clear();
        self.bids.clear();
        self.asks.clear();
        self.order_index.clear();
        self.bid_count = 0;
        self.ask_count = 0;
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_buy_order(id: u64, price: u64, quantity: u64) -> Order {
        Order::new(id, 100, Side::Buy, price, quantity, 0)
    }
    
    fn create_sell_order(id: u64, price: u64, quantity: u64) -> Order {
        Order::new(id, 100, Side::Sell, price, quantity, 0)
    }
    
    #[test]
    fn test_clob_new() {
        let clob = CLOB::new();
        
        assert!(clob.is_empty());
        assert_eq!(clob.order_count(), 0);
        assert_eq!(clob.bid_count(), 0);
        assert_eq!(clob.ask_count(), 0);
        assert!(clob.best_bid().is_none());
        assert!(clob.best_ask().is_none());
    }
    
    #[test]
    fn test_clob_with_capacity() {
        let clob = CLOB::with_capacity(10_000);
        
        assert!(clob.capacity() >= 10_000);
        assert!(clob.is_empty());
    }
    
    #[test]
    fn test_clob_add_buy_order() {
        let mut clob = CLOB::with_capacity(100);
        
        let order = create_buy_order(1, 5_000_000_000_000, 100_000_000);
        let key = clob.add_order(order);
        
        assert_eq!(clob.order_count(), 1);
        assert_eq!(clob.bid_count(), 1);
        assert_eq!(clob.ask_count(), 0);
        assert_eq!(clob.best_bid(), Some(5_000_000_000_000));
        assert!(clob.best_ask().is_none());
        assert!(clob.orders.contains(key));
    }
    
    #[test]
    fn test_clob_add_sell_order() {
        let mut clob = CLOB::with_capacity(100);
        
        let order = create_sell_order(1, 5_100_000_000_000, 100_000_000);
        clob.add_order(order);
        
        assert_eq!(clob.order_count(), 1);
        assert_eq!(clob.bid_count(), 0);
        assert_eq!(clob.ask_count(), 1);
        assert!(clob.best_bid().is_none());
        assert_eq!(clob.best_ask(), Some(5_100_000_000_000));
    }
    
    #[test]
    fn test_clob_spread() {
        let mut clob = CLOB::with_capacity(100);
        
        // No spread without both sides
        assert!(clob.spread().is_none());
        
        clob.add_order(create_buy_order(1, 5_000_000_000_000, 100_000_000));
        assert!(clob.spread().is_none());
        
        clob.add_order(create_sell_order(2, 5_100_000_000_000, 100_000_000));
        assert_eq!(clob.spread(), Some(100_000_000_000)); // $1000 spread
    }
    
    #[test]
    fn test_clob_bid_price_priority() {
        let mut clob = CLOB::with_capacity(100);
        
        // Add bids at different prices (not in order)
        clob.add_order(create_buy_order(1, 4_900_000_000_000, 100_000_000)); // 49000
        clob.add_order(create_buy_order(2, 5_100_000_000_000, 100_000_000)); // 51000
        clob.add_order(create_buy_order(3, 5_000_000_000_000, 100_000_000)); // 50000
        
        // Best bid should be highest price
        assert_eq!(clob.best_bid(), Some(5_100_000_000_000));
        assert_eq!(clob.bid_levels(), 3);
    }
    
    #[test]
    fn test_clob_ask_price_priority() {
        let mut clob = CLOB::with_capacity(100);
        
        // Add asks at different prices (not in order)
        clob.add_order(create_sell_order(1, 5_200_000_000_000, 100_000_000)); // 52000
        clob.add_order(create_sell_order(2, 5_000_000_000_000, 100_000_000)); // 50000
        clob.add_order(create_sell_order(3, 5_100_000_000_000, 100_000_000)); // 51000
        
        // Best ask should be lowest price
        assert_eq!(clob.best_ask(), Some(5_000_000_000_000));
        assert_eq!(clob.ask_levels(), 3);
    }
    
    #[test]
    fn test_clob_cancel_order() {
        let mut clob = CLOB::with_capacity(100);
        
        clob.add_order(create_buy_order(42, 5_000_000_000_000, 100_000_000));
        assert_eq!(clob.order_count(), 1);
        
        let cancelled = clob.cancel_order(42);
        assert!(cancelled.is_some());
        assert_eq!(cancelled.unwrap().id, 42);
        assert_eq!(clob.order_count(), 0);
        assert!(clob.best_bid().is_none());
    }
    
    #[test]
    fn test_clob_cancel_nonexistent() {
        let mut clob = CLOB::with_capacity(100);
        
        let cancelled = clob.cancel_order(999);
        assert!(cancelled.is_none());
    }
    
    #[test]
    fn test_clob_contains_order() {
        let mut clob = CLOB::with_capacity(100);
        
        assert!(!clob.contains_order(42));
        
        clob.add_order(create_buy_order(42, 5_000_000_000_000, 100_000_000));
        assert!(clob.contains_order(42));
        
        clob.cancel_order(42);
        assert!(!clob.contains_order(42));
    }
    
    #[test]
    fn test_clob_multiple_orders_same_price() {
        let mut clob = CLOB::with_capacity(100);
        
        // Add multiple orders at the same price
        clob.add_order(create_buy_order(1, 5_000_000_000_000, 100_000_000));
        clob.add_order(create_buy_order(2, 5_000_000_000_000, 200_000_000));
        clob.add_order(create_buy_order(3, 5_000_000_000_000, 300_000_000));
        
        assert_eq!(clob.order_count(), 3);
        assert_eq!(clob.bid_levels(), 1); // All at same price level
        
        // Check total quantity at price level
        let level = clob.best_bid_level().unwrap();
        assert_eq!(level.total_quantity, 600_000_000);
        assert_eq!(level.order_count, 3);
    }
    
    #[test]
    fn test_clob_auto_order_id() {
        let mut clob = CLOB::with_capacity(100);
        
        // Create order with id=0 (auto-assign)
        let mut order = create_buy_order(0, 5_000_000_000_000, 100_000_000);
        order.id = 0;
        
        clob.add_order(order);
        
        // Should have been assigned ID 1
        assert!(clob.contains_order(1));
        assert_eq!(clob.peek_next_order_id(), 2);
    }
    
    #[test]
    fn test_clob_clear() {
        let mut clob = CLOB::with_capacity(100);
        
        clob.add_order(create_buy_order(1, 5_000_000_000_000, 100_000_000));
        clob.add_order(create_sell_order(2, 5_100_000_000_000, 100_000_000));
        
        assert_eq!(clob.order_count(), 2);
        
        clob.clear();
        
        assert!(clob.is_empty());
        assert_eq!(clob.bid_count(), 0);
        assert_eq!(clob.ask_count(), 0);
        assert!(clob.best_bid().is_none());
        assert!(clob.best_ask().is_none());
    }
    
    #[test]
    fn test_clob_remove_empty_level() {
        let mut clob = CLOB::with_capacity(100);
        
        clob.add_order(create_buy_order(1, 5_000_000_000_000, 100_000_000));
        clob.add_order(create_buy_order(2, 4_900_000_000_000, 100_000_000));
        
        assert_eq!(clob.bid_levels(), 2);
        
        // Cancel order at best bid price
        clob.cancel_order(1);
        
        // Price level should be removed
        assert_eq!(clob.bid_levels(), 1);
        assert_eq!(clob.best_bid(), Some(4_900_000_000_000));
    }
    
    #[test]
    fn test_clob_get_order() {
        let mut clob = CLOB::with_capacity(100);
        
        let order = create_buy_order(42, 5_000_000_000_000, 100_000_000);
        let key = clob.add_order(order);
        
        let retrieved = clob.get_order(key);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, 42);
        
        // Non-existent key
        assert!(clob.get_order(999).is_none());
    }
    
    #[test]
    fn test_clob_get_key() {
        let mut clob = CLOB::with_capacity(100);
        
        let order = create_buy_order(42, 5_000_000_000_000, 100_000_000);
        let key = clob.add_order(order);
        
        assert_eq!(clob.get_key(42), Some(key));
        assert!(clob.get_key(999).is_none());
    }
}

