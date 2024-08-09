use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex, MutexGuard};
use crate::frame::Frame;
use crate::identifier;

pub trait CanListener<F: Frame, Channel: Eq>: Send {
    /// Callback when frame transmit success.
    fn on_frame_transmitted(&mut self, id: identifier::Id, channel: Channel);
    /// Callback when frames received.
    fn on_frame_received(&mut self, frames: &Vec<F>, channel: Channel);
}

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
        listener: Box<dyn CanListener<Self::Frame, Self::Channel>>,
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

pub trait SyncCanDevice {
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
        listener: Box<dyn CanListener<Self::Frame, Self::Channel>>,
    ) -> bool;
    /// Unregister transmit and receive frame listener.
    fn unregister_listener(&mut self, name: String) -> bool;
    /// Unregister all transmit and receive frame listeners.
    fn unregister_all(&mut self) -> bool;
    /// Get all transmit and receive frame listener's names.
    fn listener_names(&self) -> Vec<String>;
    /// start transmit loop.
    fn sync_transmit(device: MutexGuard<Self>,
                     interval_ms: u64,
                     stopper: Arc<Mutex<Receiver<()>>>,
    );
    /// start receive loop.
    fn sync_receive(device: MutexGuard<Self>,
                    interval_ms: u64,
                    stopper: Arc<Mutex<Receiver<()>>>,
    );
    /// start `sync_transmit` and `sync_receive`
    fn sync_start(&mut self, interval_ms: u64);
    /// Close the device and stop transmit and receive loop.
    fn close(&mut self);
}

