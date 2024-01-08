//! Common traits and types for xhci device drivers.

#![no_std]
#![feature(strict_provenance)]

use core::{alloc::Layout, num::NonZeroUsize};

use axhal::mem::{phys_to_virt, PhysAddr};
use command_ring_stuff::command_ring;
#[doc(no_inline)]
pub use driver_common::{BaseDriverOps, DeviceType};
use log::info;
use xhci::{
    accessor::Mapper,
    extended_capabilities::{self},
    ring::trb::event,
    ExtendedCapability, Registers,
};

pub struct XhciController {
    pub controller: Option<Registers<MemoryMapper>>,
    extended_cap: Option<extended_capabilities::List<MemoryMapper>>,
    mapper: Option<MemoryMapper>,
    base_addr: usize,
}

pub const VL805_VENDOR_ID: u16 = 0x1106;
pub const VL805_DEVICE_ID: u16 = 0x3483;
pub const VL805_MMIO_BASE: usize = 0x6_0000_0000;

pub mod command_ring_stuff;
use command_ring_stuff::command_type::CommandType;

use crate::command_ring_stuff::command_ring::CommandRing;
pub mod register_operations_init_xhci;

/// The information of the graphics device.
#[derive(Debug, Clone, Copy)]
pub struct XhciInfo {}

#[derive(Clone, Copy)]
struct MemoryMapper;

impl Mapper for MemoryMapper {
    unsafe fn map(&mut self, phys_base: usize, bytes: usize) -> NonZeroUsize {
        // info!("mapping:{:x}", phys_base);
        return NonZeroUsize::new_unchecked(phys_to_virt(phys_base.into()).as_usize());
    }

    fn unmap(&mut self, virt_base: usize, bytes: usize) {}
}

impl XhciController {
    pub fn init(
        pci_bar_address: usize,
        bar_size: usize,
        cap_offset_usize: usize,
    ) -> XhciController {
        // let config_enable = phys_to_virt(PhysAddr::from(0xFA000000));
        // let config_enable: usize = 0x6_0000_0000;
        // unsafe {
        //     info!("writing!");
        //     // while let stat = (*(config_enable.as_usize() as *const u16)) as u16 == 0x10 {
        //     while let stat = (*(config_enable as *const u16)) as u16 == 0x10 {
        //         *((add + 0x04) as *mut u16) = 326;
        //         info!("status:{}", stat);
        //     }
        //     info!("writed!");
        // }

        info!(
            "received address:{:x},offset:{:x},offseted:{:x}",
            pci_bar_address,
            cap_offset_usize,
            pci_bar_address + cap_offset_usize
        );

        let memory_mapper = MemoryMapper {};
        let register = unsafe { xhci::Registers::new(pci_bar_address, memory_mapper) };
        let read_volatile = register.capability.hccparams1.read_volatile();

        let extended_cap = unsafe {
            extended_capabilities::List::new(pci_bar_address, read_volatile, MemoryMapper)
        };

        let mut xhci_controller = XhciController {
            base_addr: pci_bar_address,
            mapper: Some(memory_mapper),
            controller: Some(register),
            extended_cap,
        };

        xhci_controller.startup();
        xhci_controller.enable_interrupt();
        // xhci_controller.register_interrupt_handler();
        xhci_controller.enable_ports();

        xhci_controller
    }

