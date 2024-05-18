use core::mem::MaybeUninit;
use core::{num, option, panic, result};

use alloc::string::String;
use alloc::sync::Arc;
use axalloc::GlobalNoCacheAllocator;
use conquer_once::spin::OnceCell;
use log::{debug, error, info};
use page_box::PageBox;
use spinning_top::{lock_api::Mutex, Spinlock};
use xhci::context::{Device64Byte, DeviceHandler};
use xhci::{context::Device, registers::PortRegisterSet};

use crate::host::structures::xhci_command_manager::{CommandResult, COMMAND_MANAGER};
use crate::host::structures::xhci_slot_manager::{SlotManager, SLOT_MANAGER};
use crate::{dma::DMAVec, host::structures::XHCI_CONFIG_MAX_PORTS};

use super::xhci_usb_device::XHCIUSBDevice;
use super::{registers, USBSpeed};

// 定义静态变量ROOT_HUB，用于存储根集线器的实例
pub(crate) static ROOT_HUB: OnceCell<Spinlock<Roothub>> = OnceCell::uninit();

pub struct RootPort {
    index: usize,
    device: Arc<MaybeUninit<XHCIUSBDevice>, GlobalNoCacheAllocator>,
}

impl RootPort {
    pub fn configure(&mut self) {}

    pub fn initialize(&mut self) {
        if !self.connected() {
            error!("not connected");
        }

        registers::handle(|r| {
            // r.port_register_set.read_volatile_at(self.index).portsc.port_link_state() // usb 3, not complete code
            //DEBUG lets just use usb 2 job sequence? should be compaible? might stuck at here
            r.port_register_set.update_volatile_at(self.index, |prs| {
                prs.portsc.port_reset();

                prs.portsc.set_port_reset();

                prs.portsc.set_0_port_enabled_disabled();

                debug!("waiting for port reset!");
                while !prs.portsc.port_reset() {}
            })
        });

        let get_speed = self.get_speed();
        if get_speed == USBSpeed::USBSpeedUnknown {
            error!("unknown speed, index:{}", self.index);
        }
        info!("port speed: {:?}", get_speed);

        info!("initializing device: {:?}", get_speed);

        unsafe {
            Arc::get_mut(&mut self.device)
                .unwrap()
                .write(XHCIUSBDevice::initialize())
        };
    }

    pub fn status_changed(&self) {
        // 检查MMIO（内存映射I/O），确保索引在有效范围内
        assert!(self.index < XHCI_CONFIG_MAX_PORTS);
        registers::handle(|r| {
            r.port_register_set
                .update_volatile_at(self.index, |port_register_set| {
                    // TODO: check here
                    port_register_set.portsc.clear_port_enabled_disabled();
                })
            // TODO: is plug and play support
        })
    }

    fn get_speed(&self) -> USBSpeed {
        registers::handle(|r| {
            r.port_register_set
                .read_volatile_at(self.index)
                .portsc
                .port_speed()
        })
        .into()
    }

    pub fn connected(&self) -> bool {
        registers::handle(|r| {
            r.port_register_set
                .read_volatile_at(self.index)
                .portsc
                .current_connect_status()
        })
    }
}

pub struct Roothub {
    ports: usize,
    root_ports: PageBox<[Arc<MaybeUninit<Spinlock<RootPort>>, GlobalNoCacheAllocator>]>,
}

impl Roothub {
    pub fn initialize(&mut self) {
        //todo delay?

        self.root_ports
            .iter_mut()
            .map(|a| unsafe { a.clone().assume_init() })
            .for_each(|arc| arc.lock().initialize());

        self.root_ports
            .iter_mut()
            .map(|a| unsafe { a.clone().assume_init() })
            .for_each(|arc| arc.lock().configure());
    }
}

// 当接收到根端口状态变化的通知时调用
pub(crate) fn status_changed(uch_port_id: u8) {
    // 将UCH端口ID转换为索引，并确保索引在有效范围内
    let n_port = uch_port_id as usize - 1;
    let mut root_hub = ROOT_HUB
        .try_get()
        .expect("ROOT_HUB is not initialized")
        .lock();
    assert!(n_port < root_hub.ports, "Port index out of bounds");

    // 如果端口存在，则更新其状态
    if let arc_root_port = unsafe { root_hub.root_ports[n_port].clone().assume_init() } {
        let mut root_port = arc_root_port.lock();
        //check: does clone affect value?
        root_port.status_changed();
    } else {
        panic!("Root port doesn't exist");
    }
}

pub(crate) fn new() {
    // 通过MMIO读取根集线器支持的端口数量
    registers::handle(|r| {
        let number_of_ports = r.capability.hcsparams1.read_volatile().number_of_ports() as usize;
        let mut root_ports = PageBox::new_slice(
            Arc::new_uninit_in(axalloc::global_no_cache_allocator()),
            number_of_ports,
        ); //DEBUG: using nocache allocator
        debug!("number of ports:{}", number_of_ports);
        root_ports
            .iter_mut()
            .enumerate()
            .for_each(|(i, port_uninit)| {
                Arc::get_mut(port_uninit)
                    .unwrap()
                    .write(Spinlock::new(RootPort {
                        index: i,
                        device: Arc::new_uninit_in(axalloc::global_no_cache_allocator()),
                    }));
            });
        // 初始化ROOT_HUB静态变量
        ROOT_HUB.init_once(move || {
            Roothub {
                ports: number_of_ports as usize,
                root_ports,
            }
            .into()
        })
    });

    debug!("initialized!");

    //wait 300ms

    //ininialize root port
}
