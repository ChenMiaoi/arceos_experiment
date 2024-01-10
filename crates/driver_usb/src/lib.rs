//! Common traits and types for graphics display device drivers.

#![no_std]

#[doc(no_inline)]
pub use driver_common::{BaseDriverOps, DevError, DevResult, DeviceType};
pub mod host;

use axhal::mem::phys_to_virt;

use log::info;



