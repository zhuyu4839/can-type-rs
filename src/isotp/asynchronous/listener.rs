use std::fmt::Display;
use isotp_rs::{IsoTpEvent, IsoTpFrame, IsoTpState};
use crate::frame::Frame;
use crate::device::Listener;
use crate::isotp::{CanIsoTpFrame, AsyncCanIsoTp};

impl<C, F> Listener<C, F> for AsyncCanIsoTp<C, F>
where
    C: Clone + Eq + Display + Send + Sync,
    F: Frame<Channel = C> + Clone + Send + Sync {

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
                log::debug!("ISO-TP(CAN async) received: {:?} on {}", frame.data(), channel);

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
                        log::warn!("ISO-TP(CAN async) - data convert to frame failed: {}", e);
                        self.state_append(IsoTpState::Error);
                        self.iso_tp_event(IsoTpEvent::ErrorOccurred(e));

                        break;
                    }
                }
            }
        }
    }
}