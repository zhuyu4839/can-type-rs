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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Protocol {
    Can2a,
    #[default]
    Can2b,
    J1939,
}

/// Transmit and Receive device trait.
pub trait CanDeviceSync {
    type Error;
    type Frame;
    type Channel;

    fn transmit_sync(&self, channel: Self::Channel, frames: Self::Frame, canfd: bool, _: Option<usize>)
        -> Result<usize, Self::Error>;

    fn receive_sync(&self, channel: Self::Channel, canfd: bool, timeout: Option<usize>)
        -> Result<Self::Frame, Self::Error>;
}

pub trait CanDeviceAsync {
    type Error;
    type Frame;
    type Channel;

    fn transmit_async(&self, channel: Self::Channel, frames: Self::Frame, canfd: bool, _: Option<usize>)
        -> impl Future<Output = Result<usize, Self::Error>>;

    fn receive_async(&self, channel: Self::Channel, canfd: bool, timeout: Option<usize>)
        -> impl Future<Output = Result<Self::Frame, Self::Error>>;
}
