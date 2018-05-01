use core::sync::atomic::{AtomicUsize, Ordering};

use core::alloc::{GlobalAlloc, Layout, Opaque};

#[derive(Debug)]
pub struct BumpAllocator {
    heap_start: usize,
    heap_end: usize,
    next: AtomicUsize,
}

impl BumpAllocator {
    pub const fn empty() -> Self {
        BumpAllocator {
            heap_start: 0,
            heap_end: 0,
            next: AtomicUsize::new(0),
        }
    }

    pub fn init(&mut self, heap_start: usize, heap_end: usize) {
        self.heap_start = heap_start;
        self.heap_end = heap_end;
        self.next.store(heap_start, Ordering::Relaxed);
    }
}

pub fn align_down(addr: usize, align: usize) -> usize {
    if align.is_power_of_two() {
        addr & !(align - 1)
    } else if align == 0 {
        addr
    } else {
        panic!("`align` must be a power of 2");
    }
}

pub fn align_up(addr: usize, align: usize) -> usize {
    align_down(addr + align - 1, align)
}

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut Opaque {
        loop {
            let current_next = self.next.load(Ordering::Relaxed);
            let alloc_start = align_up(current_next, layout.align());
            let alloc_end = alloc_start.saturating_add(layout.size());

            if alloc_end <= self.heap_end {
                let next_now = self.next.compare_and_swap(current_next, alloc_end, Ordering::Relaxed);

                if next_now == current_next {
                    return alloc_start as *mut Opaque;
                }
            } else {
                return 0 as *mut Opaque;
            }
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut Opaque, _layout: Layout) {
        // leak
    }
}