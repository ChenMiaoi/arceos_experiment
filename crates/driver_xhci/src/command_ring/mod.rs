pub mod command_type;

use command_type::CommandTrb;
use log::info;

// 定义CommandRing结构体

pub struct CommandRing {
    // 命令环的指针
    ring_ptr: u64,
    // 命令环的大小
    ring_size: usize,
    // 生产者循环索引
    cycle_state: u32,
}

// 实现CommandRing结构体的方法
impl CommandRing {
    // 创建一个新的命令环
    pub fn new(ring_ptr: u64) -> Self {
        info!("new");
        // 为命令环分配内存
        let ring_ptr = ring_ptr;
        // 设置命令环的大小为16
        let ring_size = 16;
        // 设置生产者循环索引为0
        let cycle_state = 0;
        // 返回一个CommandRing的实例
        CommandRing {
            ring_ptr,
            ring_size,
            cycle_state,
        }
    }

    // 获取命令环的地址
    pub fn address(&self) -> usize {
        info!("address");
        // 返回命令环的指针转换为usize
        self.ring_ptr as usize
    }

    // 创建一个命令TRB
    pub fn create_command_trb<F>(&mut self, f: F) -> *mut u64
    where
        F: FnOnce(&mut CommandTrb),
    {
        // 获取命令环的当前位置
        let current_ptr = unsafe { (self.ring_ptr as *mut u64).add(self.cycle_state as usize) };
        // 获取命令TRB的引用
        let trb = unsafe { &mut *(current_ptr as *mut CommandTrb) };
        // 调用闭包来设置命令TRB的字段
        f(trb);
        // 返回命令TRB的指针
        current_ptr
    }

    // 将一个命令TRB加入命令环
    pub fn push_command_trb(&mut self, trb_ptr: *mut u64) {
        info!("push_command_trb");
        // 获取命令TRB的引用
        let trb = unsafe { &mut *(trb_ptr as *mut CommandTrb) };
        // 设置命令TRB的循环位为当前的循环状态
        trb.set_cycle_bit(self.cycle_state);
        // 更新生产者循环索引
        self.cycle_state = (self.cycle_state + 1) % self.ring_size as u32;
    }
}
