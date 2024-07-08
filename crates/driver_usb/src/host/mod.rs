pub mod device;
pub mod usb;
pub mod xhci;
use ::xhci::ring::trb::{command, event::CommandCompletion};
use alloc::{boxed::Box, sync::Arc, vec::Vec};
use core::alloc::Allocator;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use spinlock::SpinNoIrq;
use xhci::{xhci_device, Xhci};

use crate::{addr::VirtAddr, err::*, OsDep};

#[derive(Clone)]
pub struct USBHostConfig<O>
where
    O: OsDep,
{
    pub(crate) base_addr: VirtAddr,
    pub(crate) irq_num: u32,
    pub(crate) irq_priority: u32,
    pub(crate) os: O,
}

impl<O> USBHostConfig<O>
where
    O: OsDep,
{
    pub fn new(mmio_base_addr: usize, irq_num: u32, irq_priority: u32, os_dep: O) -> Self {
        let base_addr = VirtAddr::from(mmio_base_addr);
        Self {
            base_addr,
            irq_num,
            irq_priority,
            os: os_dep,
        }
    }
}

pub trait Controller<O>: Send
where
    O: OsDep,
{
    fn new(config: USBHostConfig<O>) -> Result<Self>
    where
        Self: Sized;
    fn poll(&mut self, arc: ControllerArc<O>) -> Result<Vec<xhci_device::DeviceAttached<O>>>;

    fn post_cmd(&mut self, trb: command::Allowed) -> Result<CommandCompletion>;
}

pub(crate) type ControllerArc<O> = Arc<SpinNoIrq<Box<dyn Controller<O>>>>;

#[derive(Clone)]
pub struct USBHost<O>
where
    O: OsDep,
{
    pub(crate) config: USBHostConfig<O>,
    pub(crate) controller: ControllerArc<O>,
}

impl<O> USBHost<O>
where
    O: OsDep,
{
    pub fn new<C: Controller<O> + 'static>(config: USBHostConfig<O>) -> Result<Self> {
        let controller: Box<dyn Controller<O>> = Box::new(C::new(config.clone())?);

        let controller = Arc::new(SpinNoIrq::new(controller));
        // let controller = Arc::new( SpinNoIrq::new(controller));
        Ok(Self { config, controller })
    }

    pub fn poll(&self) -> Result {
        let controller = self.controller.clone();
        let mut g = self.controller.lock();
        g.poll(controller)?;

        Ok(())
    }

    pub fn work_temporary_example(&mut self) {
        use crate::ax::USBDeviceDriverOps;
        // unsafe {
        //     xhci::drivers.iter_mut().for_each(|d| {
        //         d.lock().work(
        //             &*((self.controller.as_ref() as *const dyn Controller<O>) as *const Xhci<O>),
        //         );
        //     })
        // }
    }
}

#[derive(Copy, Clone, FromPrimitive)]
pub enum PortSpeed {
    FullSpeed = 1,
    LowSpeed = 2,
    HighSpeed = 3,
    SuperSpeed = 4,
    SuperSpeedPlus = 5,
}
