mod register_operations_init_xhci;
mod regs;
use super::MemoryMapper;
use crate::host::xhci::{vl805::{self, register_operations_init_xhci::enable_xhci}, propertytags::{PropTag, TProperyTag, PropertyTags}};
pub use crate::host::USBHostDriverOps;
use axhal::{
    cpu,
    mem::{phys_to_virt, PhysAddr, VirtAddr},
};
use core::ptr::NonNull;
#[doc(no_inline)]
pub use driver_common::{BaseDriverOps, DeviceType};
use driver_pci::DeviceFunction;
use log::{debug, info};
use pci_types::{capability::PciCapabilityAddress, PciAddress};
use xhci::extended_capabilities::debug;

const VL805_VENDOR_ID: u16 = 0x1106;
const VL805_DEVICE_ID: u16 = 0x3483;
const VL805_MMIO_BASE: usize = 0x6_0000_0000;

pub struct VL805 {
    pci_base: VirtAddr,
    bdf: DeviceFunction,
}

impl BaseDriverOps for VL805 {
    fn device_name(&self) -> &str {
        "VL805 4-Port USB 3.0 Host Controller"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::USBHost
    }
}

impl USBHostDriverOps for VL805 {}

impl VL805 {
    fn new(pci_base: VirtAddr, bdf: DeviceFunction) -> Self {
        VL805 { pci_base, bdf }
    }
    fn get_config_addr(&self, bdf: DeviceFunction, offset: usize) -> VirtAddr {
        let config =
            driver_pci::get_pci_config_addr(self.pci_base.as_usize(), bdf, driver_pci::Cam::Ecam);

        (config + offset).into()
    }
}

impl pci_types::ConfigRegionAccess for VL805 {
    fn function_exists(&self, address: PciAddress) -> bool {
        true
    }

    unsafe fn read(&self, address: PciAddress, offset: u16) -> u32 {
        let addr = self.get_config_addr(
            DeviceFunction {
                bus: address.bus(),
                device: address.device(),
                function: address.function(),
            },
            offset as usize,
        );

        unsafe { *(addr.as_mut_ptr() as *mut u32) }
    }

    unsafe fn write(&self, address: PciAddress, offset: u16, value: u32) {
        let addr = self.get_config_addr(
            DeviceFunction {
                bus: address.bus(),
                device: address.device(),
                function: address.function(),
            },
            offset as usize,
        );

        unsafe {
            *(addr.as_mut_ptr() as *mut u32) = value;
        }
    }
}

impl VL805 {
    pub fn probe_pci(
        vendor_id: u16,
        device_id: u16,
        bdf: DeviceFunction,
        pci_base: VirtAddr,
        bar: usize,
    ) -> Option<Self> {
        if !(vendor_id == VL805_VENDOR_ID && device_id == VL805_DEVICE_ID) {
            return None;
        }

        let mapper = MemoryMapper;
        let bar_virt = phys_to_virt(bar.into());

        let vl805 = VL805::new(pci_base, bdf);

        let header =
            pci_types::PciHeader::new(PciAddress::new(0, bdf.bus, bdf.device, bdf.function));

        

        let ep = pci_types::EndpointHeader::from_header(header, &vl805);
        if let Some(ep) = ep {
            let ep_stat = ep.status(&vl805);
            let caps: pci_types::capability::CapabilityIterator<'_, VL805> =
                ep.capabilities(&vl805);
            for cap in caps {
                match cap {
                    pci_types::capability::PciCapability::PowerManagement(ref addr) => {
                        vl805.init_pme(addr.clone())
                    }
                    _ => {}
                }

                debug!("cap: {:?}", cap);
            }
        }

        let regs = unsafe { xhci::Registers::new(bar, mapper) };
        
        

        let version = regs.capability.hciversion.read_volatile();

        debug!("xhci version: {:x}", version.get());
        let mut o = regs.operational;

        let hcsp1 = regs.capability.hcsparams1.read_volatile();

        debug!("xhci max slots: {}, max ports: {}", hcsp1.number_of_device_slots(), hcsp1.number_of_ports());

        const DEV_ADDR:u32 = 1<<20| 0<<15 | 0<<12;
        // let tag = TProperyTag::new(PropTag::NotifyXhciReset, DEV_ADDR);
        // PropertyTags::get(&tag);

        PropertyTags::get();

        debug!("xhci stat: {:?}", o.usbsts.read_volatile());

        debug!("xhci wait for ready...");
        while o.usbsts.read_volatile().controller_not_ready() {}
        info!("xhci ok");

        Some(vl805)
    }

    fn init_pme(&self, addr: PciCapabilityAddress) {
        debug!("init pme");
        let addr = self.get_config_addr(self.bdf, addr.offset as usize);

        let regs = NonNull::<PCIPMERegisters>::new(addr.as_mut_ptr() as *mut _).unwrap();
        let regs = unsafe { regs.as_ref() };
        let pc = &regs.pc;

        debug!(
            "pme: {:?} version {:?} D1: {} D2: {}",
            pc.read(PCI_PME_PC_REG::PSUP),
            pc.read(PCI_PME_PC_REG::VS),
            pc.read(PCI_PME_PC_REG::D1S),
            pc.read(PCI_PME_PC_REG::D2S)
        );

        let cs = &regs.pmcs;
        // cs.matches_any(&[PCI_PME_CS_REG::PS::D0, PCI_PME_CS_REG::PMEE::SET]);
        debug!(
            "pme enable: {} ps: {:?} nsfrst: {}", 
            cs.read(PCI_PME_CS_REG::PMEE),
            cs.read(PCI_PME_CS_REG::PS),
            cs.read(PCI_PME_CS_REG::NSFRST),
        );
    }
}

use tock_registers::{
    interfaces::{ReadWriteable, Readable, Writeable},
    register_bitfields, register_structs,
    registers::{ReadOnly, ReadWrite},
};

register_bitfields![
    u16,
    PCI_PME_PC_REG[
        PSUP OFFSET(11) NUMBITS(5) [],
        D2S OFFSET(10) NUMBITS(1) [],
        D1S OFFSET(9) NUMBITS(1) [],
        AUXC OFFSET(6) NUMBITS(3) [],
        DSI OFFSET(5) NUMBITS(1) [],
        _RSV OFFSET(4) NUMBITS(1) [],
        PMEC OFFSET(3) NUMBITS(1) [],
        VS OFFSET(0) NUMBITS(3) [],
    ],
    PCI_PME_CS_REG[
        PMES OFFSET(15) NUMBITS(1) [],
        DSC OFFSET(13) NUMBITS(2) [],
        DSE OFFSET(9) NUMBITS(4) [],
        PMEE OFFSET(8) NUMBITS(1) [],
        _RSV OFFSET(4) NUMBITS(4) [],
        NSFRST OFFSET(3) NUMBITS(1) [],
        _RSV2 OFFSET(2) NUMBITS(1) [],
        PS OFFSET(0) NUMBITS(2) [
            D0=0b00,
            D1=0b01,
            D2= 0b10,
            D3hot=0b11,
        ],
    ],
];

register_structs! {
    /// VL805 header registers.
    PCIPMERegisters {
        (0x0 => pid: ReadOnly<u16>),
        (0x2 => pc: ReadOnly<u16, PCI_PME_PC_REG::Register>),
        (0x4 => pmcs: ReadWrite<u16, PCI_PME_CS_REG::Register>),
        (0x6 => @END),
    }
}
