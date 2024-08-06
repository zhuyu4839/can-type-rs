use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::sync::{Arc, RwLock};
use isotp_rs::{FlowControlContext, IsoTpEvent, IsoTpEventListener, IsoTpState};
use isotp_rs::constant::CONSECUTIVE_SEQUENCE_START;
use isotp_rs::error::Error as IsoTpError;
use crate::isotp::Address;

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

pub(crate) struct InnerContext<Channel> {
    pub(crate) address: Address,
    pub(crate) st_min: u32,    // μs
    pub(crate) data: DataContext,
    pub(crate) state: IsoTpState,
    pub(crate) listeners: Vec<Box<dyn IsoTpEventListener<Channel = Channel>>>,
}

impl<Channel> InnerContext<Channel> {
    pub(crate) fn new(address: Address) -> Self {
        Self {
            address,
            st_min: 0,
            state: Default::default(),
            data: Default::default(),
            listeners: Default::default(),
        }
    }
}

#[derive(Clone)]
pub struct IsoTpContext<Channel: Clone + Hash + Eq> {
    pub(crate) inner: Arc<RwLock<HashMap<Channel, InnerContext<Channel>>>>,
}

impl<Channel: Clone + Hash + Eq> Default for IsoTpContext<Channel> {
    fn default() -> Self {
        Self {
            inner: Default::default(),
            // rw_lock: Default::default(),
        }
    }
}

impl<Channel: Display + Clone + Hash + Eq> IsoTpContext<Channel> {
    #[inline]
    pub(crate) fn add(&mut self,
                      channel: &Channel,
                      address: Address,
    ) -> Result<(), IsoTpError> {
        match self.inner.write() {
            Ok(mut v) => {
                v.insert(channel.clone(), InnerContext::new(address));
                Ok(())
            },
            Err(_) => Err(IsoTpError::ContextError(format!("adding context for channel {}", channel))),
        }

    }

    #[inline]
    pub(crate) fn remove(&mut self, channel: &Channel) -> Result<bool, IsoTpError> {
        match self.inner.write() {
            Ok(mut v) => Ok(v.remove(channel).is_some()),
            Err(_) => Err(IsoTpError::ContextError(format!("removing context for channel {}", channel))),
        }
    }

    /// reset the p2/p2*/st_min/data-context to default.
    #[inline]
    pub(crate) fn reset(&mut self, channel: &Channel) -> Result<bool, IsoTpError> {
        match self.inner.write() {
            Ok(mut v) => match v.get_mut(channel) {
                Some(v) => {
                    v.st_min = 0;

                    v.state = IsoTpState::Idle;
                    v.data.length = Default::default();
                    v.data.sequence = Default::default();
                    v.data.buffer.clear();

                    Ok(true)
                },
                None => Ok(false),
            },
            Err(_) => Err(IsoTpError::ContextError(format!("restoring context for channel {}", channel))),
        }
    }

    #[inline]
    pub(crate) fn state(&self, channel: &Channel) -> Result<Option<IsoTpState>, IsoTpError> {
        match self.inner.read() {
            Ok(v) => Ok(
                v.get(channel).and_then(|v| Some(v.state)),
            ),
            Err(_) => Err(IsoTpError::ContextError(format!("getting state for channel {}", channel))),
        }
    }

    #[inline]
    pub(crate) fn state_remove(&mut self,
                               channel: &Channel,
                               flags: IsoTpState
    ) -> Result<Option<()>, IsoTpError> {
        match self.inner.write() {
            Ok(mut v) => Ok(
                v.get_mut(channel)
                    .and_then(|v| {
                        Self::inner_state_remove(v, flags);
                        Some(())
                    })
            ),
            Err(_) => Err(IsoTpError::ContextError(format!("removing state for channel {}", channel))),
        }
    }

