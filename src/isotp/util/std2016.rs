use isotp_rs::error::Error as IsoTpError;
use isotp_rs::{FlowControlContext, FrameType};
use isotp_rs::constant::{ISO_TP_MAX_LENGTH_2004, ISO_TP_MAX_LENGTH_2016};
use crate::constant::{CAN_FRAME_MAX_SIZE, CANFD_FRAME_MAX_SIZE, DEFAULT_PADDING};
use crate::isotp::CanIsoTpFrame;
use crate::isotp::util::{CONSECUTIVE_FRAME_SIZE, FIRST_FRAME_SIZE_2004, FIRST_FRAME_SIZE_2016, parse, SINGLE_FRAME_SIZE_2004, SINGLE_FRAME_SIZE_2016};

#[cfg(feature = "can-fd")]
use crate::isotp::util::can_fd_resize;

pub(crate) fn decode_single(data: &[u8],
                            byte0: u8,
                            length: usize
) -> Result<CanIsoTpFrame, IsoTpError> {
    #[cfg(feature = "can-fd")]
    let max_len = CANFD_FRAME_MAX_SIZE;
    #[cfg(not(feature = "can-fd"))]
    let max_len = CAN_FRAME_MAX_SIZE;

    if length > max_len {
        return Err(IsoTpError::LengthOutOfRange(length));
    }

    let mut pdu_len = byte0 & 0x0F;
    return if pdu_len > 0 {
        if length < pdu_len as usize + 1 {
            return Err(IsoTpError::InvalidPdu(data.to_vec()));
        }

        Ok(CanIsoTpFrame::SingleFrame { data: Vec::from(&data[1..=pdu_len as usize]) })
    } else {
        pdu_len = data[1];
        if length < pdu_len as usize + 2 {
            return Err(IsoTpError::InvalidPdu(data.to_vec()));
        }
        Ok(CanIsoTpFrame::SingleFrame { data: Vec::from(&data[2..=pdu_len as usize]) })
    }
}

pub(crate) fn decode_first(data: &[u8],
                           byte0: u8,
                           length: usize,
) -> Result<CanIsoTpFrame, IsoTpError> {
    #[cfg(not(feature = "can-fd"))]
    if length != CAN_FRAME_MAX_SIZE {
        return Err(IsoTpError::InvalidDataLength { actual: length, expect: CAN_FRAME_MAX_SIZE })
    }
    #[cfg(feature = "can-fd")]
    if length != CANFD_FRAME_MAX_SIZE {
        return Err(IsoTpError::InvalidDataLength { actual: length, expect: CANFD_FRAME_MAX_SIZE })
    }

    let mut pdu_len = (byte0 as u32 & 0x0F) << 8 | data[1] as u32;
    if pdu_len > 0 {
        Ok(CanIsoTpFrame::FirstFrame { length: pdu_len, data: Vec::from(&data[2..]) })
    }
    else {
        pdu_len = u32::from_be_bytes([data[2], data[3], data[4], data[5]]);
        Ok(CanIsoTpFrame::FirstFrame { length: pdu_len, data: Vec::from(&data[6..]) })
    }
}

pub(crate) fn encode_single(mut data: Vec<u8>, padding: Option<u8>) -> Vec<u8> {
    let length = data.len();
    match length {
        ..=SINGLE_FRAME_SIZE_2004 => {
            let mut result = vec![FrameType::Single as u8 | length as u8];
            result.append(&mut data);
            #[cfg(not(feature = "can-fd"))]
            result.resize(CAN_FRAME_MAX_SIZE, padding.unwrap_or(DEFAULT_PADDING));
            #[cfg(feature = "can-fd")]
            if let Some(resize) = can_fd_resize(length) {
                result.resize(resize, padding.unwrap_or(DEFAULT_PADDING));
            }
            result
        },
        _ => {
            let mut result = vec![FrameType::Single as u8, length as u8];
            result.append(&mut data);
            #[cfg(not(feature = "can-fd"))]
            result.resize(CAN_FRAME_MAX_SIZE, padding.unwrap_or(DEFAULT_PADDING));
            #[cfg(feature = "can-fd")]
            if let Some(resize) = can_fd_resize(length) {
                result.resize(resize, padding.unwrap_or(DEFAULT_PADDING));
            }

            result
        }
    }
}

pub(crate) fn encode_first(length: u32, mut data: Vec<u8>) -> Vec<u8> {
    let mut result = if length & 0xFFFFFFFF > 0x7FF {
        let mut temp = vec![FrameType::First as u8];
        temp.extend(length.to_be_bytes());
        temp
    }
    else {
        let len_h = ((length & 0x0F00) >> 8) as u8;
        let len_l = (length & 0x00FF) as u8;
        vec![FrameType::First as u8 | len_h, len_l]
    };
    result.append(&mut data);
    result
}

pub(crate) fn new_single<T: AsRef<[u8]>>(data: T) -> Result<CanIsoTpFrame, IsoTpError> {
    let data = data.as_ref();
    let length = data.len();
    match length {
        0 => Err(IsoTpError::EmptyPdu),
        1..=SINGLE_FRAME_SIZE_2016 => {
            let mut result = vec![FrameType::Single as u8 | length as u8];
            result.append(&mut data.to_vec());
            result.resize(SINGLE_FRAME_SIZE_2016, DEFAULT_PADDING);
            Ok(CanIsoTpFrame::SingleFrame { data: result })
        },
        v => Err(IsoTpError::LengthOutOfRange(v)),
    }
}


pub(crate) fn from_data(data: &[u8]) -> Result<Vec<CanIsoTpFrame>, IsoTpError> {
    let length = data.len();
    match length {
        0 => Err(IsoTpError::EmptyPdu),
        ..=SINGLE_FRAME_SIZE_2004 => Ok(vec![CanIsoTpFrame::SingleFrame { data: data.to_vec() }]),
        ..=ISO_TP_MAX_LENGTH_2004 => {
            let mut offset = 0;
            let mut sequence = 1;
            let mut results = Vec::new();

            parse::<FIRST_FRAME_SIZE_2004>(data, &mut offset, &mut sequence, &mut results, length);

            Ok(results)
        },
        ..=ISO_TP_MAX_LENGTH_2016 => {
            let mut offset = 0;
            let mut sequence = 1;
            let mut results = Vec::new();


            parse::<FIRST_FRAME_SIZE_2016>(data, &mut offset, &mut sequence, &mut results, length);

           Ok(results)
        },
        v => Err(IsoTpError::LengthOutOfRange(v)),
    }
}


