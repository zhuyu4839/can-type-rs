use std::fmt::format;
use crate::constant::{EFF_MASK, SFF_MASK};
use crate::j1939::J1939Id;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Id {
    Standard(u16),
    Extended(u32),
    J1939(J1939Id),
}

unsafe impl Send for Id {}

impl Id {
    #[inline]
    pub fn from_bits(bits: u32, extended: bool) -> Self {
        let bits = bits & EFF_MASK;
        if extended {
            Self::Extended(bits)
        }
        else {
            if bits & (!SFF_MASK & EFF_MASK) > 0 {
                Self::Extended(bits)
            }
            else {
                Self::Standard(bits as u16)
            }
        }
    }

    #[inline]
    pub fn from_hex(hex_str: &str, extended: bool) -> Self {
        let bits = u32::from_str_radix(hex_str, 16).unwrap_or_default();
        Self::from_bits(bits, extended)
    }

    #[inline]
    pub fn try_from_bits(bits: u32, extended: bool) -> Option<Self> {
        match bits {
            0..=EFF_MASK => Some(Self::from_bits(bits, extended)),
            _ => None,
        }
    }

    #[inline]
    pub fn try_from_hex(hex_str: &str, extended: bool) -> Option<Self> {
        match u32::from_str_radix(hex_str, 16) {
            Ok(v) => Self::try_from_bits(v, extended),
            Err(_) => None
        }
    }

    #[inline]
    pub fn into_bits(self) -> u32 {
        match self {
            Self::Standard(v) => v as u32,
            Self::Extended(v) => v,
            Self::J1939(v) => v.into_bits(),
        }
    }

    #[inline]
    pub fn into_hex(self) -> String {
        format(format_args!("{:08X}", self.into_bits()))
    }

    /// Returns this CAN Identifier as a raw 32-bit integer.
    #[inline]
    #[must_use]
    pub fn as_raw(self) -> u32 {
        self.into_bits()
    }

    /// Returns the Base ID part of this extended identifier.
    #[inline]
    #[must_use]
    pub fn standard_id(self) -> Self {
        match self {
            Self::Standard(_) => self.clone(),
            Self::Extended(v) => Self::Standard((v >> 18) as u16),     // ID-28 to ID-18
            Self::J1939(v) => Self::Standard((v.into_bits() >> 18) as u16)
        }
    }

    #[inline]
    pub fn is_extended(&self) -> bool {
        match self {
            Self::Standard(_) => false,
            Self::Extended(_) |
            Self::J1939(_) => true,
        }
    }
}
