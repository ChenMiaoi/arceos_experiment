use alloc::vec::{self, Vec};
use axhal::mem::PhysAddr;
use xhci::{accessor::Mapper, registers::doorbell::Register, Registers};

pub(crate) struct DeviceContextBaseAddressArray {
    devices: Vec<PhysAddr>,
}

impl DeviceContextBaseAddressArray {
    fn new(slot_count: usize) -> Self {
        let arr = vec![(0 as usize).into(); slot_count];
        Self { devices: arr }
    }

    fn init(&self, register: &mut Registers) {
        self.register_address_to_xhci_register(register);
    }

    fn register_address_to_xhci_register(&self, r: &mut Registers) {
        let _ = &self;
        r.operational.dcbaap.update_volatile(|d| {
            let _ = &self;
            d.set(self.phys_addr().as_u64());
        });
    }

    fn phys_addr(&self) -> PhysAddr {
        self.arr.phys_addr()
    }
}

impl Index<usize> for DeviceContextBaseAddressArray {
    type Output = PhysAddr;
    fn index(&self, index: usize) -> &Self::Output {
        &self.devices[index]
    }
}
impl IndexMut<usize> for DeviceContextBaseAddressArray {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.devices[index]
    }
}
