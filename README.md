## CAN type define that based from [can-types](https://crates.io/crates/can-types) crate

### This crate is deprecated. All code is move to [isopt-rs](https://github.com/zhuyu4839/isotp-rs/)

  * `CanMessage` example

  ```rust
use std::fmt::Display;

pub struct CanMessage {
    // fields of message declare
}

impl Frame for CanMessage {
    type Channel = u8;
    // impl methods that defined in `Frame` trait
}

impl Display for CanMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        <dyn Frame<Channel=u8> as Display>::fmt(self, f)
    }
}
  ```
  
  * `AsyncCanDevice` and `SyncCanDevice` [example](https://github.com/zhuyu4839/zlgcan-driver-rs/tree/master/zlgcan-driver/src/extends/mod.rs)
