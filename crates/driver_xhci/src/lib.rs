//! Common traits and types for xhci device drivers.

#![no_std]
#![feature(strict_provenance)]
pub mod command_ring;
pub mod event_ring;
pub mod register_operations_init_xhci;

use core::{alloc::Layout, num::NonZeroUsize};

use axhal::mem::{phys_to_virt, PhysAddr};
use command_ring::CommandRing;
#[doc(no_inline)]
pub use driver_common::{BaseDriverOps, DeviceType};
use event_ring::EventRing;
use log::info;
use xhci::{
    accessor::Mapper,
    extended_capabilities::{self},
    ring::trb::event,
    ExtendedCapability, Registers,
};

pub const VL805_VENDOR_ID: u16 = 0x1106;
pub const VL805_DEVICE_ID: u16 = 0x3483;
pub const VL805_MMIO_BASE: usize = 0x6_0000_0000;

pub struct XhciController {
    base_addr: usize,
    mapper: Option<MemoryMapper>,
    controller: Option<Registers<MemoryMapper>>,
    extended_cap: Option<extended_capabilities::List<MemoryMapper>>,
    event_ring: EventRing,
    command_ring: CommandRing,
}
/// The information of the graphics device.
#[derive(Debug, Clone, Copy)]
pub struct XhciInfo {}
#[derive(Clone, Copy)]
struct MemoryMapper;

impl Mapper for MemoryMapper {
    unsafe fn map(&mut self, phys_base: usize, bytes: usize) -> NonZeroUsize {
        // info!("mapping:{:x}", phys_base);
        return NonZeroUsize::new_unchecked(phys_to_virt(phys_base.into()).as_usize());
    }

    fn unmap(&mut self, virt_base: usize, bytes: usize) {}
}

impl XhciController {
    pub fn init(
        pci_bar_address: usize,
        bar_size: usize,
        cap_offset_usize: usize,
    ) -> XhciController {
        info!(
            "received address:{:x},offset:{:x},offseted:{:x}",
            pci_bar_address,
            cap_offset_usize,
            pci_bar_address + cap_offset_usize
        );

        let memory_mapper = MemoryMapper {};
        let register = unsafe { xhci::Registers::new(pci_bar_address, memory_mapper) };
        let read_volatile = register.capability.hccparams1.read_volatile();

        let extended_cap = unsafe {
            extended_capabilities::List::new(pci_bar_address, read_volatile, MemoryMapper)
        };

        let mut xhci_controller = XhciController {
            base_addr: pci_bar_address,
            mapper: Some(memory_mapper),
            controller: Some(register),
            extended_cap,
            event_ring: (),
            command_ring: (),
        };

        xhci_controller.startup_xhci();
        xhci_controller.configure_event_ring();
        //TODO configure command ring,dcbaa , scratch pad,exchanger,etc.

        xhci_controller
    }

    // 初始化控制器
    fn startup_xhci(&mut self) {
        let registers = self.controller.unwrap();
        let mut operational = registers.operational;

        operational.usbcmd.update_volatile(|r| {
            r.clear_run_stop();
        });

        while !operational.usbsts.read_volatile().hc_halted() {}

        operational.usbcmd.update_volatile(|u| {
            u.set_host_controller_reset();
        });

        while operational.usbcmd.read_volatile().host_controller_reset() {}
        while r.operational.usbsts.read_volatile().controller_not_ready() {}

        operational.config.update_volatile(|c| {
            c.set_max_device_slots_enabled(
                registers
                    .capability
                    .hcsparams1
                    .read_volatile()
                    .number_of_device_slots(),
            );
        });

        return ();
    }

    fn configure_event_ring(&mut self) {
        self.event_ring = EventRing::new(self.controller.unwrap());
        let mut event_ring = &mut self.event_ring;

        event_ring.update_deq_with_xhci(self.controller);
        event_ring.init_segtable(self.controller);
    }
}

pub trait XhciDriverOps: BaseDriverOps {
    /// Get the display information.
    fn info(&self) -> XhciInfo;
}

impl BaseDriverOps for XhciController {
    fn device_name(&self) -> &str {
        //todo  unimplemented!();
        "xhci-controller"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::XHCI
    }
}

impl XhciDriverOps for XhciController {
    fn info(&self) -> XhciInfo {
        todo!()
    }
}
