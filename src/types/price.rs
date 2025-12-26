//! Fixed-point price and quantity utilities.
//!
//! ## Overview
//!
//! All prices and quantities in Dark HyperCore use fixed-point representation
//! to avoid floating-point errors. Values are stored as u64 scaled by 10^8.
//!
//! ## Why Fixed-Point?
//!
//! Floating-point arithmetic can produce different results on different hardware,
//! breaking determinism. Fixed-point ensures identical results everywhere.
//!
//! ## Scale Factor
//!
//! We use a scale factor of 10^8 (100,000,000), providing 8 decimal places.
//! This is sufficient for most financial applications.
//!
//! ## Examples
//!
//! ```
//! use dark_hypercore::types::price::{SCALE, to_fixed, from_fixed};
//!
//! // Convert 50000.12345678 to fixed-point
//! let price = to_fixed("50000.12345678").unwrap();
//! assert_eq!(price, 5_000_012_345_678);
//!
//! // Convert back to string
//! let s = from_fixed(price);
//! assert_eq!(s, "50000.12345678");
//! ```

use rust_decimal::prelude::*;
use rust_decimal::Decimal;

/// Scaling factor for fixed-point arithmetic: 10^8
///
/// This provides 8 decimal places of precision.
pub const SCALE: u64 = 100_000_000;

/// Maximum value that can be safely represented
/// 
/// u64::MAX / SCALE ≈ 184,467,440,737 (184 billion)
pub const MAX_VALUE: u64 = u64::MAX / SCALE;

// ============================================================================
// Conversion Functions
// ============================================================================

/// Convert a decimal string to fixed-point u64
///
/// # Arguments
///
/// * `s` - Decimal string (e.g., "50000.12345678")
///
/// # Returns
///
/// * `Some(u64)` - The fixed-point representation
/// * `None` - If parsing fails or value is out of range
///
/// # Example
///
/// ```
/// use dark_hypercore::types::price::to_fixed;
///
/// assert_eq!(to_fixed("1.0"), Some(100_000_000));
/// assert_eq!(to_fixed("50000.12345678"), Some(5_000_012_345_678));
/// assert_eq!(to_fixed("0.00000001"), Some(1));
/// ```
pub fn to_fixed(s: &str) -> Option<u64> {
    let decimal = Decimal::from_str(s).ok()?;
    decimal_to_fixed(decimal)
}

/// Convert a Decimal to fixed-point u64
///
/// # Arguments
///
/// * `d` - rust_decimal::Decimal value
///
/// # Returns
///
/// * `Some(u64)` - The fixed-point representation
/// * `None` - If value is negative or out of range
pub fn decimal_to_fixed(d: Decimal) -> Option<u64> {
    if d.is_sign_negative() {
        return None;
    }
    
    let scaled = d.checked_mul(Decimal::from(SCALE))?;
    let rounded = scaled.round_dp(0);
    rounded.to_u64()
}

/// Convert fixed-point u64 to a Decimal
///
/// # Arguments
///
/// * `value` - Fixed-point value
///
/// # Returns
///
/// The Decimal representation
pub fn fixed_to_decimal(value: u64) -> Decimal {
    Decimal::from(value) / Decimal::from(SCALE)
}

/// Convert fixed-point u64 to a string with 8 decimal places
///
/// # Arguments
///
/// * `value` - Fixed-point value
///
/// # Returns
///
/// String representation with 8 decimal places
///
/// # Example
///
/// ```
/// use dark_hypercore::types::price::from_fixed;
///
/// assert_eq!(from_fixed(100_000_000), "1.00000000");
/// assert_eq!(from_fixed(5_000_012_345_678), "50000.12345678");
/// ```
pub fn from_fixed(value: u64) -> String {
    let decimal = fixed_to_decimal(value);
    format!("{:.8}", decimal)
}

