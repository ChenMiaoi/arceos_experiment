// SPDX-License-Identifier: GPL-3.0-or-later

use {
    super::dcbaa, crate::host::structures::registers, alloc::vec::Vec, axhal::mem::PhysAddr,
    conquer_once::spin::OnceCell, core::convert::TryInto, page_box::PageBox,
};

static SCRATCHPAD: OnceCell<Scratchpad> = OnceCell::uninit();

pub(crate) fn init() {
    if Scratchpad::exists() {
        init_static();
    }
}

fn init_static() {
    let mut scratchpad = Scratchpad::new();
    scratchpad.init();
    scratchpad.register_with_dcbaa();

    SCRATCHPAD.init_once(|| scratchpad);
}

struct Scratchpad {
    arr: PageBox<[PhysAddr]>,
    bufs: Vec<PageBox<[u8]>>,
}
impl Scratchpad {
    fn new() -> Self {
        let len: usize = Self::num_of_buffers().try_into().unwrap();

        Self {
            arr: PageBox::new_slice(PhysAddr::from(0 as usize), len),
            bufs: Vec::new(),
        }
    }

    fn exists() -> bool {
        Self::num_of_buffers() > 0
    }

    fn init(&mut self) {
        self.allocate_buffers();
        self.write_buffer_addresses();
    }

    fn register_with_dcbaa(&self) {
        dcbaa::register(0, self.arr.phys_addr());
    }

    fn allocate_buffers(&mut self) {
        for _ in 0..Self::num_of_buffers() {
            // Allocate the double size of memory, then register the aligned address with the
            // array.
            let b = PageBox::new_slice(0, Self::page_size() * 2);
            self.bufs.push(b);
        }
    }

    fn write_buffer_addresses(&mut self) {
        let page_size = Self::page_size();
        for (x, buf) in self.arr.iter_mut().zip(self.bufs.iter()) {
            *x = buf.phys_addr().align_up(page_size);
        }
    }

    fn num_of_buffers() -> u32 {
        registers::handle(|r| {
            r.capability
                .hcsparams2
                .read_volatile()
                .max_scratchpad_buffers()
        })
    }

    fn page_size() -> usize {
        registers::handle(|r| r.operational.pagesize.read_volatile().get()).into()
    }
}