    // 初始化控制器
    fn startup(&mut self) {
        let mut r = self.controller.as_mut().unwrap();

        // 获取操作寄存器的引用
        let o = &mut r.operational;

        // 清除状态位和中断使能位
        o.usbsts.update_volatile(|r| {
            r.clear_event_interrupt();
            r.clear_host_system_error();
            r.clear_port_change_detect();
            r.clear_save_restore_error();
        });
        o.usbcmd.update_volatile(|u| {
            u.clear_interrupter_enable();
        });

        // 为控制器分配设备上下文数组
        let dcbaap = unsafe {
            axalloc::global_allocator()
                .alloc(Layout::from_size_align_unchecked(256 * 8, 1024))
                .unwrap()
                .addr()
                .get() as u64
        };
        o.dcbaap.update_volatile(|r| r.set(dcbaap as u64));

        // // 为控制器分配事件环
        let erst = unsafe {
            axalloc::global_allocator()
                .alloc(Layout::from_size_align_unchecked(16 * 8, 1024))
                .unwrap()
                .addr()
                .get() as u64
        };

        let mut interrupter_mut = r.interrupter_register_set.interrupter_mut(0);
        interrupter_mut.erstsz.update_volatile(|r| r.0 = 1);
        interrupter_mut
            .erstba
            .update_volatile(|r| r.set(erst as u64));

        // // 将事件环的消费者循环索引写入操作寄存器
        interrupter_mut
            .erdp
            .update_volatile(|r| r.set_event_ring_dequeue_pointer(erst as u64));

        // 将控制器的运行/停止位置为1，以启动控制器
        o.usbcmd.update_volatile(|r| {
            r.set_run_stop();
        });
        while o.usbsts.read_volatile().hc_halted() {}

        info!(
            "status:not_ready-{}",
            o.usbsts.read_volatile().controller_not_ready()
        );
        info!("startup completed");
    }

    // 启用中断
    fn enable_interrupt(&mut self) {
        info!("enable interrupting");
        // 获取寄存器组的引用
        let registers = self.controller.as_mut().unwrap();
        let mut operational = &mut registers.operational;

        // 获取中断管理寄存器和中断调节寄存器
        info!("enable interrupting1");
        let iman = &mut registers.interrupter_register_set.interrupter_mut(0).iman;
        let imod = &mut registers.interrupter_register_set.interrupter_mut(0).imod;

        // 启用中断并设置中断间隔为4000微秒和中断计数器为0
        iman.update_volatile(|i| {
            i.set_interrupt_enable();
        });
        info!("enable interrupting2");
        imod.update_volatile(|r| {
            r.set_interrupt_moderation_interval(4000);
            r.set_interrupt_moderation_counter(0);
        });

        //启用扩展功能
        let ext = self.extended_cap.as_mut().unwrap();
        let iter_mut = ext.into_iter();
        info!("enable interrupting3");
        for c in iter_mut.filter_map(Result::ok) {
            if let ExtendedCapability::UsbLegacySupport(mut u) = c {
                let l = &mut u.usblegsup;
                l.update_volatile(|s| {
                    s.set_hc_os_owned_semaphore();
                });

                while l.read_volatile().hc_bios_owned_semaphore()
                    || !l.read_volatile().hc_os_owned_semaphore()
                {}
            }
        }

        info!("enable interrupting4");
        let mut command_ring = CommandRing::new(
            phys_to_virt(PhysAddr::from(
                registers
                    .interrupter_register_set
                    .interrupter(0)
                    .erstba
                    .read_volatile()
                    .get() as usize,
            ))
            .as_usize() as *mut u64,
        );

        info!("enable interrupting5");
        operational
            .crcr
            .update_volatile(|r| r.set_command_ring_pointer(command_ring.address() as u64));

        xhci::ring::trb::event::TransferEvent::new();
        // 创建一个使能中断的命令TRB
        info!("enable interrupting6");
        let enable_interrupt_trb = command_ring.create_command_trb(|t| {
            info!("creating! 1");
            // 设置命令类型为使能中断
            t.set_type(CommandType::EnableInterrupt);
            info!("creating! 2");
            // 设置中断目标为0
            t.set_interrupt_target(0);
            info!("creating! 3");
            // 设置循环位为1
            t.set_cycle_bit(1);
            info!("creating! done");
        });

        // 将命令TRB加入命令环
        info!("enable interrupting7");
        command_ring.push_command_trb(enable_interrupt_trb);

        // .interrupter_register_set
        // .interrupter_mut(0)
        // .iman
        // // 选择端口状态改变事件和传输完成事件作为中断源
        // .update_volatile(|u| {
        //     u.set_port_status_change_event_enable();
        //     u.set_transfer_event_enable();
        // });

        info!("get msi addr");
        let msix_table_address =
            unsafe { self.mapper.unwrap().map(self.base_addr + 0x92, 2) }.get();

        // // 获取MSI-X表的指针
        let msix_table_ptr = unsafe { (msix_table_address as *mut u64).as_mut().unwrap() };

        info!("msix_table:{:x}", *msix_table_ptr);

        // unsafe {
        //     // 设置中断向量的地址为0xfee00000，数据为0x00000030
        //     *msix_table_ptr = 0xfee00000;
        //     *(msix_table_ptr + 1) = 0x00000030;
        // }
    }

