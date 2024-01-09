//! [ArceOS](https://github.com/rcore-os/arceos) graphics module.
//!
//! Currently only supports direct writing to the framebuffer.

#![no_std]

#[macro_use]
extern crate log;

#[doc(no_inline)]
pub use driver_usb::*;

use axdriver::{prelude::*, AxDeviceContainer};
use axsync::Mutex;
use lazy_init::LazyInit;