    pub(crate) fn state_add(&mut self,
                            channel: &Channel,
                            flags: IsoTpState
    ) -> Result<Option<()>, IsoTpError> {
        match self.inner.write() {
            Ok(mut v) => Ok(
                v.get_mut(channel)
                    .and_then(|v| {
                        Self::inner_state_add(channel, v, flags);
                        Some(())
                    })
            ),
            Err(_) => Err(IsoTpError::ContextError(format!("adding state for channel {}", channel))),
        }
    }

    // #[inline]
    // pub(crate) fn state_restore(&mut self,
    //                             channel: &Channel,
    //                             flags: IsoTpState
    // ) -> Result<Option<()>, IsoTpError> {
    //     match self.inner.write() {
    //         Ok(mut v) => Ok(
    //             v.get_mut(channel)
    //                 .and_then(|v| {
    //                     Self::inner_state_restore(v, flags);
    //                     Some(())
    //                 })
    //         ),
    //         Err(_) => Err(IsoTpError::ContextError(format!("restoring state for channel {}", channel))),
    //     }
    // }

    #[inline]
    pub(crate) fn address(&self, channel: &Channel) -> Result<Option<Address>, IsoTpError> {
        match self.inner.read() {
            Ok(v) => Ok(
                v.get(channel)
                    .and_then(|v| Some(v.address))
            ),
            Err(_) => Err(IsoTpError::ContextError(format!("reading address for channel {}", channel))),
        }
    }

    #[inline]
    pub(crate) fn register_listener(&mut self,
                                    channel: &Channel,
                                    listener: Box<dyn IsoTpEventListener<Channel = Channel>>
    ) -> Result<Option<()>, IsoTpError> {
        match self.inner.write() {
            Ok(mut v) => Ok(
                v.get_mut(channel)
                    .and_then(|v| {
                        v.listeners.push(listener);
                        Some(())
                    })
            ),
            Err(_) => Err(IsoTpError::ContextError(format!("adding listener for channel {}", channel))),
        }
    }

    #[inline]
    pub(crate) fn unregister_listeners(&mut self, channel: &Channel) -> Result<Option<()>, IsoTpError> {
        match self.inner.write() {
            Ok(mut v) => Ok(
                v.get_mut(channel)
                    .and_then(|v| {
                        v.listeners.clear();
                        Some(())
                    })
            ),
            Err(_) => Err(IsoTpError::ContextError(format!("clearing listener for channel {}", channel))),
        }
    }

    #[inline]
    pub(crate) fn on_single_frame(&mut self,
                                  channel: &Channel,
                                  data: Vec<u8>
    ) -> Result<bool, IsoTpError> {
        match self.inner.write() {
            Ok(mut v) => match v.get_mut(channel) {
                Some(v) => {
                    Self::frame_state_check(channel, v, IsoTpState::WaitSingle)?;

                    Self::on_iso_tp_event(
                        channel,
                        IsoTpEvent::DataReceived(data.as_slice()),
                        &v.listeners,
                    );

                    Self::inner_state_remove(
                        v,
                        IsoTpState::Sending | IsoTpState::WaitSingle | IsoTpState::WaitFirst
                    );

                    Ok(true)
                },
                None => Ok(false),
            },
            Err(_) => Err(IsoTpError::ContextError(format!("writing data for channel {}", channel))),
        }
    }

    #[inline]
    pub(crate) fn on_first_frame(&mut self,
                                 channel: &Channel,
                                 length: u32,
                                 mut data: Vec<u8>
    ) -> Result<bool, IsoTpError> {
        match self.inner.write() {
            Ok(mut v) => match v.get_mut(channel) {
                Some(v) => {
                    Self::frame_state_check(channel, v, IsoTpState::WaitFirst)?;
                    v.data.length = Some(length);
                    v.data.buffer.clear();
                    v.data.buffer.append(&mut data);
                    Self::inner_state_remove(
                        v,
                        IsoTpState::Sending | IsoTpState::WaitSingle | IsoTpState::WaitFirst
                    );

                    Ok(true)
                },
                None => Ok(false),
            },
            Err(_) => Err(IsoTpError::ContextError(format!("writing data for channel {}", channel))),
        }
    }

