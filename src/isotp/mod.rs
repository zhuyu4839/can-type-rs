pub mod client;
mod util;
// pub mod server;


use isotp_rs::{FlowControlContext, FlowControlState, FrameType, IsoTpFrame};
use isotp_rs::constant::{P2_ISO14229, P2_STAR_ISO14229};
use isotp_rs::error::Error as IsoTpError;
use crate::constant::*;

#[derive(Debug, Clone, Copy)]
pub struct P2Context {
    pub p2: u16,        // ms
    pub p2_offset: u16, // ms
    pub p2_star: u32,   // ms
}

impl Default for P2Context {
    fn default() -> Self {
        Self {
            p2: P2_ISO14229,
            p2_offset: 0,
            p2_star: P2_STAR_ISO14229,
        }
    }
}

/// ISO-TP address format.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AddressFormat {
    // UNKNOWN = 0xFF,
    // None = 0x00,
    #[default]
    Normal = 0x01,      // 11bit CAN-ID
    NormalFixed = 0x02, // 29bit CAN-ID
    Extend = 0x03,      // 11bit Remote CAN-ID
    ExtendMixed = 0x04, // 11bit and 29bit Remote CAN-ID mixed
    Enhanced = 0x05,
}

/// ISO-TP address
///
/// * `tx_id`: transmit identifier.
/// * `rx_id`: receive identifier.
/// * `fid`: functional address identifier.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Address {
    pub tx_id: u32,
    pub rx_id: u32,
    pub fid: u32,
}

/// ISO-TP address type.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub enum AddressType {
    #[default]
    Physical,
    Functional,
}

/// ISO-TP frame define.
#[derive(Debug, Clone)]
pub enum CanIsoTpFrame {
    /// The ISO-TP single frame.
    SingleFrame { data: Vec<u8> },
    /// The ISO-TP first frame.
    FirstFrame { length: u32, data: Vec<u8> },
    /// The ISO-TP consecutive frame.
    ConsecutiveFrame { sequence: u8, data: Vec<u8> },
    /// The ISO-TP flow control frame.
    FlowControlFrame(FlowControlContext)
}

unsafe impl Send for CanIsoTpFrame {}

impl IsoTpFrame for CanIsoTpFrame {
    fn decode<T: AsRef<[u8]>>(data: T) -> Result<Self, IsoTpError> {
        let data = data.as_ref();
        let length = data.len();
        match length {
            0 => Err(IsoTpError::EmptyPdu),
            1..=2 => Err(IsoTpError::InvalidPdu(data.to_vec())),
            3.. => {
                let byte0 = data[0];
                match FrameType::try_from(byte0)? {
                    FrameType::Single => {   // Single frame
                        util::decode_single(data, byte0, length)
                    },
                    FrameType::First => {   // First frame
                        util::decode_first(data, byte0, length)
                    },
                    FrameType::Consecutive => {
                        let sequence = byte0 & 0x0F;
                        Ok(Self::ConsecutiveFrame { sequence, data: Vec::from(&data[1..]) })
                    },
                    FrameType::FlowControl => {
                        let data1 = data[1];
                        // let suppress_positive = (data1 & 0x80) == 0x80;
                        let state = FlowControlState::try_from(data1 & 0x7F)?;
                        let st_min = data[2];
                        Ok(Self::FlowControlFrame(
                            FlowControlContext::new(state, data1, st_min)
                        ))
                    },
                }
            }
            // v => Err(IsoTpError::LengthOutOfRange(v)),
        }
    }

    fn encode(self, padding: Option<u8>) -> Vec<u8> {
        match self {
            Self::SingleFrame { data } => {
                util::encode_single(data, padding)
            },
            Self::FirstFrame { length, data } => {
                util::encode_first(length, data)
            },
            Self::ConsecutiveFrame { sequence, mut data } => {
                let mut result = vec![FrameType::Consecutive as u8 | sequence];
                result.append(&mut data);
                result.resize(CAN_FRAME_MAX_SIZE, padding.unwrap_or(Default::default()));
                result
            },
            Self::FlowControlFrame(context) => {
                let byte0_h: u8 = FrameType::FlowControl.into();
                let byte0_l: u8 = context.state().into();
                let mut result = vec![
                    byte0_h | byte0_l,
                    context.block_size(),
                    context.st_min(),
                ];
                result.resize(CAN_FRAME_MAX_SIZE, padding.unwrap_or(Default::default()));
                result
            },
        }
    }

    fn from_data<T: AsRef<[u8]>>(data: T, flow_ctrl: Vec<FlowControlContext>) -> Result<Vec<Self>, IsoTpError> {
        util::from_data(data.as_ref(), flow_ctrl)
    }

    fn single_frame<T: AsRef<[u8]>>(data: T) -> Result<Self, IsoTpError> {
        util::new_single(data)
    }

    fn flow_ctrl_frame(state: FlowControlState,
                       block_size: u8,
                       st_min: u8,
    ) -> Self {
        Self::FlowControlFrame(
            FlowControlContext::new(state, block_size, st_min)
        )
    }
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;
    use isotp_rs::IsoTpFrame;
    use crate::constant::CAN_FRAME_MAX_SIZE;
    use crate::isotp::util::{CONSECUTIVE_FRAME_SIZE, FIRST_FRAME_SIZE_2004};
    use super::{FlowControlState, CanIsoTpFrame};

