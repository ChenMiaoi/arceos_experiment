pub use crate::bcm::raspberrypi::MailBoxImpl;
use memory_addr::{VirtAddr, PhysAddr};

pub trait Page: Clone {
    fn phys_to_virt(paddr: PhysAddr) -> VirtAddr;
}




pub trait MailBoxMessage {
    fn as_data<'a>(&'a self)-> &'a [u8];
}

pub trait  MailBox{
    fn send(&self, msg: impl MailBoxMessage);
}


struct Buffer{
    
}