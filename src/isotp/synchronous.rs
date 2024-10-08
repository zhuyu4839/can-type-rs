mod listener;

use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;
use std::thread::sleep;
use std::time::Duration;
use isotp_rs::{FlowControlContext, FlowControlState, IsoTpEvent, IsoTpEventListener, IsoTpFrame, IsoTpState, can::{Address, CanIsoTpFrame}};
use isotp_rs::error::Error as IsoTpError;
use crate::frame::Frame;
use crate::identifier::Id;
use crate::isotp::context::IsoTpContext;

#[derive(Clone)]
pub struct SyncCanIsoTp<C, F> {
    pub(crate) channel: C,
    pub(crate) address: Address,
    pub(crate) sender: Sender<F>,
    pub(crate) context: IsoTpContext,
    pub(crate) state: Arc<Mutex<IsoTpState>>,
    pub(crate) listener: Arc<Mutex<Box<dyn IsoTpEventListener>>>,
}

unsafe impl<C, F> Send for SyncCanIsoTp<C, F> {}

impl<C: Clone, F: Frame<Channel = C>> SyncCanIsoTp<C, F> {

    pub fn new(channel: C,
               address: Address,
               sender: Sender<F>,
               listener: Box<dyn IsoTpEventListener>
    ) -> Self {
        Self {
            channel,
            address,
            sender,
            context: Default::default(),
            state: Default::default(),
            listener: Arc::new(Mutex::new(listener)),
        }
    }

    pub fn write(&mut self, functional: bool, data: Vec<u8>) -> Result<(), IsoTpError> {
        log::debug!("ISO-TP(CAN sync) - Sending: {:?}", data);
        let frames = CanIsoTpFrame::from_data(data)?;
        let frame_len = frames.len();

        let can_id = if functional { self.address.fid } else { self.address.tx_id };
        for (index, frame) in frames.into_iter().enumerate() {
            self.write_waiting(index)?;
            let mut frame = F::from_iso_tp(Id::from_bits(can_id, false), frame, None)
                .ok_or(IsoTpError::ConvertError {
                    src: "iso-tp frame",
                    target: "can-frame",
                })?;
            frame.set_channel(self.channel.clone());

            self.state_append(IsoTpState::Sending);
            if 0 == index && 1 < frame_len {
                self.state_append(IsoTpState::WaitFlowCtrl);
            }
            self.sender.send(frame)
                .map_err(|e| {
                    log::warn!("ISO-TP(CAN sync) - transmit failed: {:?}", e);
                    IsoTpError::DeviceError
                })?;
        }

        Ok(())
    }

    #[inline]
    pub(crate) fn on_single_frame(&mut self, data: Vec<u8>) {
        self.iso_tp_event(IsoTpEvent::DataReceived(data));
    }

    #[inline]
    pub(crate) fn on_first_frame(&mut self, length: u32, data: Vec<u8>) {
        self.context.update_consecutive(length, data);

        let iso_tp_frame = CanIsoTpFrame::default_flow_ctrl_frame();

        match F::from_iso_tp(
            Id::from_bits(self.address.tx_id, false),
            iso_tp_frame,
            None
        ) {
            Some(mut frame) => {
                frame.set_channel(self.channel.clone());

                self.state_append(IsoTpState::Sending);
                match self.sender.send(frame) {
                    Ok(_) => {
                        self.iso_tp_event(IsoTpEvent::FirstFrameReceived);
                    },
                    Err(e) => {
                        log::warn!("ISO-TP - transmit failed: {:?}", e);
                        self.state_append(IsoTpState::Error);

                        self.iso_tp_event(IsoTpEvent::ErrorOccurred(IsoTpError::DeviceError));
                    },
                }
            },
            None => log::error!("ISO-TP: convert `iso-tp frame` to `can-frame` error"),
        }
    }

    #[inline]
    pub(crate) fn on_consecutive_frame(&mut self, sequence: u8, data: Vec<u8>) {
        match self.context.append_consecutive(sequence, data) {
            Ok(event) => {
                match event {
                    IsoTpEvent::DataReceived(_) => {
                        self.context.reset();
                    },
                    _ => {},
                }
                self.iso_tp_event(event);
            },
            Err(e) => {
                self.state_append(IsoTpState::Error);
                self.iso_tp_event(IsoTpEvent::ErrorOccurred(e));
            }
        }
    }

    #[inline]
    pub(crate) fn on_flow_ctrl_frame(&mut self, ctx: FlowControlContext) {
        match ctx.state() {
            FlowControlState::Continues => {
                self.state_remove(IsoTpState::WaitBusy | IsoTpState::WaitFlowCtrl);
            },
            FlowControlState::Wait => {
                self.state_append(IsoTpState::WaitBusy);
                self.iso_tp_event(IsoTpEvent::Wait);
                return;
            }
            FlowControlState::Overload => {
                self.state_append(IsoTpState::Error);
                self.iso_tp_event(IsoTpEvent::ErrorOccurred(IsoTpError::OverloadFlow));
                return;
            }
        }

        self.context.update_flow_ctrl(ctx);
    }

    fn iso_tp_event(&self, event: IsoTpEvent) {
        match self.listener.lock() {
            Ok(mut listener) => {
                // println!("ISO-TP(CAN asyn): Sending iso-tp event: {:?}", event);
                log::trace!("ISO-TP(CAN asyn): Sending iso-tp event: {:?}", event);
                listener.on_iso_tp_event(event);
            },
            Err(_) => log::warn!("ISO-TP(CAN async): Sending event failed"),
        }
    }

    fn write_waiting(&mut self, index: usize) -> Result<(), IsoTpError> {
        if let Some(ctx) = &self.context.flow_ctrl {
            if ctx.block_size != 0 &&
                0 == ctx.block_size as usize % (index + 1) {
                self.state_append(IsoTpState::WaitFlowCtrl);
            }
            sleep(Duration::from_micros(ctx.st_min as u64));
        }

        loop {
            if self.state_contains(IsoTpState::Error) {
                return Err(IsoTpError::DeviceError);
            }

            if self.state_contains(IsoTpState::Sending | IsoTpState::WaitBusy | IsoTpState::WaitFlowCtrl) {
                sleep(Duration::from_micros(10));
            }
            else {
                break;
            }
        }

        Ok(())
    }

    #[inline]
    fn state_contains(&self, flags: IsoTpState) -> bool {
        match self.state.lock() {
            Ok(v) => *v & flags != IsoTpState::Idle,
            Err(_) => {
                log::warn!("ISO-TP: state mutex is poisoned");
                false
            },
        }
    }

    #[inline]
    fn state_append(&self, flags: IsoTpState) {
        match self.state.lock() {
            Ok(mut v) => {
                if flags.contains(IsoTpState::Error) {
                    *v = IsoTpState::Error;
                }
                else {
                    *v |= flags;
                }
            }
            Err(_) => log::warn!("ISO-TP: state mutex is poisoned"),
        }
    }

    #[inline]
    fn state_remove(&self, flags: IsoTpState) {
        match self.state.lock() {
            Ok(mut v) => v.remove(flags),
            Err(_) => log::warn!("ISO-TP: state mutex is poisoned"),
        }
    }
}
