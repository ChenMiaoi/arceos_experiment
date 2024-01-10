#[cfg(feature = "vl805")]
pub mod vl805;
use axhal::mem::phys_to_virt;
use log::info;
use xhci::accessor::Mapper;
use core::num::NonZeroUsize;


#[derive(Clone, Copy)]
struct MemoryMapper;

impl Mapper for MemoryMapper {
    unsafe fn map(&mut self, phys_base: usize, bytes: usize) -> NonZeroUsize {
        info!("mapping:{:x}", phys_base);
        return NonZeroUsize::new_unchecked(phys_to_virt(phys_base.into()).as_usize());
    }

    fn unmap(&mut self, virt_base: usize, bytes: usize) {}
}