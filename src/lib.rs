use std::sync::{Arc, mpsc, Mutex};
use crate::identifier::Id;

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
#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub enum Direct {
    #[default]
    Transmit,
    Receive,
}

pub trait AsyncCanDevice {
    type Frame;
    type Device;
    fn new(device: Self::Device) -> Self;
    /// Get the sender for transmit frame.
    fn sender(&self) -> mpsc::Sender<Self::Frame>;
    /// Register transmit and receive frame listener.
    fn register_listener(&mut self, name: String, listener: Box<dyn CanListener<Frame = Self::Frame>>) -> bool;
    /// Unregister transmit and receive frame listener.
    fn unregister_listener(&mut self, name: String) -> bool;
    /// Unregister all transmit and receive frame listeners.
    fn unregister_all(&mut self) -> bool;
    /// Get all transmit and receive frame listener's names.
    fn listener_names(&self) -> Vec<String>;
    /// start transmit loop.
    fn async_transmit(device: Arc<Mutex<Self>>, interval_ms: u64) -> impl std::future::Future<Output = ()> + Send;
    /// start receive loop.
    fn async_receive(device: Arc<Mutex<Self>>, interval_ms: u64) -> impl std::future::Future<Output = ()> + Send;
    /// start `async_transmit` and `async_receive`
    fn async_start(&self, interval_ms: u64);
    /// Close the device and stop transmit and receive loop.
    fn close(&mut self);
}

pub trait CanListener: Send + Sync {
    type Frame;
    /// Callback when frame transmit success.
    fn on_frame_transmitted(&self, id: Id);
    /// Callback when frames received.
    fn on_frame_received(&self, frames: &Vec<Self::Frame>);
}