    // // 注册中断处理函数
    // fn register_interrupt_handler(&self) {
    //     // 定义一个中断处理函数
    //     extern "C" fn xhci_interrupt_handler() {
    //         // 读取事件环的生产者循环索引
    //         let erst = unsafe { &mut *(0x251000 as *mut u32) };
    //         let producer_cycle_state = erst.read_volatile() & 1;

    //         // 读取事件环的消费者循环索引
    //         let erdp = unsafe { &mut *(0x251010 as *mut u32) };
    //         let consumer_cycle_state = erdp.read_volatile() & 1;

    //         // 如果生产者和消费者的循环状态相同，说明有新的事件
    //         if producer_cycle_state == consumer_cycle_state {
    //             // 读取事件环的当前事件
    //             let trb = unsafe { &mut *(erdp.read_volatile() as *mut u32) };

    //             // 根据事件的类型和参数执行相应的操作
    //             match trb.read_volatile() >> 10 & 0b111111 {
    //                 0b000001 => info!("Transfer Event"),
    //                 0b000100 => info!("Port Status Change Event"),
    //                 _ => info!("Unknown Event"),
    //             }

    //             // 更新消费者循环索引和循环状态
    //             erdp.write_volatile(erdp.read_volatile() + 16);
    //             erdp.update(|e| e ^ 1);
    //         }

    //         // 发送EOI（结束中断）信号
    //         unsafe {
    //             PICS.lock().notify_end_of_interrupt(0x30);
    //         }
    //     }

    //     // 注册中断处理函数到IDT中
    //     unsafe {
    //         idt::set_gate(0x30, xhci_interrupt_handler as usize, 0x8E);
    //     }

    //     // 加载IDT到CPU中
    //     unsafe {
    //         idt::load();
    //     }
    // }

    // 启用端口
    fn enable_ports(&mut self) {
        // 获取寄存器组的引用
        let r = self.controller.as_mut().unwrap();

        // 获取端口的数量
        let port_count = r.capability.hcsparams1.read().number_of_ports();

        // 遍历每个端口
        for i in 1..port_count {
            // 获取端口的状态和控制寄存器
            let portrs = &mut r.port_register_set.read_volatile_at(i as usize);
            info!(
                "port{} status(error:{},link_state:{},power:{})",
                i,
                portrs.portli.link_error_count(),
                portrs.portsc.port_link_state(),
                portrs.portsc.port_power()
            );

            let mut portsc = portrs.portsc;
            // 检查端口是否连接了设备
            if portsc.current_connect_status() {
                // 重置端口
                portsc.set_port_reset();
                while portsc.port_reset() {}

                // 使能端口
                portsc.set_0_port_enabled_disabled_change(); //核查一下是不是这个寄存器
                while !portsc.port_enabled_disabled() {}

                // 配置端口
                // 设置端口速度
                // p.set_port_speed(PortSpeed::SuperSpeed);
                // 设置端口功率
                portsc.set_port_power();
                // 设置端口链路状态
                portsc.set_port_link_state(0);
            }
        }
    }
}

pub trait XhciDriverOps: BaseDriverOps {
    /// Get the display information.
    fn info(&self) -> XhciInfo;
}

impl BaseDriverOps for XhciController {
    fn device_name(&self) -> &str {
        //todo  unimplemented!();
        "xhci-controller"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::XHCI
    }
}

impl XhciDriverOps for XhciController {
    fn info(&self) -> XhciInfo {
        todo!()
    }
}
