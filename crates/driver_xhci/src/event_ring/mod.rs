use alloc::{boxed::Box, vec};
use axhal::mem::PhysAddr;
use bit_field::BitField;
use log::warn;
use xhci::{
    ring::trb::{self, event::Allowed},
    Registers,
};

use crate::MemoryMapper;

use self::segment_table::SegmentTable;

pub mod segment_table;

const MAX_NUM_OF_TRB_IN_QUEUE: usize = 256;

pub(crate) struct EventRing {
    seg_table: Box<SegmentTable>,
    rings: vec::Vec<[[u32; 4]; MAX_NUM_OF_TRB_IN_QUEUE]>,
    counter: usize,
    index: usize,
    erst_num: usize,
    flipsign: bool,
}
impl EventRing {
    pub fn new(r: &mut Registers<MemoryMapper>) -> Self {
        let erst_num = r
            .capability
            .hcsparams2
            .read_volatile()
            .event_ring_segment_table_max() as usize;
        Self {
            seg_table: segment_table::new(erst_num),
            rings: vec![[[0; 4]; MAX_NUM_OF_TRB_IN_QUEUE]; erst_num],
            counter: 0,
            index: 0,
            erst_num,
            flipsign: true,
        }
    }

    pub fn cycle(&mut self) -> Option<Allowed> {
        let next = self.rings[self.index][self.counter];
        let sign = bool::from(next[3].get_bit(0));
        match sign == self.flipsign {
            false => None,
            true => {
                let allowed = Allowed::try_from(next);
                if allowed.is_err() {
                    warn!("Unrecognized ID: {}", next[3].get_bits(10..=15));
                }

                self.counter += 1;
                if self.counter >= MAX_NUM_OF_TRB_IN_QUEUE.into() {
                    self.index += 1;
                    self.counter = 0;

                    if self.index >= self.erst_num {
                        self.index = 0;
                        self.flipsign = !self.flipsign;
                    }
                }

                allowed.ok()
            }
        }
    }

    pub fn update_deq_with_xhci(&self, r: &mut Registers<MemoryMapper>) {
        r.interrupter_register_set
            .interrupter_mut(0)
            .erdp
            .update_volatile(|r| {
                r.set_event_ring_dequeue_pointer(
                    (self.rings[self.index].as_ptr().addr() + trb::BYTES * self.counter) as u64, //TODO maybe convert phys to virt
                );
            });
    }

    pub fn rings_addr(&self) -> vec::Vec<usize> {
        self.rings.iter().map(|v| v.as_ptr().addr()).collect() //TODO: WHOULD Vec convert into primvative array?
    }

    pub fn segment_table_addr(&self) -> PhysAddr {
        self.seg_table.as_ptr().addr().into()
    }

    pub fn num_of_erst(&self) -> usize {
        self.erst_num
    }

    pub(crate) fn init_segtable(&mut self, r: &mut Registers<MemoryMapper>) {
        let head_addrs = self.rings_addr();
        for (ent, add) in self.seg_table.iter_mut().zip(head_addrs) {
            ent.set(add.into(), MAX_NUM_OF_TRB_IN_QUEUE as u16);
        }

        r.interrupter_register_set
            .interrupter_mut(0)
            .erstsz
            .update_volatile(|r| r.set(self.seg_table.len().try_into().unwrap()));

        r.interrupter_register_set
            .interrupter_mut(0)
            .erstba
            .update_volatile(|r| r.set(self.segment_table_addr().as_usize() as u64));
    }
}