/// Convert fixed-point u64 to a human-readable string (trimmed trailing zeros)
///
/// # Example
///
/// ```
/// use dark_hypercore::types::price::from_fixed_trimmed;
///
/// assert_eq!(from_fixed_trimmed(100_000_000), "1");
/// assert_eq!(from_fixed_trimmed(150_000_000), "1.5");
/// assert_eq!(from_fixed_trimmed(123_456_789), "1.23456789");
/// ```
pub fn from_fixed_trimmed(value: u64) -> String {
    let decimal = fixed_to_decimal(value);
    let s = format!("{}", decimal.normalize());
    s
}

// ============================================================================
// Arithmetic Functions (using rust_decimal for safety)
// ============================================================================

/// Multiply two fixed-point values
///
/// This performs proper scaling to avoid overflow.
///
/// # Arguments
///
/// * `a` - First fixed-point value
/// * `b` - Second fixed-point value
///
/// # Returns
///
/// * `Some(u64)` - Result of a * b (properly scaled)
/// * `None` - If overflow occurs
///
/// # Example
///
/// ```
/// use dark_hypercore::types::price::checked_mul;
///
/// // 100.0 * 0.5 = 50.0
/// let a = 10_000_000_000u64; // 100.0
/// let b = 50_000_000u64;      // 0.5
/// assert_eq!(checked_mul(a, b), Some(5_000_000_000)); // 50.0
/// ```
pub fn checked_mul(a: u64, b: u64) -> Option<u64> {
    let da = fixed_to_decimal(a);
    let db = fixed_to_decimal(b);
    let result = da.checked_mul(db)?;
    decimal_to_fixed(result)
}

/// Divide two fixed-point values
///
/// # Arguments
///
/// * `a` - Dividend (fixed-point)
/// * `b` - Divisor (fixed-point)
///
/// # Returns
///
/// * `Some(u64)` - Result of a / b (properly scaled)
/// * `None` - If divisor is zero or overflow occurs
///
/// # Example
///
/// ```
/// use dark_hypercore::types::price::checked_div;
///
/// // 100.0 / 2.0 = 50.0
/// let a = 10_000_000_000u64; // 100.0
/// let b = 200_000_000u64;     // 2.0
/// assert_eq!(checked_div(a, b), Some(5_000_000_000)); // 50.0
/// ```
pub fn checked_div(a: u64, b: u64) -> Option<u64> {
    if b == 0 {
        return None;
    }
    
    let da = fixed_to_decimal(a);
    let db = fixed_to_decimal(b);
    let result = da.checked_div(db)?;
    decimal_to_fixed(result)
}

/// Add two fixed-point values
///
/// # Arguments
///
/// * `a` - First fixed-point value
/// * `b` - Second fixed-point value
///
/// # Returns
///
/// * `Some(u64)` - Result of a + b
/// * `None` - If overflow occurs
pub fn checked_add(a: u64, b: u64) -> Option<u64> {
    a.checked_add(b)
}

/// Subtract two fixed-point values
///
/// # Arguments
///
/// * `a` - First fixed-point value
/// * `b` - Second fixed-point value
///
/// # Returns
///
/// * `Some(u64)` - Result of a - b
/// * `None` - If underflow occurs
pub fn checked_sub(a: u64, b: u64) -> Option<u64> {
    a.checked_sub(b)
}

// ============================================================================
// Comparison Helpers
// ============================================================================

