use core::ptr::NonNull;

use axhal::mem::VirtAddr;
use tock_registers::{
    interfaces::{ReadWriteable, Readable, Writeable},
    register_bitfields, register_structs,
    registers::{ReadOnly, ReadWrite},
};

// register_bitfields![
//     u16,

// ];


register_structs! {
    XHCIRegisters {
        (0x0 => caplength: ReadOnly<u8>),
        (0x1 => rsvd: ReadOnly<u8>),
        (0x2 => hciversion:ReadOnly<u16>),
        (0x4 => @END),
    }
}
pub(crate) struct Registers{
    base: NonNull<XHCIRegisters>,
}

impl Registers {
    pub fn new(base: VirtAddr) -> Self {
        Self {
            base: NonNull::new(base.as_mut_ptr()).unwrap().cast(),
        }
    }

    fn regs(&self)->&XHCIRegisters{
       unsafe{ self.base.as_ref()}
    }

    pub fn caplength(&self) -> u8 {
        self.regs().caplength.get()
    }
    pub fn hciversion(&self) -> u16 {
        self.regs().hciversion.get()
    }
}