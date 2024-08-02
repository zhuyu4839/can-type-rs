use std::sync::{Arc, mpsc::{Sender, Receiver}, Mutex};
use super::CanListener;

pub trait AsyncCanDevice {
    type Channel;
    type Frame;
    type Device;

    fn new(device: Self::Device) -> Self;
    /// Get the sender for transmit frame.
    fn sender(&self) -> Sender<Self::Frame>;
    /// Register transmit and receive frame listener.
    fn register_listener(
        &mut self,
        name: String,
        listener: Box<dyn CanListener<Frame= Self::Frame, Channel= Self::Channel>>,
    ) -> bool;
    /// Unregister transmit and receive frame listener.
    fn unregister_listener(&mut self, name: String) -> bool;
    /// Unregister all transmit and receive frame listeners.
    fn unregister_all(&mut self) -> bool;
    /// Get all transmit and receive frame listener's names.
    fn listener_names(&self) -> Vec<String>;
    /// start transmit loop.
    fn async_transmit(device: Arc<Mutex<Self>>,
                      interval_ms: u64,
                      stopper: Arc<Mutex<Receiver<()>>>,
    ) -> impl std::future::Future<Output = ()> + Send;
    /// start receive loop.
    fn async_receive(device: Arc<Mutex<Self>>,
                     interval_ms: u64,
                     stopper: Arc<Mutex<Receiver<()>>>,
    ) -> impl std::future::Future<Output = ()> + Send;
    /// start `async_transmit` and `async_receive`
    fn async_start(&mut self, interval_ms: u64);
    /// Close the device and stop transmit and receive loop.
    fn close(&mut self) -> impl std::future::Future<Output = ()> + Send;
}

