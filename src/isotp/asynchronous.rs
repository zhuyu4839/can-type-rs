use std::sync::mpsc::Sender;
use std::time::Duration;
use isotp_rs::error::Error as IsoTpError;
use isotp_rs::{FrameType, IsoTpFrame, IsoTpState};
use tokio::time::sleep;
use crate::frame::Frame;
use crate::identifier::Id;
use crate::isotp::{Address, CanIsoTpFrame, CanIsoTp};
use crate::isotp::context::IsoTpContext;

#[derive(Clone)]
pub struct AsyncCanIsoTp<Channel: Clone + Eq> {
    context: IsoTpContext<Channel>,
}

impl<Channel: Clone + Eq + 'static> CanIsoTp for AsyncCanIsoTp<Channel> {
    type Channel = Channel;
    fn new(channel: Self::Channel, address: Address) -> Self {
        Self {
            context: IsoTpContext::new(channel, address),
        }
    }
    fn context(&self) -> &IsoTpContext<Self::Channel> {
        &self.context
    }

    fn mut_context(&mut self) -> &mut IsoTpContext<Self::Channel> {
        &mut self.context
    }
}

impl<Channel: Clone + Eq + 'static> AsyncCanIsoTp<Channel> {
    pub async fn write<F: Frame>(
        &mut self,
        sender: Sender<F>,
        functional: bool,
        data: Vec<u8>,
    ) -> Result<(), IsoTpError> {
        log::debug!("ISO-TP(CAN async) - Sending: {:?}", data);

        // self.write_p2_star(&channel, p2_server.p2_star).await?;

        let address = self.context.address()?;
        let frames = CanIsoTpFrame::from_data(data, vec![])?;
        let length = frames.len() - 1;

        for (index, frame) in frames.into_iter().enumerate() {
            self.write_wait().await?;

            let frame_type = FrameType::from(&frame);

            let can_id = if functional { address.fid } else { address.tx_id };
            let frame = F::from_iso_tp(Id::from_bits(can_id, false), frame, None)
                .ok_or(IsoTpError::ConvertError {
                    src: "iso-tp frame",
                    target: "can-frame",
                })?;

            self.on_frame_writing(frame_type, index < length).await?;
            if let Err(e) = sender.send(frame) {
                log::warn!("ISO-TP(CAN async) - transmit failed: {:?}", e);
                self.context.state_add(IsoTpState::Error)?;

                break;
            }

            // self.write_p2(&channel, &p2_server).await?;
        }

        Ok(())
    }

    /// before frame write
    async fn on_frame_writing(&mut self,
                              frame_type: FrameType,
                              is_last: bool,
    ) -> Result<(), IsoTpError> {
        self.context.clear_listener_buffer()?;
        match frame_type {
            FrameType::Single => {
                self.context.state_add(
                    IsoTpState::Sending | IsoTpState::WaitSingle | IsoTpState::WaitFirst
                )
            },
            FrameType::First => {
                self.context.state_add(
                    IsoTpState::Sending | IsoTpState::WaitFlowCtrl
                )
            },
            FrameType::Consecutive => {
                sleep(Duration::from_micros(self.context.st_min()? as u64)).await;    // sleep st_min

                if is_last {
                    self.context.state_add(
                        IsoTpState::Sending | IsoTpState::WaitSingle | IsoTpState::WaitFirst
                    )
                }
                else {
                    self.context.state_add(
                        IsoTpState::Sending | IsoTpState::WaitData
                    )
                }
            },
            FrameType::FlowControl => self.context.state_add(
                IsoTpState::Sending | IsoTpState::Idle
            ),
        }
    }

    async fn write_wait(&self) -> Result<(), IsoTpError> {
        while self.context.write_waiting()? {
            let state = self.context.state()?;
            if state.contains(IsoTpState::Sending | IsoTpState::ResponsePending) {
                sleep(Duration::from_millis(10)).await;
            }
            // else if state.contains(IsoTpState::ResponsePending) {
            //     sleep(Duration::from_millis(50)).await;
            // }
            else {
                break;
            }
        }

        Ok(())
    }
}
