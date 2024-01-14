const ARM_IO_BASE: usize = 0xFE000000;
const MAIL_BOX_BASE: usize = ARM_IO_BASE + 0xB880;
const MAILBOX_STATUS_EMPTY: u32 = 0x40000000;
const MAILBOX_STATUS_FULL: u32 = 0x80000000;
const MAILBOX0_READ: usize = MAIL_BOX_BASE + 0x00;
const MAILBOX0_STATUS: usize = MAIL_BOX_BASE + 0x18;
const MAILBOX1_WRITE: usize = MAIL_BOX_BASE + 0x20;
const MAILBOX1_STATUS: usize = MAIL_BOX_BASE + 0x38;



use core::time::Duration;

use aarch64_cpu::asm::ret;
use axhal::{mem::phys_to_virt, time};
use log::debug;
use tock_registers::{
    interfaces::{ReadWriteable, Readable, Writeable},
    register_bitfields, register_structs,
    registers::{ReadOnly, ReadWrite, WriteOnly},
};
use xhci::extended_capabilities::debug;

pub struct MailBox {
    n_channel: u32,
}

impl MailBox {
    pub fn new(n_channel: u32) -> Self {
        Self { n_channel }
    }

    pub fn read(&self) -> u32 {
        while read32(MAILBOX0_STATUS) == MAILBOX_STATUS_EMPTY{
            //println!("Mailbox is empty");
        }

        loop {
            let r = read32(MAILBOX0_READ);
            if (r & 0xf) == self.n_channel {
                // return r >> 4;
                return r & !0xf;
            }
        }
    }

    pub fn write(&self, data: u32) -> () {
        while read32(MAILBOX1_STATUS) == MAILBOX_STATUS_FULL {
            //println!("Mailbox is full");
        }
        // let data = data << 4;
        debug!("mailbox write {:x}", data);
        write32(MAILBOX1_WRITE, data  | self.n_channel);
    }

    pub fn flush(&self){
        loop{
            let r = read32(MAILBOX0_STATUS);
            if r == MAILBOX_STATUS_EMPTY {
                return;
            }
            read32(MAILBOX0_READ);
            time::busy_wait(Duration::from_millis(20));
        }
    }

    pub fn write_read(&self, data: u32)->u32{
        self.flush();
        debug!("flush ok");
        self.write(data);
        debug!("write ok");
        self.read()
    }


}
fn read32(addr: usize) -> u32 {
    let vaddr = phys_to_virt(addr.into());
    unsafe { *(vaddr.as_ptr() as *const u32) }
}
fn write32(addr: usize, data: u32) -> () {
    let vaddr = phys_to_virt(addr.into());
    unsafe {
        *(vaddr.as_mut_ptr() as *mut u32) = data;
    }
}

