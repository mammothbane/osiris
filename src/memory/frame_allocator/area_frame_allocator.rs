use memory::{Frame, FrameAllocator};
use memory::frame_set::FrameSetMut;
use memory::frame::IFrame;
use bootinfo::{MemoryMap, MemoryRegion, MemoryRegionType};

pub struct AreaFrameAllocator<T> {
    memory_map: MemoryMap,
    next_free_frame: Frame,
    current_region: Option<MemoryRegion>,
    frame_set: T,
}

impl <T: FrameSetMut> AreaFrameAllocator<T> {
    pub fn new(
        memory_map: MemoryMap,
        frame_set: T,
    ) -> AreaFrameAllocator<T> {
        let mut allocator = AreaFrameAllocator {
            memory_map,
            next_free_frame: Frame::containing_addr(0),
            current_region: None,
            frame_set,
        };

        allocator.choose_next_area();
        allocator
    }

    pub fn set_start_frame(&mut self, f: Frame) {
        self.next_free_frame = f;
        self.choose_next_area();
    }

    fn choose_next_area(&mut self) {
        self.current_region = self.memory_map.iter()
            .filter(|region| {
                let addr = region.range.end_addr() - 1;
                region.region_type == MemoryRegionType::Usable && Frame::containing_addr(addr as usize) >= self.next_free_frame
            })
            .min_by_key(|region| region.range.start_addr())
            .map(|x| *x);

        self.current_region.map(|region| {
            let start_frame = Frame::containing_addr(region.range.start_addr() as usize);
            if self.next_free_frame < start_frame {
                self.next_free_frame = start_frame;
            }
        });
    }

    pub fn next_free(self) -> Frame { self.next_free_frame }
}

impl <T: FrameSetMut> FrameAllocator for AreaFrameAllocator<T> {
    fn alloc(&mut self) -> Option<Frame> {
        self.current_region.and_then(|area| {
            let frame = Frame::new(self.next_free_frame.index());

            let current_area_last_frame = {
                let addr = area.range.end_addr() - 1;
                Frame::containing_addr(addr as usize)
            };

            if frame > current_area_last_frame {
                self.choose_next_area();
            } else {
                let index = self.next_free_frame.index();

                self.next_free_frame.set_index(index + 1);

                self.frame_set.add(frame.clone()).unwrap_or_else(|_| panic!("allocator's frame set was full"));
                return Some(frame);
            }

            self.alloc()
        })
    }

    fn release(&mut self, f: Frame) {
        self.frame_set.remove(f.index()).unwrap_or_else(|_| panic!("unable to release frame"));
    }
}
