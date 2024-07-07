use std::future::Future;

pub mod constant;
pub mod frame;
pub mod identifier;
pub mod j1939;

pub trait Conversion
    where
        Self: Sized, {
    type Type;

    /// Convert an integer of type `Self::Type` into `Self`
    fn from_bits(bits: Self::Type) -> Self;

    /// Convert a hexadecimal string slice into `Self`
    fn from_hex(hex_str: &str) -> Self;

    /// Convert an integer of type `Self::Type` into `Self`
    /// # Errors
    /// - Implementation dependent
    fn try_from_bits(bits: Self::Type) -> Option<Self>;

    /// Convert a hexadecimal string slice into `Self`
    /// # Errors
    /// - Implementation dependent
    fn try_from_hex(hex_str: &str) -> Option<Self>;

    /// Convert `self` into an integer of type `Self::Type`
    fn into_bits(self) -> Self::Type;

    /// Convert `self` into a hexadecimal string
    fn into_hex(self) -> String;
}

#[repr(C)]
#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub enum Direct {
    #[default]
    Transmit,
    Receive,
}
