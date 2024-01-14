use crate::bcm::raspberrypi::MailBoxAccessImpl;

pub trait MailBoxAccess{
    pub fn read(&self) -> u32;
    pub fn write(&self, data: u32);
}

pub struct MailBox<T: MailBoxAccess>{
    access: T,
}
pub trait MailBoxMessage {
    pub fn as_data<'a>(&'a self)-> &'a [u8];
}


impl <T:MailBoxAccess> MailBox<T> {
   pub fn new() -> Self {
        Self{
            access: MailBoxAccessImpl::new(),
        }
    }

    pub fn send(&self, msg: impl MailBoxMessage){
        let data = msg.as_data();
        self.access.write(0);
    }
}