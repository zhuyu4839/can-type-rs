use std::fmt::Debug;
use std::sync::{Arc, RwLock};
use isotp_rs::{FlowControlContext, IsoTpEvent, IsoTpEventListener, IsoTpFrame, IsoTpState};
use isotp_rs::constant::CONSECUTIVE_SEQUENCE_START;
use isotp_rs::error::Error as IsoTpError;
use crate::identifier::Id;
use crate::device::CanListener;
use crate::frame::Frame;
use crate::isotp::{Address, CanIsoTpFrame};

/// Consecutive frame data context.
#[derive(Debug, Clone)]
pub(crate) struct DataContext {
    pub(crate) sequence: Option<u8>,
    pub(crate) length: Option<u32>,
    pub(crate) buffer: Vec<u8>,
}

impl Default for DataContext {
    fn default() -> Self {
        Self {
            sequence: Default::default(),
            length: Default::default(),
            buffer: Default::default(),
        }
    }
}

pub(crate) struct InnerContext {
    pub(crate) address: Address,
    pub(crate) st_min: u32,    // μs
    pub(crate) data: DataContext,
    pub(crate) state: IsoTpState,
    pub(crate) listener: Option<Box<dyn IsoTpEventListener>>,
    pub(crate) write_waiting: bool,
}

impl InnerContext {
    pub(crate) fn new(address: Address) -> Self {
        Self {
            address,
            st_min: 0,
            state: Default::default(),
            data: Default::default(),
            listener: Default::default(),
            write_waiting: true,
        }
    }
}

#[derive(Clone)]
pub struct IsoTpContext<Channel: Clone> {
    pub(crate) channel: Channel,
    pub(crate) inner: Arc<RwLock<InnerContext>>,
}

impl<Channel: Clone> IsoTpContext<Channel> {
    #[inline]
    pub(crate) fn new(channel: Channel, address: Address) -> Self {
        Self {
            channel,
            inner: Arc::new(RwLock::new(InnerContext::new(address)))
        }
    }

    /// reset the p2/p2*/st_min/data-context to default.
    #[inline]
    pub fn reset(&mut self) -> Result<(), IsoTpError> {
        match self.inner.write() {
            Ok(mut v) => {
                v.st_min = 0;

                v.state = IsoTpState::Idle;
                v.data.length = Default::default();
                v.data.sequence = Default::default();
                v.data.buffer.clear();

                Ok(())
            },
            Err(_) => Err(IsoTpError::ContextError("restoring context".to_string())),
        }
    }

    #[inline]
    pub fn state(&self) -> Result<IsoTpState, IsoTpError> {
        match self.inner.read() {
            Ok(v) => Ok(v.state),
            Err(_) => Err(IsoTpError::ContextError("getting state".to_string())),
        }
    }

    #[inline]
    pub fn state_remove(&mut self,
                        flags: IsoTpState
    ) -> Result<(), IsoTpError> {
        match self.inner.write() {
            Ok(mut v) => {
                Self::inner_state_remove(&mut v, flags);
                Ok(())
            },
            Err(_) => Err(IsoTpError::ContextError("removing state".to_string())),
        }
    }

    pub fn state_add(&mut self,
                     flags: IsoTpState
    ) -> Result<(), IsoTpError> {
        match self.inner.write() {
            Ok(mut v) => {
                Self::inner_state_add(&mut v, flags);
                Ok(())
            },
            Err(_) => Err(IsoTpError::ContextError("adding state".to_string())),
        }
    }

    pub fn clear_listener_buffer(&mut self) -> Result<(), IsoTpError> {
        match self.inner.write() {
            Ok(mut v) => {
                if let Some(listener) = &mut v.listener {
                    listener.clear_buffer();
                }
                Ok(())
            },
            Err(_) => Err(IsoTpError::ContextError("adding state".to_string())),
        }
    }

    pub fn write_waiting(&self) -> Result<bool, IsoTpError> {
        match self.inner.read() {
            Ok(v) => Ok(v.write_waiting),
            Err(_) => Err(IsoTpError::ContextError("reading write waiting flag".to_string())),
        }
    }

    pub fn set_write_waiting(&self, flag: bool) -> Result<(), IsoTpError> {
        match self.inner.write() {
            Ok(mut v) => {
                v.write_waiting = flag;
                Ok(())
            },
            Err(_) => Err(IsoTpError::ContextError("writing write waiting flag".to_string())),
        }
    }