    pub(crate) fn on_consecutive_frame(&mut self,
                                       channel: &Channel,
                                       sequence: u8,
                                       mut data: Vec<u8>,
    ) -> Result<bool, IsoTpError> {
        match self.inner.write() {
            Ok(mut v) => match v.get_mut(channel) {
                Some(v) => {
                    Self::frame_state_check(channel, v, IsoTpState::WaitData)?;

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
                                Self::inner_state_restore(v, IsoTpState::Idle);
                                return Err(
                                    IsoTpError::InvalidDataLength {
                                        actual: len as usize, expect: buff_len
                                    });
                            }
                            else if buff_len == len as usize {
                                let data = v.data.buffer.as_slice();
                                Self::on_iso_tp_event(
                                    channel,
                                    IsoTpEvent::DataReceived(data),
                                    &v.listeners,
                                );

                                Self::inner_state_restore(v, IsoTpState::Idle);
                            }
                            else {
                                v.state |= IsoTpState::WaitData;
                            }

                            Ok(true)
                        },
                        None => Err(IsoTpError::MixFramesError),
                    }
                },
                None => Ok(false),
            },
            Err(_) => Err(IsoTpError::ContextError(format!("writing data for channel {}", channel))),
        }
    }

    pub(crate) fn on_flow_ctrl_frame(&mut self,
                                     channel: &Channel,
                                     ctx: FlowControlContext
    ) -> Result<bool, IsoTpError> {
        match self.inner.write() {
            Ok(mut v) => match v.get_mut(channel) {
                Some(v) => {
                    Self::frame_state_check(channel, v, IsoTpState::WaitFlowCtrl)?;

                    v.st_min = ctx.st_min_us();
                    Ok(true)
                },
                None => Ok(false),
            },
            Err(_) => Err(IsoTpError::ContextError(format!("writing data for channel {}", channel))),
        }
    }

    fn on_iso_tp_event(channel: &Channel,
                       event: IsoTpEvent,
                       listeners: &Vec<Box<dyn IsoTpEventListener<Channel = Channel>>>
    ) {
        listeners.iter()
            .for_each(|o| o.on_iso_tp_event(channel.clone(), event.clone()));
    }

    fn inner_state_add(channel: &Channel,
                       v: &mut InnerContext<Channel>,
                       flags: IsoTpState
    ) {
        if flags.contains(IsoTpState::WaitData) {
            if flags.contains(IsoTpState::Sending) {
                v.state = IsoTpState::Sending | IsoTpState::WaitData;
            }
            else {
                v.state = IsoTpState::WaitData;
            }
        }
        else if flags.contains(IsoTpState::Error) {
            v.state = IsoTpState::Error;
            Self::on_iso_tp_event(channel, IsoTpEvent::ErrorOccurred(IsoTpError::DeviceError), &v.listeners);
        }
        else {
            v.state |= flags;
        }
    }

    #[inline]
    fn inner_state_remove(v: &mut InnerContext<Channel>, flags: IsoTpState) {
        v.state &= !flags;
    }

    #[inline]
    fn inner_state_restore(v: &mut InnerContext<Channel>, flags: IsoTpState) {
        v.state = flags;
    }

    #[inline]
    fn frame_state_check(
        channel: &Channel,
        v: &mut InnerContext<Channel>,
        expected: IsoTpState,
    ) -> Result<(), IsoTpError>{
        if !v.state.contains(expected) {
            let err = IsoTpError::StateError { expected, found: v.state };

            Self::on_iso_tp_event(
                channel,
                IsoTpEvent::ErrorOccurred(err.clone()),
                &v.listeners
            );

            return Err(err);
        }

        Ok(())
    }
}
