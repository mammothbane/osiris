use memory::{Frame, FrameAllocator};
use multiboot2::{MemoryAreaIter, MemoryArea};

pub struct AreaFrameAllocator {
    next_free_frame: Frame,
    current_area: Option<&'static MemoryArea>,
    areas: MemoryAreaIter,
    kernel_start: Frame,
    kernel_end: Frame,
    multiboot_start: Frame,
    multiboot_end: Frame,
}

impl AreaFrameAllocator {
    pub fn new(
        kern_start: usize, kern_end: usize,
        mb_start: usize, mb_end: usize,
        mem_areas: MemoryAreaIter
    ) -> AreaFrameAllocator {
        let mut allocator = AreaFrameAllocator {
            next_free_frame: Frame::containing_addr(0),
            current_area: None,
            areas: mem_areas,
            kernel_start: Frame::containing_addr(kern_start),
            kernel_end: Frame::containing_addr(kern_end),
            multiboot_start: Frame::containing_addr(mb_start),
            multiboot_end: Frame::containing_addr(mb_end),
        };

        allocator.choose_next_area();
        allocator
    }

    fn choose_next_area(&mut self) {
        self.current_area = self.areas.clone().filter(|area| {
            let addr = area.base_addr + area.length - 1;
            Frame::containing_addr(addr as usize) >= self.next_free_frame
        }).min_by_key(|area| area.base_addr);

        self.current_area.map(|area| {
            let start_frame = Frame::containing_addr(area.base_addr as usize);
            if self.next_free_frame < start_frame {
                self.next_free_frame = start_frame;
            }
        });
    }
}

impl FrameAllocator for AreaFrameAllocator {
    fn alloc(&mut self) -> Option<Frame> {
        self.current_area.and_then(|area| {
            let frame = Frame{ index: self.next_free_frame.index };
            let current_area_last_frame = {
                let addr = area.base_addr + area.length - 1;
                Frame::containing_addr(addr as usize)
            };

            if frame > current_area_last_frame {
                self.choose_next_area();
            } else if frame >= self.kernel_start && frame <= self.kernel_end {
                self.next_free_frame = Frame {
                    index: self.kernel_end.index + 1
                };
            } else if frame >= self.multiboot_start && frame <= self.multiboot_end {
                self.next_free_frame = Frame {
                    index: self.multiboot_end.index + 1
                };
            } else {
                self.next_free_frame.index += 1;
                return Some(frame);
            }

            self.alloc()
        })
    }

    fn release(&mut self, _: Frame) {
        unimplemented!()
    }
}