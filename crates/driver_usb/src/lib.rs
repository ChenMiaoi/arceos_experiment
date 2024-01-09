//! Common traits and types for graphics display device drivers.

#![no_std]

#[doc(no_inline)]
pub use driver_common::{BaseDriverOps, DevError, DevResult, DeviceType};
pub mod xhci;


use axhal::mem::phys_to_virt;

use log::info;



/// The information of the graphics device.
#[derive(Debug, Clone, Copy)]
pub struct USBInfo {

}



/// Operations that require a graphics device driver to implement.
pub trait USBDriverOps: BaseDriverOps {

}