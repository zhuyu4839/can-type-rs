mod can2a;
mod can2b;
mod j1939;

use crate::constant::SFF_MASK;
pub use crate::identifier::can2a::Can2A;
pub use crate::identifier::can2b::Can2B;
pub use crate::identifier::j1939::J1939;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Id {
    Can2A(Can2A),
    Can2B(Can2B),
    J1939(J1939),
}

impl Id {
    /// Returns this CAN Identifier as a raw 32-bit integer.
    #[inline]
    #[must_use]
    pub const fn as_raw(&self) -> u32 {
        match self {
            Self::Can2A(v) => v.into_bits() as u32,
            Self::Can2B(v) => v.into_bits(),
            Self::J1939(v) => v.into_bits(),
        }
    }

    /// Returns the Base ID part of this extended identifier.
    #[inline]
    #[must_use]
    pub fn standard_id(&self) -> Self {
        match self {
            Self::Can2A(_) => self.clone(),
            Self::Can2B(v) => Self::Can2A(Can2A::from_bits((v.into_bits() >> 18) as u16)),     // ID-28 to ID-18
            Self::J1939(v)  => Self::Can2A(Can2A::from_bits((v.into_bits() >> 18) as u16)),     // ID-28 to ID-18
        }
    }

    #[inline]
    pub fn is_extended(&self) -> bool {
        match self {
            Self::Can2A(_) => false,
            Self::Can2B(v) => (v.into_bits() & !SFF_MASK) > 0,
            Self::J1939(v) => (v.into_bits() & !SFF_MASK) > 0,
        }
    }
}
