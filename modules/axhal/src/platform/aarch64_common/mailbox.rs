use core::{mem::size_of, ptr::{slice_from_raw_parts_mut, slice_from_raw_parts}, cell::UnsafeCell};


#[repr(u32)]
pub(crate) enum RpiFirmwarePropertyStatus {
    Request = 0,
    Success = 0x80000000,
    Error = 0x80000001,
}

#[derive(Debug, Default)]
pub(crate) struct RpiFirmwarePropertyTagHeader {
    pub tag: u32,
    pub buf_size: u32,
    pub req_resp_size: u32,
}

#[repr(u32)]
pub enum RpiFirmwarePropertyTag {
    PropertyEnd = 0,
    GetFirmwareRevision = 0x00000001,
    GetFirmwareVariant = 0x00000002,
    GetFirmwareHash = 0x00000003,

    SetCursorInfo = 0x00008010,
    SetCursorState = 0x00008011,

    GetBoardModel = 0x00010001,
    GetBoardRevision = 0x00010002,
    GetBoardMacAddress = 0x00010003,
    GetBoardSerial = 0x00010004,
    GetArmMemory = 0x00010005,
    GetVcMemory = 0x00010006,
    GetClocks = 0x00010007,
    GetPowerState = 0x00020001,
    GetTiming = 0x00020002,
    SetPowerState = 0x00028001,
    GetClockState = 0x00030001,
    GetClockRate = 0x00030002,
    GetVoltage = 0x00030003,
    GetMaxClockRate = 0x00030004,
    GetMaxVoltage = 0x00030005,
    GetTemperature = 0x00030006,
    GetMinClockRate = 0x00030007,
    GetMinVoltage = 0x00030008,
    GetTurbo = 0x00030009,
    GetMaxTemperature = 0x0003000a,
    GetStc = 0x0003000b,
    AllocateMemory = 0x0003000c,
    LockMemory = 0x0003000d,
    UnlockMemory = 0x0003000e,
    ReleaseMemory = 0x0003000f,
    ExecuteCode = 0x00030010,
    ExecuteQpu = 0x00030011,
    SetEnableQpu = 0x00030012,
    GetDispmanxResourceMemHandle = 0x00030014,
    GetEdidBlock = 0x00030020,
    GetCustomerOtp = 0x00030021,
    GetEdidBlockDisplay = 0x00030023,
    GetDomainState = 0x00030030,
    GetThrottled = 0x00030046,
    GetClockMeasured = 0x00030047,
    NotifyReboot = 0x00030048,
    SetClockState = 0x00038001,
    SetClockRate = 0x00038002,
    SetVoltage = 0x00038003,
    SetTurbo = 0x00038009,
    SetCustomerOtp = 0x00038021,
    SetDomainState = 0x00038030,
    GetGpioState = 0x00030041,
    SetGpioState = 0x00038041,
    SetSdhostClock = 0x00038042,
    GetGpioConfig = 0x00030043,
    SetGpioConfig = 0x00038043,
    GetPeriphReg = 0x00030045,
    SetPeriphReg = 0x00038045,
    GetPoeHatVal = 0x00030049,
    SetPoeHatVal = 0x00038049,
    SetPoeHatValOld = 0x00030050,
    NotifyXhciReset = 0x00030058,
    GetRebootFlags = 0x00030064,
    SetRebootFlags = 0x00038064,
    NotifyDisplayDone = 0x00030066,
    GetButtonsPressed = 0x00030088,

    // Dispmanx TAGS
    FramebufferAllocate = 0x00040001,
    FramebufferBlank = 0x00040002,
    FramebufferGetPhysicalWidthHeight = 0x00040003,
    FramebufferGetVirtualWidthHeight = 0x00040004,
    FramebufferGetDepth = 0x00040005,
    FramebufferGetPixelOrder = 0x00040006,
    FramebufferGetAlphaMode = 0x00040007,
    FramebufferGetPitch = 0x00040008,
    FramebufferGetVirtualOffset = 0x00040009,
    FramebufferGetOverscan = 0x0004000a,
    FramebufferGetPalette = 0x0004000b,
    FramebufferGetLayer = 0x0004000c,
    FramebufferGetTransform = 0x0004000d,
    FramebufferGetVsync = 0x0004000e,
    FramebufferGetTouchbuf = 0x0004000f,
    FramebufferGetGpiovirtbuf = 0x00040010,
    FramebufferRelease = 0x00048001,
    FramebufferGetDisplayId = 0x00040016,
    FramebufferSetDisplayNum = 0x00048013,
    FramebufferGetNumDisplays = 0x00040013,
    FramebufferGetDisplaySettings = 0x00040014,
    FramebufferTestPhysicalWidthHeight = 0x00044003,
    FramebufferTestVirtualWidthHeight = 0x00044004,
    FramebufferTestDepth = 0x00044005,
    FramebufferTestPixelOrder = 0x00044006,
    FramebufferTestAlphaMode = 0x00044007,
    FramebufferTestVirtualOffset = 0x00044009,
    FramebufferTestOverscan = 0x0004400a,
    FramebufferTestLayer = 0x0004400c,
    FramebufferTestTransform = 0x0004400d,
    FramebufferTestVsync = 0x0004400e,
    FramebufferSetPhysicalWidthHeight = 0x00048003,
    FramebufferSetVirtualWidthHeight = 0x00048004,
    FramebufferSetDepth = 0x00048005,
    FramebufferSetPixelOrder = 0x00048006,
    FramebufferSetAlphaMode = 0x00048007,
    FramebufferSetPitch = 0x00048008,
    FramebufferSetVirtualOffset = 0x00048009,
    FramebufferSetOverscan = 0x0004800a,
    FramebufferSetPalette = 0x0004800b,

