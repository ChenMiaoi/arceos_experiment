pub mod drivers;
pub mod operation;
use alloc::{boxed::Box, sync::Arc, vec::Vec};
use drivers::driverapi::{USBSystemDriverModule, USBSystemDriverModuleInstance};
use log::trace;
use spinlock::SpinNoIrq;
use urb::URB;

use crate::{
    abstractions::PlatformAbstractions,
    glue::driver_independent_device_instance::DriverIndependentDeviceInstance, USBSystemConfig,
};

use self::drivers::DriverContainers;

pub mod descriptors;
pub mod trasnfer;
pub mod urb;

#[cfg(feature = "packed_drivers")]
pub(crate) mod universal_drivers;

pub struct USBDriverSystem<'a, O>
where
    O: PlatformAbstractions,
{
    config: Arc<SpinNoIrq<USBSystemConfig<O>>>,
    managed_modules: DriverContainers<'a, O>,
    driver_device_instances: Vec<Arc<SpinNoIrq<dyn USBSystemDriverModuleInstance<'a, O>>>>,
}

impl<'a, O> USBDriverSystem<'a, O>
where
    O: PlatformAbstractions + 'static,
{
    pub fn new(config: Arc<SpinNoIrq<USBSystemConfig<O>>>) -> Self {
        Self {
            config,
            managed_modules: DriverContainers::new(),
            driver_device_instances: Vec::new(),
        }
    }

    pub fn init(&mut self) {
        #[cfg(feature = "packed_drivers")]
        {
            self.managed_modules.load_driver(Box::new(
                universal_drivers::hid_drivers::hid_mouse::HidMouseDriverModule,
            ))
        }

        trace!("usb system driver modules load complete!")
    }

    /**
     * this method should invoked after driver independent devices created
     */
    pub fn init_probe(
        &mut self,
        devices: &Vec<DriverIndependentDeviceInstance<O>>,
        preparing_list: &mut Vec<Vec<URB<'a, O>>>,
    ) {
        devices
            .iter()
            .flat_map(|device| {
                self.managed_modules
                    .create_for_device(device, self.config.clone(), preparing_list)
            })
            .collect_into(&mut self.driver_device_instances);
        trace!(
            "current driver managed device num: {}",
            self.driver_device_instances.len()
        )
    }
}
