mod context;
#[cfg(feature = "async")]
mod asynchronous;
#[cfg(feature = "async")]
pub use asynchronous::AsyncCanIsoTp;
mod synchronous;
pub use synchronous::SyncCanIsoTp;

// use context::IsoTpContext;
// use std::fmt::Display;
// use std::hash::Hash;
// // use isotp_rs::error::Error as IsoTpError;
// use isotp_rs::{FlowControlContext, IsoTpFrame, IsoTpState};
// use crate::frame::Frame as CanFrame;
// use crate::device::CanListener;
// use crate::identifier::Id;
// use super::CanIsoTpFrame;
//
//
// #[derive(Debug, Clone)]
// pub(crate) struct CanIsoTpInner<Channel, Frame>
// where
//     Channel: Display + Clone + Hash + Eq,
//     Frame: CanFrame + Clone, {
//     pub(crate) context: IsoTpContext<Channel>,
//     pub(crate) _marker: std::marker::PhantomData<Frame>,
// }
//
// unsafe impl<Channel, Frame> Send for CanIsoTpInner<Channel, Frame>
// where
//     Channel: Display + Clone + Hash + Eq,
//     Frame: CanFrame + Clone, {
//
// }
//
// impl<Channel, Frame> CanIsoTpInner<Channel, Frame>
// where
//     Channel: Display + Clone + Hash + Eq,
//     Frame: CanFrame + Clone {
//     fn on_single_frame(&mut self, channel: &Channel, data: Vec<u8>) {
//         if let Err(e) = self.context.on_single_frame(channel, data) {
//             log::warn!("{}", e);
//         }
//     }
//
//     fn on_first_frame(&mut self, channel: &Channel, length: u16, data: Vec<u8>) {
//         if let Err(e) = self.context.on_first_frame(channel, length, data) {
//             log::warn!("{}", e);
//         }
//     }
//
//     fn on_consecutive_frame(&mut self, channel: &Channel, sequence: u8, data: Vec<u8>) {
//         if let Err(e) = self.context.on_consecutive_frame(channel, sequence, data) {
//             log::warn!("{}", e);
//         }
//     }
//
//     fn on_flow_ctrl_frame(&mut self, channel: &Channel, ctx: FlowControlContext) {
//         if let Err(e) = self.context.on_flow_ctrl_frame(channel, ctx) {
//             log::warn!("{}", e);
//         }
//     }
// }

