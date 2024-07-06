use std::fmt::format;
use bitfield_struct::bitfield;
use crate::constant::EFF_MASK;
use crate::Conversion;

/// Bitfield representation of an extended 29-bit CAN identifier.
///
/// ### Repr: `u32`
///
/// | Field                  | Size (bits) |
/// |------------------------|-------------|
/// | Padding bits (private) | 3           |
/// | Identifier bits        | 29          |
#[bitfield(u32, order = Msb)]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct Can2B {
    #[bits(3)]
    _padding_bits: u8,
    #[bits(29)]
    id_bits: u32,
}

impl Conversion for Can2B {
    type Type = u32;

    /// Creates a new 29-bit extended identifier from a 16-bit integer.
    ///
    /// # Examples
    /// ```rust
    /// use can_type_rs::Conversion;
    /// use can_type_rs::identifier::{Id, Can2B};
    /// let id_a = Can2B::from_bits(16711935);
    ///
    /// assert_eq!(0b00000_11111111_00000000_11111111, id_a.into_bits());
    /// ```
    #[inline]
    fn from_bits(bits: u32) -> Self {
        Can2B(bits)
    }

    /// Creates a new 29-bit extended identifier from a base-16 (hex) string slice.
    ///
    /// # Examples
    /// ```rust
    /// use can_type_rs::Conversion;
    /// use can_type_rs::identifier::{Id, Can2B};
    /// let id_a = Can2B::from_hex("00FF00FF");
    ///
    /// assert_eq!(0b00000_11111111_00000000_11111111, id_a.into_bits());
    /// ```
    #[inline]
    fn from_hex(hex_str: &str) -> Self {
        let bits = u32::from_str_radix(hex_str, 16).unwrap_or_default();
        Can2B(bits)
    }

    /// Creates a new 29-bit extended identifier from a 16-bit integer.
    ///
    /// # Errors
    /// - If value out of range for valid 11-bit identifiers
    ///
    /// # Examples
    /// ```rust
    /// use can_type_rs::Conversion;
    /// use can_type_rs::identifier::{Id, Can2B};
    /// let id_a = Can2B::try_from_bits(16711935).unwrap();
    /// let id_b = Can2B::try_from_bits(536870912);
    ///
    /// assert_eq!(0b00000_11111111_00000000_11111111, id_a.into_bits());
    /// assert!(id_b.is_none());
    /// ```
    #[inline]
    fn try_from_bits(bits: u32) -> Option<Self> {
        if bits > EFF_MASK { None }
        else { Some(Can2B(bits)) }
    }

    /// Creates a new 29-bit extended identifier from a base-16 (hex) string slice.
    ///
    /// # Errors
    /// - If failed to parse input hexadecimal string slice.
    /// - If value out of range for valid 11-bit identifiers
    ///
    /// # Examples
    /// ```rust
    /// use can_type_rs::Conversion;
    /// use can_type_rs::identifier::{Id, Can2B};
    /// let id_a = Can2B::try_from_hex("00FF00FF").unwrap();
    /// let id_b = Can2B::try_from_hex("20000000");
    ///
    /// assert_eq!(0b00000_11111111_00000000_11111111, id_a.into_bits());
    /// assert!(id_b.is_none());
    /// ```
    #[inline]
    fn try_from_hex(hex_str: &str) -> Option<Self> {
        match u32::from_str_radix(hex_str, 16) {
            Ok(bits) => Self::try_from_bits(bits),
            Err(_) => None,
        }
    }

    /// Creates a new 16-bit integer from the 29-bit extended identifier.
    ///
    /// # Examples
    /// ```rust
    /// use can_type_rs::Conversion;
    /// use can_type_rs::identifier::{Id, Can2B};
    /// let id_a = Can2B::from_bits(16711935);
    ///
    /// assert_eq!(16711935, id_a.into_bits());
    /// ```
    #[inline]
    fn into_bits(self) -> u32 {
        self.into_bits()
    }

    /// Creates a new base-16 (hex) `String` from the 29-bit extended identifier.
    ///
    /// # Requires
    /// - `alloc`
    ///
    /// # Examples
    /// ```rust
    /// use can_type_rs::Conversion;
    /// use can_type_rs::identifier::{Id, Can2B};
    /// let id_a = Can2B::from_bits(16711935);
    ///
    /// assert_eq!("00FF00FF", id_a.into_hex());
    /// ```
    #[inline]
    fn into_hex(self) -> String {
        format(format_args!("{:08X}", self.into_bits()))
    }
}