/// Compare two prices with a tolerance (for testing)
///
/// # Arguments
///
/// * `a` - First price
/// * `b` - Second price
/// * `tolerance` - Maximum allowed difference
///
/// # Returns
///
/// `true` if |a - b| <= tolerance
pub fn approx_eq(a: u64, b: u64, tolerance: u64) -> bool {
    if a >= b {
        a - b <= tolerance
    } else {
        b - a <= tolerance
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_scale_constant() {
        assert_eq!(SCALE, 100_000_000);
    }
    
    #[test]
    fn test_to_fixed_basic() {
        assert_eq!(to_fixed("1.0"), Some(100_000_000));
        assert_eq!(to_fixed("1"), Some(100_000_000));
        assert_eq!(to_fixed("0.5"), Some(50_000_000));
        assert_eq!(to_fixed("0.00000001"), Some(1));
        assert_eq!(to_fixed("50000.12345678"), Some(5_000_012_345_678));
    }
    
    #[test]
    fn test_to_fixed_edge_cases() {
        assert_eq!(to_fixed("0"), Some(0));
        assert_eq!(to_fixed("0.0"), Some(0));
        
        // Negative values should return None
        assert_eq!(to_fixed("-1.0"), None);
        
        // Invalid strings should return None
        assert_eq!(to_fixed("abc"), None);
        assert_eq!(to_fixed(""), None);
    }
    
    #[test]
    fn test_from_fixed() {
        assert_eq!(from_fixed(100_000_000), "1.00000000");
        assert_eq!(from_fixed(50_000_000), "0.50000000");
        assert_eq!(from_fixed(1), "0.00000001");
        assert_eq!(from_fixed(5_000_012_345_678), "50000.12345678");
        assert_eq!(from_fixed(0), "0.00000000");
    }
    
    #[test]
    fn test_from_fixed_trimmed() {
        assert_eq!(from_fixed_trimmed(100_000_000), "1");
        assert_eq!(from_fixed_trimmed(150_000_000), "1.5");
        assert_eq!(from_fixed_trimmed(123_456_789), "1.23456789");
    }
    
    #[test]
    fn test_roundtrip() {
        let values = ["1.0", "0.5", "50000.12345678", "0.00000001", "123456.78901234"];
        
        for s in values {
            let fixed = to_fixed(s).unwrap();
            let back = from_fixed(fixed);
            // Parse both to compare (handles trailing zeros)
            let original = Decimal::from_str(s).unwrap();
            let converted = Decimal::from_str(&back).unwrap();
            assert_eq!(original, converted, "Roundtrip failed for {}", s);
        }
    }
    
    #[test]
    fn test_checked_mul() {
        // 100.0 * 0.5 = 50.0
        let a = to_fixed("100.0").unwrap();
        let b = to_fixed("0.5").unwrap();
        let result = checked_mul(a, b).unwrap();
        assert_eq!(result, to_fixed("50.0").unwrap());
        
        // 2.0 * 3.0 = 6.0
        let a = to_fixed("2.0").unwrap();
        let b = to_fixed("3.0").unwrap();
        let result = checked_mul(a, b).unwrap();
        assert_eq!(result, to_fixed("6.0").unwrap());
    }
    
    #[test]
    fn test_checked_div() {
        // 100.0 / 2.0 = 50.0
        let a = to_fixed("100.0").unwrap();
        let b = to_fixed("2.0").unwrap();
        let result = checked_div(a, b).unwrap();
        assert_eq!(result, to_fixed("50.0").unwrap());
        
        // Division by zero should return None
        assert_eq!(checked_div(a, 0), None);
    }
    
    #[test]
    fn test_checked_add() {
        let a = to_fixed("100.0").unwrap();
        let b = to_fixed("50.5").unwrap();
        let result = checked_add(a, b).unwrap();
        assert_eq!(result, to_fixed("150.5").unwrap());
        
        // Overflow should return None
        assert_eq!(checked_add(u64::MAX, 1), None);
    }
    
    #[test]
    fn test_checked_sub() {
        let a = to_fixed("100.0").unwrap();
        let b = to_fixed("50.5").unwrap();
        let result = checked_sub(a, b).unwrap();
        assert_eq!(result, to_fixed("49.5").unwrap());
        
        // Underflow should return None
        assert_eq!(checked_sub(0, 1), None);
    }
    
    #[test]
    fn test_approx_eq() {
        assert!(approx_eq(100, 100, 0));
        assert!(approx_eq(100, 101, 1));
        assert!(approx_eq(101, 100, 1));
        assert!(!approx_eq(100, 102, 1));
    }
    
    #[test]
    fn test_precision() {
        // Verify we maintain 8 decimal places of precision
        let value = "123456789.12345678";
        let fixed = to_fixed(value).unwrap();
        let back = from_fixed(fixed);
        assert_eq!(back, value);
    }
}

