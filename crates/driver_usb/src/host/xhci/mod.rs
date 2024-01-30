#[cfg(feature = "vl805")]
pub mod vl805;
use axhal::mem::{phys_to_virt, PhysAddr, VirtAddr};
use core::{alloc::Layout, num::NonZeroUsize, ptr::NonNull};
use log::info;
use xhci::accessor::Mapper;

mod command_ring;
mod event_ring;
pub mod xhci_controller;

#[derive(Clone, Copy)]
struct MemoryMapper;

impl Mapper for MemoryMapper {
    unsafe fn map(&mut self, phys_base: usize, bytes: usize) -> NonZeroUsize {
        let virt = phys_to_virt(phys_base.into());
        // info!("mapping: [{:x}]->[{:x}]", phys_base, virt.as_usize());
        return NonZeroUsize::new_unchecked(virt.as_usize());
    }

    fn unmap(&mut self, virt_base: usize, bytes: usize) {}
}
