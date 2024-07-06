use std::fmt::format;
use bitfield_struct::bitfield;
use crate::constant::SFF_MASK;
use crate::Conversion;

/// Bitfield representation of a standard 11-bit CAN identifier.
///
/// ### Repr: `u16`
///
/// | Field                  | Size (bits) |
/// |------------------------|-------------|
/// | Padding bits (private) | 5           |
/// | Identifier bits        | 11          |
#[bitfield(u16, order = Msb)]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct Can2A {
    #[bits(5)]
    _padding_bits: u8,
    #[bits(11)]
    id_bits: u16,
}

impl Conversion for Can2A {
    type Type = u16;

    /// Creates a new 11-bit standard identifier from a 16-bit integer.
    ///
    /// # Examples
    /// ```rust
    /// use can_type_rs::Conversion;
    /// use can_type_rs::identifier::{Id, Can2A};
    /// let id_a = Can2A::from_bits(15);
    ///
    /// assert_eq!(0b000_0000_1111, id_a.into_bits());
    /// ```
    #[inline]
    fn from_bits(bits: u16) -> Self {
        Can2A(bits)
    }

    /// Creates a new 11-bit standard identifier from a base-16 (hex) string slice.
    ///
    /// # Examples
    /// ```rust
    /// use can_type_rs::Conversion;
    /// use can_type_rs::identifier::{Id, Can2A};
    /// let id_a = Can2A::from_hex("00F");
    ///
    /// assert_eq!(0b000_0000_1111, id_a.into_bits());
    /// ```
    #[inline]
    fn from_hex(hex_str: &str) -> Self {
        let bits = u16::from_str_radix(hex_str, 16).unwrap_or_default();
        Can2A(bits)
    }

    /// Creates a new 11-bit standard identifier from a 16-bit integer.
    ///
    /// # Examples
    /// ```rust
    /// use can_type_rs::Conversion;
    /// use can_type_rs::identifier::{Id, Can2A};
    /// let id_a = Can2A::try_from_bits(15).unwrap();
    /// let id_b = Can2A::try_from_bits(2048);
    ///
    /// assert_eq!(0b000_0000_1111, id_a.into_bits());
    /// assert!(id_b.is_none());
    /// ```
    #[inline]
    fn try_from_bits(bits: u16) -> Option<Self> {
        if bits > SFF_MASK as u16 { None}
        else { Some(Can2A(bits)) }
    }

    /// Creates a new 11-bit standard identifier from a base-16 (hex) string slice.
    ///
    /// # Examples
    /// ```rust
    /// use can_type_rs::Conversion;
    /// use can_type_rs::identifier::{Id, Can2A};
    /// let id_a = Can2A::try_from_hex("00F").unwrap();
    /// let id_b = Can2A::try_from_hex("FFF");
    ///
    /// assert_eq!(0b000_0000_1111, id_a.into_bits());
    /// assert!(id_b.is_none());
    /// ```
    #[inline]
    fn try_from_hex(hex_str: &str) -> Option<Self> {
        match u16::from_str_radix(hex_str, 16) {
            Ok(bits) => Self::try_from_bits(bits),
            Err(_) => None,
        }

    }

    /// Creates a new 16-bit integer from the 11-bit standard identifier.
    /// # Examples
    /// ```rust
    /// use can_type_rs::Conversion;
    /// use can_type_rs::identifier::{Id, Can2A};
    /// let id_a = Can2A::from_bits(15);
    ///
    /// assert_eq!(15, id_a.into_bits());
    /// assert_eq!(0b000_0000_1111, id_a.into_bits());
    /// assert_eq!(0x00F, id_a.into_bits());
    /// ```
    #[inline]
    fn into_bits(self) -> u16 {
        self.into_bits()
    }

    /// Creates a new base-16 (hex) `String` from the 11-bit standard identifier.
    ///
    /// # Examples
    /// ```rust
    /// use can_type_rs::Conversion;
    /// use can_type_rs::identifier::{Id, Can2A};
    /// let id_a = Can2A::from_bits(15);
    ///
    /// assert_eq!("00F", id_a.into_hex());
    /// ```
    #[inline]
    fn into_hex(self) -> String {
        format(format_args!("{:03X}", self.into_bits()))
    }
}


