use crate::host::xhci::event_ring::EventRing;

use super::{
    command_ring::{self, CommandRing},
    event_ring, MemoryMapper,
};
use core::num::NonZeroUsize;

use aarch64_cpu::asm::barrier;
use axhal::mem::phys_to_virt;
#[doc(no_inline)]
pub use driver_common::{BaseDriverOps, DeviceType};
use log::info;
use page_table_entry::aarch64;
use xhci::{
    accessor::Mapper,
    extended_capabilities::{self},
    Registers,
};
pub struct XhciController {
    base_addr: usize,
    mapper: Option<MemoryMapper>,
    controller: Option<Registers<MemoryMapper>>,
    extended_cap: Option<extended_capabilities::List<MemoryMapper>>,
    event_ring: Option<EventRing>,
    command_ring: Option<CommandRing>,
}

impl XhciController {
    pub fn init(pci_bar_address: usize) -> XhciController {
        info!("received address:{:x}", pci_bar_address,);

        // let mapper = MemoryMapper;
        // let regs = unsafe { xhci::Registers::new(address, mapper) };
        // let version = regs.capability.hciversion.read_volatile();
        // debug!("xhci version: {:x}", version.get());
        // let mut o = regs.operational;
        // debug!("xhci stat: {:?}", o.usbsts.read_volatile());

        // debug!("xhci wait for ready...");
        // while o.usbsts.read_volatile().controller_not_ready() {}
        // info!("xhci ok");

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
            event_ring: None,
            command_ring: None,
        };
        barrier::isb(barrier::SY);
        xhci_controller.startup_xhci();
        xhci_controller.configure_event_ring();
        //TODO configure command ring,dcbaa , scratch pad,exchanger,etc.

        xhci_controler
    }

    // 初始化控制器
    fn startup_xhci(&mut self) {
        info!("startup_xhci");
        let r = self.controller.as_mut().unwrap();
        let mut operational = &mut r.operational;

        info!(
            "xhci version: {:x}",
            r.capability.hciversion.read_volatile().get()
        );

        barrier::isb(barrier::SY);
        operational.usbcmd.update_volatile(|r| {
            r.clear_run_stop();
        });

        info!("waiting halt");
        while !operational.usbsts.read_volatile().hc_halted() {}

        barrier::isb(barrier::SY);
        operational.usbcmd.update_volatile(|u| {
            u.set_host_controller_reset();
        });

        barrier::isb(barrier::SY);
        info!("waiting reset");
        while operational.usbcmd.read_volatile().host_controller_reset() {}

        info!("xhci stat: {:?}", operational.usbsts.read_volatile());

        barrier::isb(barrier::SY);
        info!("waiting for ready");
        while operational.usbsts.read_volatile().controller_not_ready() {}

        operational.config.update_volatile(|c| {
            c.set_max_device_slots_enabled(
                r.capability
                    .hcsparams1
                    .read_volatile()
                    .number_of_device_slots(),
            );
        });

        barrier::isb(barrier::SY);
        info!("xhci ok");

        return ();
    }

    fn configure_event_ring(&mut self) {
        info!("configure_event_ring");
        self.event_ring = Some(EventRing::new(self.controller.as_mut().unwrap()));
        let mut event_ring = self.event_ring.as_mut().unwrap();

        event_ring.update_deq_with_xhci(self.controller.as_mut().unwrap());
        event_ring.init_segtable(self.controller.as_mut().unwrap());
    }
}
