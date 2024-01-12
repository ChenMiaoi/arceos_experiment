use core::{fmt::Display, ptr::NonNull};
use log::info;
use axhal::mem::VirtAddr;
use tock_registers::{
    interfaces::{ReadWriteable, Readable, Writeable},
    register_bitfields, register_structs,
    registers::{ReadOnly, ReadWrite},
};

register_bitfields![
    u16,
    PCI_COMMAND_REG[
        INTRDIS OFFSET(10) NUMBITS(1) [],
        SERREN OFFSET(8) NUMBITS(1) [],
    ]
];


register_structs! {
    /// VL805 header registers.
    VL805HeaderRegisters {
        (0x0 => vid: ReadOnly<u16>),
        (0x2 => devid: ReadOnly<u16>),
        (0x4 => pci_command: ReadWrite<u16, PCI_COMMAND_REG::Register>),
        (0x6 => @END),
    }
}



pub(crate) struct  VL805Header{
    base: NonNull<VL805HeaderRegisters>,
}


impl VL805Header {
    pub(crate) fn new(vaddr: VirtAddr) -> VL805Header {
        VL805Header{
            base: NonNull::new(vaddr.as_mut_ptr()).unwrap().cast(),
        }
    }

    fn regs(&self)->&VL805HeaderRegisters{
        unsafe{ self.base.as_ref() }
    }    

    pub fn log_info(&self){
       let regs = self.regs();
       info!("VID: {:x} DEVID: {:x}", regs.vid.get(), regs.devid.get());
    }
}

