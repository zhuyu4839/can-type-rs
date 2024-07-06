use std::fmt::{Display, Formatter, Write};
use crate::{Direct, Protocol};
use crate::identifier::Id;

/// CAN 2.0
pub trait Frame<T: Display> {
    
    fn new(id: impl Into<Id>, data: &[u8]) -> Option<Self>
        where Self: Sized;
    
    fn new_remote(id: impl Into<Id>, len: usize) -> Option<Self>
        where Self: Sized;
    
    fn timestamp(&self) -> u64;
    
    fn set_timestamp(&mut self, value: Option<u64>) -> &mut Self
        where Self: Sized;
    
    fn id(&self, protocol: Protocol) -> Id;
    
    fn is_can_fd(&self) -> bool;
    
    fn set_can_fd(&mut self, value: bool) -> &mut Self
        where Self: Sized;
    
    fn is_remote(&self) -> bool;
    
    fn is_extended(&self) -> bool;
    
    fn direct(&self) -> Direct;
    
    fn set_direct(&mut self, direct: Direct) -> &mut Self
        where Self: Sized;
    
    fn is_bitrate_switch(&self) -> bool;
    
    fn set_bitrate_switch(&mut self, value: bool) -> &mut Self
        where Self: Sized;
    
    fn is_error_frame(&self) -> bool;
    
    fn set_error_frame(&mut self, value: bool) -> &mut Self
        where Self: Sized;

    /// Error state indicator
    fn is_esi(&self) -> bool;

    /// Set error state indicator
    fn set_esi(&mut self, value: bool) -> &mut Self
        where Self: Sized;
    
    fn channel(&self) -> T;
    
    fn set_channel(&mut self, value: T) -> &mut Self
        where Self: Sized;
    
    fn data(&self) -> &[u8];
    
    fn dlc(&self) -> usize;
    
    fn length(&self) -> usize;
}

impl<T: Display> Display for dyn Frame<T> {
    /// Output Frame as `asc` String.
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let data_str = if self.is_remote() {
            " ".to_owned()
        } else {
            self.data().iter()
                .fold(String::new(), |mut out, &b| {
                    let _ = write!(out, "{b:02x} ");
                    out
                })
        };

        if self.is_can_fd() {
            let mut flags = 1 << 12;
            write!(f, "{:.3} CANFD {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {}",
                   self.timestamp() as f64 / 1000.,
                   self.channel(),
                   direct(self.direct()),
                   // if self.is_rx() { "Rx" } else { "Tx" },
                   format!("{: >8x}", self.id(Default::default()).as_raw()),
                   if self.is_bitrate_switch() {
                       flags |= 1 << 13;
                       1
                   } else { 0 },
                   if self.is_esi() {
                       flags |= 1 << 14;
                       1
                   } else { 0 },
                   format!("{: >2}", self.dlc()),
                   format!("{: >2}", self.length()),
                   data_str,
                   format!("{: >8}", 0),       // message_duration
                   format!("{: <4}", 0),       // message_length
                   format!("{: >8x}", flags),
                   format!("{: >8}", 0),       // crc
                   format!("{: >8}", 0),       // bit_timing_conf_arb
                   format!("{: >8}", 0),       // bit_timing_conf_data
                   format!("{: >8}", 0),       // bit_timing_conf_ext_arb
                   format!("{: >8}", 0),       // bit_timing_conf_ext_data
            )
        }
        else {
            write!(f, "{:.3} {} {}{: <4} {} {} {} {}",
                   self.timestamp() as f64 / 1000.,
                   self.channel(),
                   format!("{: >8x}", self.id(Default::default()).as_raw()),
                   if self.is_extended() { "x" } else { "" },
                   direct(self.direct()),
                   // if self.is_rx() { "Rx" } else { "Tx" },
                   if self.is_remote() { "r" } else { "d" },
                   format!("{: >2}", self.length()),
                   data_str,
            )
        }
    }
}

#[inline]
fn direct<'a>(direct: Direct) -> &'a str {
    match direct {
        Direct::Transmit => "Tx",
        Direct::Receive => "Rx",
    }
}
