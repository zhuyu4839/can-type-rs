use std::sync::{Arc, mpsc::{Sender, Receiver}, Mutex, MutexGuard};
use super::CanListener;

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
        listener: Box<dyn CanListener<Frame= Self::Frame, Channel= Self::Channel>>,
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


