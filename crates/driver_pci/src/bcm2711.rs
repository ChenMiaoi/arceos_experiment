use axhal::mem::VirtAddr;
use brcm_pcie::*;



struct Hal;

impl brcm_pcie::BCM2711Hal for Hal {
    fn sleep(ms: core::time::Duration) {
        axhal::time::busy_wait(ms)
    }
}




pub(crate) fn init_pci(base: VirtAddr){
    let pcie_host_bridge = BCM2711PCIeHostBridge::<Hal>::new(base.as_usize());
    pcie_host_bridge.setup();
}