//! Device driver prelude that includes some traits and types.

pub use driver_common::{BaseDriverOps, DevError, DevResult, DeviceType};

#[cfg(feature = "block")]
pub use {crate::structs::AxBlockDevice, driver_block::BlockDriverOps};
#[cfg(feature = "display")]
pub use {crate::structs::AxDisplayDevice, driver_display::DisplayDriverOps};
#[cfg(feature = "net")]
pub use {crate::structs::AxNetDevice, driver_net::NetDriverOps};
#[cfg(feature = "xhci")]
pub use {crate::structs::AxXHciDevice, driver_xhci::XhciDriverOps};
#[cfg(feature = "usb")]
pub use {crate::structs::AxUSBDevice, driver_usb::USBDriverOps};