    #[inline]
    pub(crate) fn address(&self) -> Result<Address, IsoTpError> {
        match self.inner.read() {
            Ok(v) => Ok(v.address),
            Err(_) => Err(IsoTpError::ContextError("reading address".to_string())),
        }
    }

    #[inline]
    pub fn register_listener(&mut self,
                             listener: Box<dyn IsoTpEventListener>
    ) -> Result<(), IsoTpError> {
        match self.inner.write() {
            Ok(mut v) => {
                v.listener = Some(listener);
                Ok(())
            },
            Err(_) => Err(IsoTpError::ContextError("adding listener".to_string())),
        }
    }

    #[inline]
    pub fn unregister_listeners(&mut self) -> Result<(), IsoTpError> {
        match self.inner.write() {
            Ok(mut v) => {
                v.listener = None;
                Ok(())
            },
            Err(_) => Err(IsoTpError::ContextError("clearing listener".to_string())),
        }
    }

    #[inline]
    pub(crate) fn on_single_frame(&mut self, data: Vec<u8>) -> Result<(), IsoTpError> {
        match self.inner.write() {
            Ok(mut v) => {
                Self::frame_state_check(&mut v, IsoTpState::WaitSingle)?;
                v.st_min = 0;

                Self::on_iso_tp_event(
                    IsoTpEvent::DataReceived(data),
                    &mut v.listener,
                );

                Self::inner_state_remove(
                    &mut v,
                    IsoTpState::Sending | IsoTpState::WaitSingle | IsoTpState::WaitFirst
                );

                Ok(())
            },
            Err(_) => Err(IsoTpError::ContextError("writing data".to_string())),
        }
    }

    /// first frame received
    #[inline]
    pub(crate) fn on_first_frame(&mut self,
                                 length: u32,
                                 mut data: Vec<u8>
    ) -> Result<(), IsoTpError> {
        match self.inner.write() {
            Ok(mut v) => {
                Self::on_iso_tp_event(
                    IsoTpEvent::Wait,
                    &mut v.listener,
                );

                Self::frame_state_check(&mut v, IsoTpState::WaitFirst)?;
                v.st_min = 0;
                v.data.length = Some(length);
                v.data.buffer.clear();
                v.data.buffer.append(&mut data);
                Self::inner_state_remove(
                    &mut v,
                    IsoTpState::Sending | IsoTpState::WaitSingle | IsoTpState::WaitFirst
                );

                Ok(())
            },
            Err(_) => Err(IsoTpError::ContextError("writing data".to_string())),
        }
    }

    /// consecutive frame received
    pub(crate) fn on_consecutive_frame(&mut self,
                                       sequence: u8,
                                       mut data: Vec<u8>,
    ) -> Result<bool, IsoTpError> {
        match self.inner.write() {
            Ok(mut v) => {
                    Self::frame_state_check(&mut v, IsoTpState::WaitData)?;

                    match v.data.length {
                        Some(len) => {
                            match v.data.sequence {
                                Some(mut v) => {
                                    match v {
                                        ..=0x0F => v = 0,
                                        _ => v += 1,
                                    }

                                    if sequence != v {
                                        return Err(IsoTpError::InvalidSequence {
                                            actual: sequence,
                                            expect: v,
                                        });
                                    }
                                },
                                None => {
                                    if sequence != CONSECUTIVE_SEQUENCE_START {
                                        return Err(IsoTpError::InvalidSequence {
                                            actual: sequence,
                                            expect: CONSECUTIVE_SEQUENCE_START,
                                        });
                                    }
                                }
                            }
                            v.data.sequence = Some(sequence);

                            v.data.buffer.append(&mut data);
                            let buff_len = v.data.buffer.len();

                            if buff_len > len as usize {
                                v.state = IsoTpState::Idle;

                                return Err(IsoTpError::InvalidDataLength {
                                    actual: len as usize,
                                    expect: buff_len
                                });
                            }
                            else if buff_len == len as usize {
                                // let data = v.data.buffer.as_slice();

                                Self::on_iso_tp_event(
                                    IsoTpEvent::DataReceived(v.data.buffer.clone()),
                                    &mut v.listener,
                                );

                                v.state = IsoTpState::Idle;
                            }
                            else {
                                Self::on_iso_tp_event(
                                    IsoTpEvent::Wait,
                                    &mut v.listener,
                                );

                                v.state |= IsoTpState::WaitData;
                            }

                            Ok(true)
                        },
                        None => Err(IsoTpError::MixFramesError),
                    }
            },
            Err(_) => Err(IsoTpError::ContextError("writing data".to_string())),
        }
    }

