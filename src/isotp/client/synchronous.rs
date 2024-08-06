use std::fmt::Display;
use std::hash::Hash;
use isotp_rs::error::Error as IsoTpError;
use isotp_rs::{FlowControlContext, FrameType, IsoTpEventListener, IsoTpFrame, IsoTpState};
use crate::device::{SyncCanDevice, CanListener};
use crate::frame::Frame as CanFrame;
use crate::identifier::Id;
use crate::isotp::{Address, CanIsoTpFrame};
use crate::isotp::client::context::IsoTpContext;

#[derive(Clone)]
pub struct SyncCanIsoTp<D, Device, Channel, Frame>
where
    D: SyncCanDevice<Device= Device, Channel= Channel, Frame= Frame>,
    Channel: Display + Clone + Hash + Eq,
    Frame: CanFrame, {
    device: D,
    context: IsoTpContext<Channel>,
}

unsafe impl<D, Device, Channel, Frame> Send for SyncCanIsoTp<D, Device, Channel, Frame>
where
    D: SyncCanDevice<Device= Device, Channel= Channel, Frame= Frame>,
    Channel: Display + Clone + Hash + Eq,
    Frame: CanFrame {}

impl<D, Device, Channel, Frame> SyncCanIsoTp<D, Device, Channel, Frame>
where
    D: SyncCanDevice<Device= Device, Channel= Channel, Frame= Frame>,
    Channel: Display + Clone + Hash + Eq,
    Frame: CanFrame, {

    #[inline]
    pub fn new(device: D) -> Self {
        Self {
            device,
            context: Default::default(),
        }
    }

    #[inline]
    pub fn add_channel(&mut self,
                       channel: Channel,
                       address: Address,
    ) -> Result<(), IsoTpError> {
        self.context.add(&channel, address)
    }

    #[inline]
    pub fn remove_channel(&mut self, channel: Channel) -> Result<bool, IsoTpError> {
        self.context.remove(&channel)
    }

    pub fn reset_channel(&mut self, channel: Channel) -> Result<bool, IsoTpError> {
        self.context.reset(&channel)
    }

    #[inline]
    pub fn state_add(&mut self, channel: Channel, flags: IsoTpState) -> Result<bool, IsoTpError> {
        let ret = self.context.state_add(&channel, flags)?;
        Ok(ret.is_some())
    }

    #[inline]
    pub fn register_listener(&mut self,
                             channel: Channel,
                             listener: Box<dyn IsoTpEventListener<Channel = Channel>>
    ) -> Result<bool, IsoTpError> {
        let ret = self.context.register_listener(&channel, listener)?;
        Ok(ret.is_some())
    }

    #[inline]
    pub fn unregister_listeners(&mut self, channel: Channel) -> Result<bool, IsoTpError> {
        let ret = self.context.unregister_listeners(&channel)?;
        Ok(ret.is_some())
    }

    pub fn write(&mut self,
                 channel: Channel,
                 functional: bool,
                 data: Vec<u8>,
                 // p2_server: &P2Context,
    ) -> Result<(), IsoTpError> {
        if let Some(address) = self.context.address(&channel)? {
            // self.write_p2_star(&channel, p2_server.p2_star)?;

            log::debug!("ISO-TP(CAN sync) - Sending: {:?}", data);

            let frames = CanIsoTpFrame::from_data(data, vec![])?;
            for frame in frames {
                let frame_type = match &frame {
                    CanIsoTpFrame::SingleFrame { .. } => FrameType::Single,
                    CanIsoTpFrame::FirstFrame { .. } => FrameType::First,
                    CanIsoTpFrame::ConsecutiveFrame { .. } => FrameType::Consecutive,
                    CanIsoTpFrame::FlowControlFrame(_) => FrameType::FlowControl,
                };

                let can_id = if functional { address.fid } else { address.tx_id };
                let frame = Frame::from_iso_tp(Id::from_bits(can_id, false), frame, None)
                    .ok_or(IsoTpError::ConvertError {
                        src: "iso-tp frame",
                        target: "can-frame",
                    })?;

                if let Err(e) = self.device.sender()
                    .send(frame) {
                    log::warn!("ISO-TP(CAN sync) - transmit failed: {:?}", e);
                    self.context.state_add(&channel, IsoTpState::Error)?;

                    break;
                }

                let _ = self.write_state(&channel, frame_type)?;

                // self.write_p2(&channel, &p2_server)?;
            }
        }

        Ok(())
    }

    pub fn close(&mut self) {
        // println!("ISO-TP close");
        log::info!("ISO-TP(CAN sync) - closing device");
        self.device.close();
    }

    // #[allow(unused_assignments)]
    // fn write_p2_star(&mut self, channel: &Channel, p2_star: u32) -> Result<(), IsoTpError> {
    //     let mut state = IsoTpState::Error;
    //
    //     while let Some(v) = self.context.state(channel)? {
    //         if v.contains(IsoTpState::ResponsePending) {
    //             log::info!("ISO-TP(CAN sync) - response pending, timeout reset to P2*({}ms)", p2_star);
    //             std::thread::sleep(std::time::Duration::from_millis(p2_star as u64));
    //         }
    //         else {
    //             state = v;
    //             break;
    //         }
    //     }
    //
    //     if state.contains(IsoTpState::Error) ||
    //         state.contains(IsoTpState::Sending) {
    //         self.context.state_add(channel, IsoTpState::Error)?;
    //         return Err(IsoTpError::DeviceError);
    //     }
    //
    //     Ok(())
    // }

    fn write_state(&mut self, channel: &Channel, frame_type: FrameType) -> Result<bool, IsoTpError> {
        let ret = match frame_type {
            FrameType::Single => {
                self.context.state_add(
                    channel,
                    IsoTpState::Sending | IsoTpState::WaitSingle | IsoTpState::WaitFirst
                )
            },
            FrameType::First => {
                self.context.state_add(
                    channel,
                    IsoTpState::Sending | IsoTpState::WaitFlowCtrl
                )
            },
            FrameType::Consecutive => { // 发送最后一帧
                let last = false;
                if last {
                    self.context.state_add(
                        channel,
                        IsoTpState::Sending | IsoTpState::WaitSingle | IsoTpState::WaitFirst
                    )
                }
                else {
                    self.context.state_add(
                        channel,
                        IsoTpState::Sending | IsoTpState::WaitData
                    )
                }
            },
            FrameType::FlowControl => self.context.state_add(
                channel,
                IsoTpState::Sending | IsoTpState::Idle
            ),
        }?;

        Ok(ret.is_some())
    }

    // fn write_p2(&self, channel: &Channel, p2_server: &P2Context) -> Result<(), IsoTpError> {
    //     let timeout = std::time::Duration::from_millis(
    //         (p2_server.p2 + p2_server.p2_offset) as u64
    //     );
    //     let start = std::time::Instant::now();
    //
    //     while let Some(v) = self.context.state(channel)? {
    //         if v == IsoTpState::Idle {
    //             break;
    //         }
    //         else {
    //             std::thread::sleep(std::time::Duration::from_millis(1));
    //         }
    //
    //         let elapsed_time = std::time::Instant::now().duration_since(start);
    //
    //         if elapsed_time >= timeout {
    //             return Err(IsoTpError::Timeout { value: p2_server.p2 as u64, unit: "ms" });
    //         }
    //     }
    //
    //     Ok(())
    // }

    fn on_single_frame(&mut self, channel: &Channel, data: Vec<u8>) {
        if let Err(e) = self.context.on_single_frame(channel, data) {
            log::warn!("{}", e);
        }
    }

    fn on_first_frame(&mut self, channel: &Channel, length: u32, data: Vec<u8>) {
        if let Err(e) = self.context.on_first_frame(channel, length, data) {
            log::warn!("{}", e);
        }
    }

    fn on_consecutive_frame(&mut self, channel: &Channel, sequence: u8, data: Vec<u8>) {
        if let Err(e) = self.context.on_consecutive_frame(channel, sequence, data) {
            log::warn!("{}", e);
        }
    }

    fn on_flow_ctrl_frame(&mut self, channel: &Channel, ctx: FlowControlContext) {
        if let Err(e) = self.context.on_flow_ctrl_frame(channel, ctx) {
            log::warn!("{}", e);
        }
    }
}