    FramebufferSetTouchbuf = 0x0004801f,
    FramebufferSetGpiovirtbuf = 0x00048020,
    FramebufferSetVsync = 0x0004800e,
    FramebufferSetLayer = 0x0004800c,
    FramebufferSetTransform = 0x0004800d,
    FramebufferSetBacklight = 0x0004800f,

    VchiqInit = 0x00048010,

    SetPlane = 0x00048015,
    GetDisplayTiming = 0x00040017,
    SetTiming = 0x00048017,
    GetDisplayCfg = 0x00040018,
    SetDisplayPower = 0x00048019,
    GetCommandLine = 0x00050001,
    GetDmaChannels = 0x00060001,
}

const ARM_IO_BASE: usize = 0xFE000000;
const MAIL_BOX_BASE: usize = ARM_IO_BASE + 0xB880;
const MAILBOX_STATUS_EMPTY: u32 = 0x40000000;
const MAILBOX_STATUS_FULL: u32 = 0x80000000;
const MAILBOX0_READ: usize = MAIL_BOX_BASE + 0x00;
const MAILBOX0_STATUS: usize = MAIL_BOX_BASE + 0x18;
const MAILBOX1_WRITE: usize = MAIL_BOX_BASE + 0x20;
const MAILBOX1_STATUS: usize = MAIL_BOX_BASE + 0x38;

pub const BCM_MAILBOX_PROP_OUT: u32 = 8;
const GPU_MEM_BASE: usize = 0xC0000000;
use crate:: mailbox_re_set;

pub struct MailBoxImpl {
    n_channel: u32,
}
impl MailBoxImpl {
    pub fn new(n_channel: u32) -> Self {
        Self { n_channel }
    }

    fn read(&self) -> u32 {
        while read32(MAILBOX0_STATUS) == MAILBOX_STATUS_EMPTY {
            // println!("Mailbox is empty");
        }

        loop {
            let r = read32(MAILBOX0_READ);
            if (r & 0xf) == self.n_channel {
                return r & !0xf;
            }
        }
    }

    fn write(&self, data: u32) {
        while read32(MAILBOX1_STATUS) == MAILBOX_STATUS_FULL {
            // println!("Mailbox is full");
        }
        // info!("mailbox write 0x{:X}", data);
        write32(MAILBOX1_WRITE, data | self.n_channel);
    }
    pub fn flush(&self) {
        loop {
            let r = read32(MAILBOX0_STATUS);
            if r == MAILBOX_STATUS_EMPTY {
                return;
            }
            read32(MAILBOX0_READ);
        }
    }

    pub fn write_read(&self, data: u32) -> u32 {
        self.flush();
        // info!("flush ok");
        self.write(data);
        while read32(MAILBOX1_STATUS) != MAILBOX_STATUS_EMPTY {
            // println!("Mailbox is full");
        }
        // info!("write ok");
        self.read()
    }

    pub fn send(&self) {
        // let data = msg.as_data();
        // let buffer = 0x10_0000usize;
        let buffer = 0x10_0000usize;

        // let addr = bus_address(buffer) as u32; 
        let addr = bus_address(buffer) as u32; 

        // mailbox_tag
        unsafe {
            let mut i = 0;
            let num_fill = 1;

            // let data = [0u32; 20];


            // let p = &mut *slice_from_raw_parts_mut(buffer as *mut u8, 80);
            let p = &mut *slice_from_raw_parts_mut(buffer as *mut u32, 20);
            // let data = &* slice_from_raw_parts(data.as_ptr() as *const u8, 80);

            // p.copy_from_slice(data);

            p[i] = 0; // size. Filled in below
            i += 1;
            p[i] = 0x00000000;
            i += 1;
            p[i] = 1; // get version
            i += 1;
            p[i] = 1 << 2;
            i += 1;
            p[i] = 0 << 2;
            i += 1;
            for j in 0..num_fill {
                 p[i] = 0x00000000;
                i += 1;
            }
            p[i] = 0x00000000; // end tag
            i += 1;
            p[0] = (i * size_of::<u32>()) as u32; // actual size

            // buff.copy_from_slice(data);
        }


        // info!("mailbox write addr: {:X},  phy_addr: {:X}", addr, phy);
        let out = self.write_read(addr);
        // info!("mailbox read addr: {:X}", out);

        unsafe {
            loop {
                let p = &mut *slice_from_raw_parts_mut(buffer as *mut u32, 0x50);
                let r = p[1];
                if r != 0 {
                    // mailbox_re_set(r) ;
                    break;
                }
            }
        }
    }
}

fn read32(addr: usize) -> u32 {
    unsafe { (addr as *const u32).read_volatile() }
}
fn write32(addr: usize, data: u32) -> () {
    unsafe { (addr as *mut u32).write_volatile(data) }
}
fn bus_address(addr: usize) -> usize {
    // addr | GPU_MEM_BASE
    (addr & !GPU_MEM_BASE) | GPU_MEM_BASE
}

pub struct MsgGetFirmwareRevision {}