    /// flow control frame received
    pub(crate) fn on_flow_ctrl_frame(&mut self,
                                     ctx: FlowControlContext
    ) -> Result<(), IsoTpError> {
        match self.inner.write() {
            Ok(mut v) => {
                Self::frame_state_check(&mut v, IsoTpState::WaitFlowCtrl)?;
                v.st_min = ctx.st_min_us();
                Ok(())
            },
            Err(_) => Err(IsoTpError::ContextError("writing data".to_string())),
        }
    }

    #[inline]
    pub(crate) fn st_min(&self) -> Result<u32, IsoTpError> {
        match self.inner.read() {
            Ok(v) => Ok(v.st_min),
            Err(_) => Err(IsoTpError::ContextError("reading st_min".to_string())),
        }
    }

    #[inline]
    fn on_iso_tp_event(event: IsoTpEvent,
                       listener: &mut Option<Box<dyn IsoTpEventListener>>
    ) {
        if let Some(listener) = listener {
            listener.on_iso_tp_event(event);
        }
    }

    fn inner_state_add(ctx: &mut InnerContext,
                       flags: IsoTpState
    ) {
        if flags.contains(IsoTpState::WaitData) {
            if flags.contains(IsoTpState::Sending) {
                ctx.state = IsoTpState::Sending | IsoTpState::WaitData;
            }
            else {
                ctx.state = IsoTpState::WaitData;
            }
        }
        else if flags.contains(IsoTpState::Error) {
            ctx.state = IsoTpState::Error;

            Self::on_iso_tp_event(
                IsoTpEvent::ErrorOccurred(IsoTpError::DeviceError),
                &mut ctx.listener
            );
        }
        else {
            ctx.state |= flags;
        }
    }

    #[inline]
    fn inner_state_remove(ctx: &mut InnerContext, flags: IsoTpState) {
        ctx.state &= !flags;
    }

    #[inline]
    fn frame_state_check(
        v: &mut InnerContext,
        expected: IsoTpState,
    ) -> Result<(), IsoTpError>{
        if !v.state.contains(expected) {
            let err = IsoTpError::StateError { expected, found: v.state };

            Self::on_iso_tp_event(
                IsoTpEvent::ErrorOccurred(err.clone()),
                &mut v.listener
            );

            return Err(err);
        }

        Ok(())
    }
}

unsafe impl<Channel: Clone + Eq> Send for IsoTpContext<Channel> {}

impl<F: Frame, Channel: Clone + Eq> CanListener<F, Channel> for IsoTpContext<Channel> {
    fn on_frame_transmitted(&mut self, id: Id, channel: Channel) {
        if channel != self.channel {
            return;
        }

        match self.address() {
            Ok(address) => {
                if address.tx_id == id.as_raw() ||
                    address.fid == id.as_raw() {
                    if let Err(e) = self.state_remove(IsoTpState::Sending) {
                        log::warn!("{}", e);
                    }
                }
            },
            Err(e) => log::warn!("{}", e),
        }
    }

    fn on_frame_received(&mut self, frames: &Vec<F>, channel: Channel) {
        if channel != self.channel {
            return;
        }

        match self.address() {
            Ok(address) => {
                for frame in frames {
                    if frame.id(false).as_raw() == address.rx_id {
                        match CanIsoTpFrame::decode(frame.data()) {
                            Ok(frame) => {
                                match frame {
                                    CanIsoTpFrame::SingleFrame { data } => {
                                        if let Err(e) = self.on_single_frame(data) {
                                            log::warn!("{}", e);
                                        }
                                    },
                                    CanIsoTpFrame::FirstFrame { length, data } => {
                                        if let Err(e) = self.on_first_frame(length, data) {
                                            log::warn!("{}", e);
                                        }
                                    },
                                    CanIsoTpFrame::ConsecutiveFrame { sequence, data } => {
                                        if let Err(e) = self.on_consecutive_frame(sequence, data) {
                                            log::warn!("{}", e);
                                        }
                                    },
                                    CanIsoTpFrame::FlowControlFrame(ctx) => {
                                        if let Err(e) = self.on_flow_ctrl_frame(ctx) {
                                            log::warn!("{}", e);
                                        }
                                    },
                                }
                            },
                            Err(e) => {
                                log::warn!("ISO-TP(CAN sync) - data convert to frame failed: {}", e);
                                if let Err(e) = self.state_add(IsoTpState::Error) {
                                    log::warn!("{}", e);
                                }
                                break;
                            },
                        }
                    }
                }
            },
            Err(e) => log::warn!("{}", e),
        }
    }
}
