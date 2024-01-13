use super::arm_mailbox::MailBox;

pub const BCM_MAILBOX_PROP_OUT: u32 = 8;


pub struct PropertyTags{

}



impl PropertyTags{
    pub fn get(tag: PropTag, )->Self{
        let mailbox = MailBox::new(BCM_MAILBOX_PROP_OUT);


        Self{}
    }
}


#[repr(C)]
pub enum PropTag{
    NotifyXhciReset = 0x00030058,

}