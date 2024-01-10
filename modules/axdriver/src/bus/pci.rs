use crate::{prelude::*, AllDevices};
use axhal::mem::phys_to_virt;
use driver_pci::{
    BarInfo, Cam, Command, DeviceFunction, HeaderType, MemoryBarType, PciRangeAllocator, PciRoot,
};

const PCI_BAR_NUM: u8 = 6;

fn config_pci_device(
    root: &mut PciRoot,
    bdf: DeviceFunction,
    allocator: &mut Option<PciRangeAllocator>,
) -> DevResult {
    let mut bar = 0;
    while bar < PCI_BAR_NUM {
        match root.bar_info(bdf, bar) {
            Ok(info) => {
                if let BarInfo::Memory {
                    address_type,
                    address,
                    size,
                    ..
                } = info
                {
                    // if the BAR address is not assigned, call the allocator and assign it.
                    if size > 0 && address == 0 {
                        let new_addr = allocator
                            .as_mut()
                            .expect("No memory ranges available for PCI BARs!")
                            .alloc(size as _)
                            .ok_or(DevError::NoMemory)?;
                        // let alloced = global_allocator.alloc(Layout::for_value(&new_addr));
                        if address_type == MemoryBarType::Width32 {
                            root.set_bar_32(bdf, bar, new_addr as _);
                            // info!("type 32");
                        } else if address_type == MemoryBarType::Width64 {
                            root.set_bar_64(bdf, bar, new_addr);
                            // info!("type 64");
                        }
                    }
                }
            }
            Err(_) => {}
        };

        // read the BAR info again after assignment.
        match root.bar_info(bdf, bar) {
            Ok(info) => {
                match info {
                    BarInfo::IO { address, size } => {
                        if address > 0 && size > 0 {
                            debug!("  BAR {}: IO  [{:#x}, {:#x})", bar, address, address + size);
                        }
                    }
                    BarInfo::Memory {
                        address_type,
                        prefetchable,
                        address,
                        size,
                    } => {
                        if address > 0 && size > 0 {
                            debug!(
                                "  BAR {}: MEM [{:#x}, {:#x}){}{}",
                                bar,
                                address,
                                address + size as u64,
                                if address_type == MemoryBarType::Width64 {
                                    " 64bit"
                                } else {
                                    ""
                                },
                                if prefetchable { " pref" } else { "" },
                            );
                        }
                    }
                }

                bar += 1;
                if info.takes_two_entries() {
                    info!("two ents");
                    bar += 1;
                }
            }
            Err(_) => {
                bar += 1;
            }
        };
    }

    // Enable the device.
    let (_status, cmd) = root.get_status_command(bdf);
    root.set_command(
        bdf,
        cmd | Command::IO_SPACE | Command::MEMORY_SPACE | Command::BUS_MASTER,
    );
    Ok(())
}

impl AllDevices {
    pub(crate) fn probe_bus_devices(&mut self) {
        let base_vaddr = phys_to_virt(axconfig::PCI_ECAM_BASE.into());
        // let base_paddr: usize = 0x6_0000_0000;
        // let base_paddr: usize = 0xfd50_0000;
        // let base_vaddr = phys_to_virt(base_paddr.into());
        info!("base_vaddr:{:x}", base_vaddr.as_usize());

        driver_pci::init(base_vaddr);
        let mut root = unsafe { PciRoot::new(base_vaddr.as_mut_ptr(), Cam::Ecam) };

        // PCI 32-bit MMIO space
        let mut allocator = axconfig::PCI_RANGES
            .get(1)
            .map(|range| PciRangeAllocator::new(range.0 as u64, range.1 as u64));
        // let allocator = axalloc::global_allocator();

        // info!("pci_ranges = ({},{})", axconfig::PCI_RANGES.first());

        'out: for bus in 0..=axconfig::PCI_BUS_END as u8 {
            // info!("iter {bus}");
            for (bdf, dev_info) in root.enumerate_bus(bus) {
                if !((dev_info.class == 0xc as u8) && (dev_info.subclass == 0x3 as u8)) {
                    //hard code, need modify
                    continue;
                } else {
                    debug!("PCI {}: {}", bdf, dev_info);

                    // info!("in!");
                    if dev_info.header_type != HeaderType::Standard {
                        // info!("continue enum");
                        continue;
                    }

                    // if dev_info.class == 0 {
                    //     info!("skipped unknown device");
                    //     continue;
                    // }
                    // info!("{} == 0: {}", dev_info.class, dev_info.class == 0);

                    match config_pci_device(&mut root, bdf, &mut allocator) {
                        Ok(_) => for_each_drivers!(type Driver, {
                            if let Some(dev) = Driver::probe_pci(&mut root, bdf, &dev_info) {
                                info!(
                                    "registered a new {:?} device at {}: {:?}",
                                    dev.device_type(),
                                    bdf,
                                    dev.device_name(),
                                );
                                self.add_device(dev);
                                break 'out; // skip to the next device
                            }
                        }),
                        Err(e) => warn!(
                            "failed to enable PCI device at {}({}): {:?}",
                            bdf, dev_info, e
                        ),
                    } //todo fix memory alloc
                      // info!("break!");
                    break 'out;
                }
                // info!("iter complete")
            }
        }
    }
}
