use core::{usize, mem::size_of, ptr::{slice_from_raw_parts_mut, slice_from_raw_parts}, alloc::Layout, time::Duration};

use axhal::mem::{PAGE_SIZE_4K, phys_to_virt, virt_to_phys};
use log::debug;
use xhci::registers::runtime;

use super::arm_mailbox::MailBox;

pub const BCM_MAILBOX_PROP_OUT: u32 = 8;
const GPU_MEM_BASE: usize = 0x40000000;
const CORES: usize = axconfig::SMP;
const MEM_KERNEL_START: usize = axconfig::KERNEL_BASE_PADDR;
const MEGABYTE: usize = 0x100000;
const MEM_COHERENT_REGION: usize = 0x500000;

pub struct PropertyTags{

}

fn bus_address(addr: usize)-> usize{
    // addr | GPU_MEM_BASE
    (addr  & !GPU_MEM_BASE) | GPU_MEM_BASE
}

fn get_coherent_page(n_slot: usize)->usize{
    MEM_COHERENT_REGION + n_slot* PAGE_SIZE_4K
}

impl PropertyTags{
    pub fn get(
        // tag: &TProperyTag
    )->Self{        
        let mailbox = MailBox::new(BCM_MAILBOX_PROP_OUT);
        // let p_buffer_phy = get_coherent_page(0);

        let p_buffer = phys_to_virt(0x100_000.into()).as_usize();

        // let layout = Layout::from_size_align(16 * 12 + 1 << 16, 16).unwrap();
        // let vaddr = axalloc::global_allocator().alloc(layout).unwrap();
        // let p_buffer = vaddr.as_ptr() as usize * 1 << 16;

        // let buffer = bus_address(p_buffer_phy);
        // let p_buffer = phys_to_virt(buffer.into()).as_usize();

        debug!("p_buffer: @virt {:x}", p_buffer);

        let buffer = unsafe{&mut*(p_buffer  as *mut TPropertyBuffer)};

        debug!("buffer: {:p}", buffer);

        // unsafe{
        //     let data_ptr = tag as *const TProperyTag as *const u8;
        //     let tag_data = &*slice_from_raw_parts(data_ptr, size_of::<TProperyTag>());
        //     let tag_len = tag_data.len();
        //     header.n_code = PropertyCode::Request;
        //     let header_len = size_of::<TPropertyBuffer>() ;
        //     header.bffer_size = (header_len + tag_len + size_of::<u32>()) as u32;

        //     let tag_ptr = (p_buffer + header_len)as  *mut u8;

        //     let tag_dst = &mut *slice_from_raw_parts_mut(tag_ptr, tag_len);

        //     tag_dst.copy_from_slice(tag_data);

        //     let end = (p_buffer+ header_len + tag_len) as *mut u32;

        //     *end = 0;
        // }

        unsafe{
            let buffer_size = size_of::<TPropertyBuffer>()+ size_of::<TProperyTag>();
            let buffer_size_with_end = buffer_size + size_of::<u32>();
            
            debug!("buffer_size_with_end: {:x}", buffer_size_with_end);

            buffer.n_code = PropertyCode::Request;
            buffer.bffer_size = buffer_size_with_end as u32;
            buffer.tag.tag_id= PropTag::GetFirmwareRevision;
            buffer.tag.value_bufsize=0;
            buffer.tag.code_and_value_len=0;
            *((p_buffer +  buffer_size) as *mut u32) = 0;
            
        }

        debug!("buffer {:?}", buffer);

        let send_addr = p_buffer;
        // let send_addr = virt_to_phys(p_buffer.into()).as_usize();

        // 发送
        let send_addr = bus_address(send_addr);
        use aarch64_cpu::asm::barrier::{self, SY};
        
        barrier::dsb(SY);

        let result = mailbox.write_read(send_addr as u32);

        barrier::dmb(SY);



        debug!("read: 0x{:X}", result);
        // if (send_addr != result as usize){
        //     panic!("send_addr: {:x}, result: {:x}", send_addr, result);
        // }

        debug!("wait for result...");

        axhal::time::busy_wait(Duration::from_secs(1));
        // while buffer.n_code == PropertyCode::Request {}

        debug!("tag result: {:?}", buffer);

        Self{}
    }
}


#[repr(u32)]
#[derive(Debug)]
pub enum PropTag{
    NotifyXhciReset = 0x00030058,
    GetFirmwareRevision = 0x1,
}
#[repr(u32)]
#[derive(Debug, PartialEq, Eq)]
pub enum PropertyCode{
    Request = 0x00000000,
    ResponseSuccess = 0x80000000,
    ResponseFailure = 0x80000001,
}



#[repr(C)]
#[derive(Debug)]
pub struct TProperyTag{
    tag_id: PropTag,
    value_bufsize: u32,
    code_and_value_len: u32,
}
#[repr(C)]
#[derive(Debug)]
struct TPropertyBuffer
{
	bffer_size: u32,			// bytes
	n_code: PropertyCode,
    tag: TProperyTag,
}
