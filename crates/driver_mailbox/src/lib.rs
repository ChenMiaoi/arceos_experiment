//! Structures and functions for PCI bus operations.
//!
//! Currently, it just re-exports structures from the crate [virtio-drivers][1]
//! and its module [`virtio_drivers::transport::pci::bus`][2].
//!
//! [1]: https://docs.rs/virtio-drivers/latest/virtio_drivers/
//! [2]: https://docs.rs/virtio-drivers/latest/virtio_drivers/transport/pci/bus/index.html

#![no_std]


pub mod mailbox;
pub mod bcm;


use axhal::mem::phys_to_virt;


pub(crate) fn read32(addr: usize) -> u32 {
    let vaddr = phys_to_virt(addr.into());
    unsafe { *(vaddr.as_ptr() as *const u32) }
}
pub(crate) fn write32(addr: usize, data: u32) -> () {
    let vaddr = phys_to_virt(addr.into());
    unsafe {
        *(vaddr.as_mut_ptr() as *mut u32) = data;
    }
}