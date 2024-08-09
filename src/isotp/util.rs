#![allow(unused_imports, dead_code)]

#[cfg(feature = "std2004")]
mod std2004;
#[cfg(feature = "std2004")]
pub(crate) use std2004::*;
#[cfg(feature = "std2016")]
mod std2016;
#[cfg(feature = "std2016")]
pub(crate) use std2016::*;

use isotp_rs::{FlowControlContext, FrameType, IsoTpFrame};
use isotp_rs::error::Error as IsoTpError;
use crate::constant::{CAN_FRAME_MAX_SIZE, CANFD_FRAME_MAX_SIZE};
use crate::isotp::CanIsoTpFrame;

#[cfg(not(feature = "can-fd"))]
pub(crate) const SINGLE_FRAME_SIZE_2004: usize = CAN_FRAME_MAX_SIZE - 1;
#[cfg(feature = "can-fd")]
pub(crate) const SINGLE_FRAME_SIZE_2004: usize = CANFD_FRAME_MAX_SIZE - 1;
#[cfg(not(feature = "can-fd"))]
pub(crate) const SINGLE_FRAME_SIZE_2016: usize = CAN_FRAME_MAX_SIZE - 2;
#[cfg(feature = "can-fd")]
pub(crate) const SINGLE_FRAME_SIZE_2016: usize = CANFD_FRAME_MAX_SIZE - 2;

#[cfg(not(feature = "can-fd"))]
pub(crate) const FIRST_FRAME_SIZE_2004: usize = CAN_FRAME_MAX_SIZE - 2;
#[cfg(feature = "can-fd")]
pub(crate) const FIRST_FRAME_SIZE_2004: usize = CANFD_FRAME_MAX_SIZE - 2;
#[cfg(not(feature = "can-fd"))]
pub(crate) const FIRST_FRAME_SIZE_2016: usize = CAN_FRAME_MAX_SIZE - 5;
#[cfg(feature = "can-fd")]
pub(crate) const FIRST_FRAME_SIZE_2016: usize = CANFD_FRAME_MAX_SIZE - 5;

#[cfg(not(feature = "can-fd"))]
pub(crate) const CONSECUTIVE_FRAME_SIZE: usize = CAN_FRAME_MAX_SIZE - 1;
#[cfg(feature = "can-fd")]
pub(crate) const CONSECUTIVE_FRAME_SIZE: usize = CANFD_FRAME_MAX_SIZE - 1;

#[cfg(feature = "can-fd")]
#[inline]
fn can_fd_resize(length: usize) -> Option<usize> {
    match length {
        ..=CAN_FRAME_MAX_SIZE => Some(length),
        9..=12 =>  Some(12),
        13..=16 => Some(16),
        17..=20 => Some(20),
        21..=24 => Some(24),
        25..=32 => Some(32),
        33..=48 => Some(48),
        49..=64 => Some(64),
        _ => None,
    }
}

#[inline]
fn add_flow_control(results: &mut Vec<CanIsoTpFrame>, flow_ctrl: &Vec<FlowControlContext>) {
    if flow_ctrl.len() == 0 {
        results.push(CanIsoTpFrame::default_flow_ctrl_frame());
    }
    else {
        for fc in flow_ctrl {
            results.push(CanIsoTpFrame::flow_ctrl_frame(
                fc.state(), fc.block_size(), fc.st_min()
            ));
        }
    }
}

fn parse<const FIRST_FRAME_SIZE: usize>(data: &[u8],
                                        offset: &mut usize,
                                        sequence: &mut u8,
                                        results: &mut Vec<CanIsoTpFrame>,
                                        flow_ctrl: Vec<FlowControlContext>,
                                        length: usize,
) {
    loop {
        match *offset {
            0 => {
                *offset += FIRST_FRAME_SIZE;
                let frame = CanIsoTpFrame::FirstFrame {
                    length: length as u32,
                    data: Vec::from(&data[..*offset])
                };
                results.push(frame);

                add_flow_control(results, &flow_ctrl);

                continue;
            },
            _ => {
                if *offset + CONSECUTIVE_FRAME_SIZE >= length {
                    let frame = CanIsoTpFrame::ConsecutiveFrame {
                        sequence: *sequence,
                        data: Vec::from(&data[*offset..length])
                    };
                    results.push(frame);
                    break;
                }

                let frame = CanIsoTpFrame::ConsecutiveFrame {
                    sequence: *sequence,
                    data: Vec::from(&data[*offset..*offset + CONSECUTIVE_FRAME_SIZE])
                };
                *offset += CONSECUTIVE_FRAME_SIZE;
                if *sequence >= 0x0F {
                    *sequence = 0;
                }
                else {
                    *sequence += 1;
                }

                results.push(frame);
            }
        }
    }
}

#[inline]
fn new_single_u<const N: usize, T: AsRef<[u8]>>(data: T) -> Result<CanIsoTpFrame, IsoTpError> {
    let data = data.as_ref();
    let length = data.len();
    match length {
        0 => Err(IsoTpError::EmptyPdu),
        1..=SINGLE_FRAME_SIZE_2016 => {
            let mut result = vec![FrameType::Single as u8 | length as u8];
            result.append(&mut data.to_vec());
            // result.resize(CAN_FRAME_MAX_SIZE, Default::default());
            Ok(CanIsoTpFrame::SingleFrame { data: result })
        },
        v => Err(IsoTpError::LengthOutOfRange(v)),
    }
}
