mod asynchronous;
pub use asynchronous::*;
mod synchronous;
pub use synchronous::*;


use crate::identifier;

pub trait CanListener: Send {
    type Frame;
    type Channel;
    /// Callback when frame transmit success.
    fn on_frame_transmitted(&mut self, id: identifier::Id, channel: Self::Channel);
    /// Callback when frames received.
    fn on_frame_received(&mut self, frames: &Vec<Self::Frame>, channel: Self::Channel);
}

