use crate::{
    dma,
    host::structures::{
        event_ring, xhci_command_manager::command_completed, xhci_roothub::status_changed,
        XHCI_CMD_COMPLETION_EVENT_TRB_CONTROL_SLOTID_SHIFT,
        XHCI_EVENT_TRB_STATUS_COMPLETION_CODE_SHIFT,
        XHCI_PORT_STATUS_EVENT_TRB_PARAMETER1_PORTID_SHIFT,
    },
};
use conquer_once::spin::OnceCell;
use core::f32::consts::E;
use log::warn;
use log::{debug, info};
use page_box::PageBox;
use spinning_top::Spinlock;
use xhci::ring::trb::command::Allowed as CommandAllowed;
use xhci::ring::trb::event::Allowed as EventAllowed;
use xhci::ring::trb::transfer::Allowed as TransferAllowed;
use xhci::ring::trb::{event::CompletionCode, transfer::Normal, Type};

use super::{
    event_ring::{EvtRing, TypeXhciTrb},
    registers, XHCI_CONFIG_IMODI,
};
#[derive(Clone)]
struct ErstEntry {
    pub seg_base: usize,
    pub seg_size: u32,
    pub reserved: u32,
}

pub(crate) struct EventManager {
    event_ring: EvtRing,
    erst_entry: PageBox<[ErstEntry]>, // event ring segment table
}

pub(crate) static EVENT_MANAGER: OnceCell<Spinlock<EventManager>> = OnceCell::uninit();

pub(crate) fn new() {
    debug!("initilizating!");
    let mut event_manager = EventManager {
        event_ring: EvtRing::new(),
        erst_entry: PageBox::new_slice(
            ErstEntry {
                seg_base: 0,
                seg_size: 0,
                reserved: 0,
            },
            1,
        ),
    };
    registers::handle(|r| {
        debug!("test");
        let erst_ent = &mut event_manager.erst_entry[0];
        erst_ent.seg_base = event_manager.event_ring.get_ring_addr().as_usize();
        erst_ent.seg_size = event_manager.event_ring.get_trb_count() as u32;
        erst_ent.reserved = 0;

        let mut ir0 = r.interrupter_register_set.interrupter_mut(0);
        ir0.erstsz.update_volatile(|e| {
            e.set(1);
        });

        ir0.erstba.update_volatile(|b| {
            b.set(event_manager.erst_entry.virt_addr().as_usize() as u64);
        });
        //TODO FIXIT
        ir0.erdp.update_volatile(|dp| {
            dp.set_event_ring_dequeue_pointer(
                event_manager.event_ring.get_ring_addr().as_usize() as u64
            );
        });
        ir0.imod.update_volatile(|im| {
            im.set_interrupt_moderation_interval(XHCI_CONFIG_IMODI);
        });
        ir0.iman.update_volatile(|im| {
            im.set_interrupt_enable();
        });

        EVENT_MANAGER
            .try_init_once(move || Spinlock::new(event_manager))
            .expect("Failed to initialize `EventManager`.");

        //     let slot_manager = SlotManager {
        //         dcbaa: PageBox::new_slice(VirtAddr::from(0 as usize), XHCI_CONFIG_MAX_SLOTS + 1),
        //         device: PageBox::new_slice(Device::new_64byte(), XHCI_CONFIG_MAX_SLOTS + 1),
        //     };

        //     r.operational
        //         .dcbaap
        //         .update_volatile(|d| d.set(slot_manager.dcbaa.virt_addr()));

        //     SLOT_MANAGER
        //         .try_init_once(move || Spinlock::new(slot_manager))
        //         .expect("Failed to initialize `SlotManager`.");
    });

    debug!("initialized!");
}

pub(crate) fn handle_event() -> Result<TypeXhciTrb, ()> {
    debug!("start to handle event...\n");
    //TODO 事件环返回的TRB应当被入队，需要修改
    if let Some(mut manager) = EVENT_MANAGER.get().unwrap().try_lock() {
        if let Some(trb) = manager.event_ring.get_deque_trb() {
            match trb {
                EventAllowed::TransferEvent(evt) => {
                    debug!("event = {:?}", evt);
                    debug!("step into transfer event\n");
                    // let trb_array = trb.into_raw();
                    // TODO: transfer_event
                    // transfer_event(
                    //         trb_array[2] >> XHCI_EVENT_TRB_STATUS_COMPLETION_CODE_SHIFT,
                    //         trb_array[2] & XHCI_TRANSFER_EVENT_TRB_STATUS_TRB_TRANSFER_LENGTH_MASK,
                    //         trb_array[3] >> XHCI_CMD_COMPLETION_EVENT_TRB_CONTROL_SLOTID_SHIFT,
                    //         (trb_array[3] & XHCI_TRANSFER_EVENT_TRB_CONTROL_ENDPOINTID_MASK) >> XHCI_TRANSFER_EVENT_TRB_CONTROL_ENDPOINTID_SHIFT
                    // )
                }
                EventAllowed::CommandCompletion(_) => {
                    debug!("step into command completion.\n");
                    let trb_array = trb.into_raw();
                    command_completed(
                        (((trb_array[0] as usize) << 32) | ((trb_array[1] as usize) << 32)).into(),
                        (trb_array[2] >> XHCI_EVENT_TRB_STATUS_COMPLETION_CODE_SHIFT)
                            .try_into()
                            .unwrap(),
                        (trb_array[3] >> XHCI_CMD_COMPLETION_EVENT_TRB_CONTROL_SLOTID_SHIFT)
                            .try_into()
                            .unwrap(),
                    );
                    return Ok(TypeXhciTrb::default());
                }
                EventAllowed::PortStatusChange(_) => {
                    debug!("step into port status change.\n");
                    let trb_array = trb.into_raw();
                    assert!(
                        trb_array[2] >> XHCI_EVENT_TRB_STATUS_COMPLETION_CODE_SHIFT
                            == CompletionCode::Success as u32
                    );
                    debug!("exec handler!");
                    status_changed(
                        (trb_array[0] >> XHCI_PORT_STATUS_EVENT_TRB_PARAMETER1_PORTID_SHIFT)
                            .try_into()
                            .unwrap(),
                    );
                }
                EventAllowed::BandwidthRequest(_) => todo!(),
                EventAllowed::Doorbell(_) => todo!(),
                EventAllowed::HostController(_) => {
                    debug!("step into host controller.\n");
                    let trb_array = trb.into_raw();
                    let uch_completion_code =
                        trb_array[2] >> XHCI_EVENT_TRB_STATUS_COMPLETION_CODE_SHIFT;
                    use num_traits::cast::FromPrimitive;
                    match CompletionCode::from_u32(uch_completion_code) {
                        Some(CompletionCode::EventRingFullError) => {
                            warn!("Event ring full");
                        }
                        _ => todo!(),
                    }
                    debug!("Host controller event completion")
                }
                EventAllowed::DeviceNotification(_) => todo!(),
                EventAllowed::MfindexWrap(_) => todo!(),
            }
        };
        manager.event_ring.inc_deque();
    }
    return Err(());
}
