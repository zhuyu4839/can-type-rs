use std::fmt::Display;
use isotp_rs::{IsoTpEvent, IsoTpFrame, IsoTpState};
use crate::device::Listener;
use crate::frame::Frame;
use crate::isotp::{CanIsoTpFrame, SyncCanIsoTp};

impl<C, F> Listener<C, F> for SyncCanIsoTp<C, F>
where
    C: Clone + Eq + Display,
    F: Frame<Channel = C> + Clone {

    fn on_frame_transmitting(&mut self, _: C, _: &F) {

    }
    fn on_frame_transmitted(&mut self, channel: C, id: u32) {
        if channel != self.channel {
            return;
        }

        if id == self.address.tx_id ||
            id == self.address.fid {
            self.state_remove(IsoTpState::Sending);
        }
    }

    fn on_frame_received(&mut self, channel: C, frames: &[F]) {
        if channel != self.channel
            || self.state_contains(IsoTpState::Error) {
            return;
        }

        let rx_id = self.address.rx_id;
        for frame in frames {
            if frame.id(false).as_raw() == rx_id {
                log::debug!("ISO-TP(CAN sync) received: {:?} on {}", frame.data(), channel);

                match CanIsoTpFrame::decode(frame.data()) {
                    Ok(frame) => match frame {
                        CanIsoTpFrame::SingleFrame { data } => {
                            self.on_single_frame(data);
                        }
                        CanIsoTpFrame::FirstFrame { length, data } => {
                            self.on_first_frame(length, data);
                        }
                        CanIsoTpFrame::ConsecutiveFrame { sequence, data } => {
                            self.on_consecutive_frame(sequence, data);
                        },
                        CanIsoTpFrame::FlowControlFrame(ctx) => {
                            self.on_flow_ctrl_frame(ctx);
                        },
                    },
                    Err(e) => {
                        log::warn!("ISO-TP(CAN sync) - data convert to frame failed: {}", e);
                        self.state_append(IsoTpState::Error);
                        self.iso_tp_event(IsoTpEvent::ErrorOccurred(e));

                        break;
                    }
                }
            }
        }
    }
}