    #[test]
    fn test_single() -> anyhow::Result<()> {
        let data = hex!("02 10 01 00 00 00 00 00").as_slice();
        let frame = CanIsoTpFrame::decode(data)?;
        match frame.clone() {
            CanIsoTpFrame::SingleFrame { data } => {
                assert_eq!(data, hex!("1001"));
            },
            _ => {
                panic!("Invalid frame type");
            }
        }
        assert_eq!(frame.encode(Some(0x00)), data.to_vec());

        let frame = CanIsoTpFrame::SingleFrame { data: hex!("1001").to_vec() };
        assert_eq!(frame.encode(Some(0x00)), data.to_vec());
        Ok(())
    }

    #[test]
    fn test_first() -> anyhow::Result<()> {
        let data = hex!("10 0f 62 f1 87 44 56 43");
        let frame = CanIsoTpFrame::decode(data)?;
        match frame.clone() {
            CanIsoTpFrame::FirstFrame { length, data } => {
                assert_eq!(length, 0x0f);
                assert_eq!(data, hex!("62 f1 87 44 56 43"));
            },
            _ => {
                panic!("Invalid frame type");
            }
        }
        assert_eq!(frame.encode(None), data.to_vec());

        let frame = CanIsoTpFrame::FirstFrame {
            length: 0x0f,
            data: hex!("62 f1 87 44 56 43").to_vec()
        };
        assert_eq!(frame.encode(None), data.to_vec());

        Ok(())
    }

    #[test]
    fn test_consecutive() -> anyhow::Result<()> {
        let data = hex!("21 37 45 32 30 30 30 30");
        let frame = CanIsoTpFrame::decode(data)?;
        match frame.clone() {
            CanIsoTpFrame::ConsecutiveFrame { sequence, data } => {
                assert_eq!(sequence, 1);
                assert_eq!(data, hex!("37 45 32 30 30 30 30"));
            },
            _ => {
                panic!("Invalid frame type");
            }
        }
        assert_eq!(frame.encode(None), data.to_vec());

        let frame = CanIsoTpFrame::ConsecutiveFrame {
            sequence: 1,
            data: hex!("37 45 32 30 30 30 30").to_vec()
        };
        assert_eq!(frame.encode(None), data.to_vec());
        Ok(())
    }

    #[test]
    fn test_flow_control() -> anyhow::Result<()> {
        let data = hex!("30 80 01 55 55 55 55 55").as_slice();
        let frame = CanIsoTpFrame::decode(data)?;
        match frame.clone() {
            CanIsoTpFrame::FlowControlFrame(context) => {
                assert_eq!(context.state(), FlowControlState::Continues);
                assert_eq!(context.block_size(), 0x80);
                assert_eq!(context.st_min(), 0x01);
            },
            _ => {
                panic!("Invalid frame type");
            }
        }
        assert_eq!(frame.encode(Some(0x55)), data.to_vec());

        let frame = CanIsoTpFrame::default_flow_ctrl_frame();
        assert_eq!(frame.encode(Some(0x55)), hex!("30 00 0a 55 55 55 55 55"));
        Ok(())
    }

    #[test]
    fn test_data_to_multi() -> anyhow::Result<()> {
        let data = hex!("62 f1 87 44 56 43 37 45 32 30 30 30 30 30 37").as_slice();
        let frames = CanIsoTpFrame::from_data(data, vec![])?;
        for (index, frame) in frames.into_iter().enumerate() {
            match index {
                0 => {
                    assert_eq!(frame.encode(None), hex!("10 0f 62 f1 87 44 56 43").to_vec());
                },
                1 => {
                    assert_eq!(frame.encode(None), hex!("30 00 0a 00 00 00 00 00").to_vec());
                },
                2 => {
                    assert_eq!(frame.encode(None), hex!("21 37 45 32 30 30 30 30").to_vec());
                },
                3 => assert_eq!(frame.encode(None), hex!("22 30 37 00 00 00 00 00").to_vec()),
                _ => panic!()
            }
        }

        let mut size = 0x96;
        let data = vec![0x30; size];
        let frames = CanIsoTpFrame::from_data(data, vec![])?;
        for (index, frame) in frames.into_iter().enumerate() {
            match index {
                0 => {
                    size -= FIRST_FRAME_SIZE_2004;
                    assert_eq!(frame.encode(None), hex!("10 96 30 30 30 30 30 30"))
                },
                1 => {
                    assert_eq!(frame.encode(None), hex!("30 00 0a 00 00 00 00 00").to_vec());
                },
                2..=15 => {
                    size -= CONSECUTIVE_FRAME_SIZE;
                    let expect = vec![0x20 + (index - 1) as u8, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30];
                    assert_eq!(frame.encode(None), expect);
                }
                _ => {
                    if size > CONSECUTIVE_FRAME_SIZE {
                        size -= CONSECUTIVE_FRAME_SIZE;
                        let expect = vec![0x20 + ((index - 1) % 16) as u8, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30];
                        assert_eq!(frame.encode(None), expect);
                    }
                    else {
                        let mut expect = vec![0x20 + ((index - 1) % 16) as u8];
                        for _ in 0..size {
                            expect.push(0x30);
                        }
                        expect.resize(CAN_FRAME_MAX_SIZE, Default::default());
                        assert_eq!(frame.encode(None), expect);
                    }
                },
            }
        }
        Ok(())
    }
}