impl<D, Device, Channel, Frame> CanListener for SyncCanIsoTp<D, Device, Channel, Frame>
where
    D: SyncCanDevice<Device= Device, Channel= Channel, Frame= Frame>,
    Channel: Display + Clone + Hash + Eq,
    Frame: CanFrame, {
    type Frame = Frame;
    type Channel = Channel;

    fn on_frame_transmitted(&mut self, id: Id, channel: Self::Channel) {
        match self.context.address(&channel) {
            Ok(address) => if let Some(address) = address {
                if address.tx_id == id.as_raw() ||
                    address.fid == id.as_raw() {
                    if let Err(e) = self.context.state_remove(&channel, IsoTpState::Sending) {
                        log::warn!("{}", e);
                    }
                }
            },
            Err(e) => log::warn!("{}", e),
        }
    }

    fn on_frame_received(&mut self, frames: &Vec<Self::Frame>, channel: Self::Channel) {
        match self.context.address(&channel) {
            Ok(address) => if let Some(address) = address {
                for frame in frames {
                    if frame.id(false).as_raw() == address.rx_id {
                        match CanIsoTpFrame::decode(frame.data()) {
                            Ok(frame) => {
                                match frame {
                                    CanIsoTpFrame::SingleFrame { data } =>
                                        self.on_single_frame(&channel, data),
                                    CanIsoTpFrame::FirstFrame { length, data } =>
                                        self.on_first_frame(&channel, length, data),
                                    CanIsoTpFrame::ConsecutiveFrame { sequence, data } =>
                                        self.on_consecutive_frame(&channel, sequence, data),
                                    CanIsoTpFrame::FlowControlFrame(ctx) =>
                                        self.on_flow_ctrl_frame(&channel, ctx),
                                }
                            },
                            Err(e) => {
                                log::warn!("ISO-TP(CAN sync) - data convert to frame failed: {}", e);
                                if let Err(e) = self.context.state_add(&channel, IsoTpState::Error) {
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
