use memory::{Frame, FrameAllocator};
use memory::frame_set::EmptyFrameSet;
use super::{ActivePageTable, Page, VirtualAddr};
use super::table::{Level1, Table};

pub struct TemporaryPage {
    page: Page,
    alloc: TinyAllocator,
}

struct TinyAllocator([Option<Frame>; 3]);

impl TemporaryPage {
    pub fn new<A>(page: Page, alloc: &mut A) -> TemporaryPage
        where A: FrameAllocator
    {
        TemporaryPage {
            page,
            alloc: TinyAllocator::new(alloc),
        }
    }

    pub fn from_frames(page: Page, frames: [Frame; 3]) -> TemporaryPage {
        TemporaryPage {
            page,
            alloc: TinyAllocator::from_frames(frames)
        }
    }

    pub fn map(&mut self, frame: Frame, active_table: &mut ActivePageTable) -> VirtualAddr {
        use super::entry::WRITABLE;

        assert!(active_table.translate_page(self.page).is_none(),
            "page is already mapped");

        active_table.map_to(self.page, frame, WRITABLE, &mut self.alloc);
        self.page.start_addr()
    }

    // use level1 table to forbid calling next_table
    pub fn map_table_frame(&mut self, frame: Frame, active_table: &mut ActivePageTable)
        -> &mut Table<Level1> {
        unsafe { &mut *(self.map(frame, active_table) as *mut Table<Level1>) }
    }

    pub fn unmap(&mut self, active_table: &mut ActivePageTable) {
        active_table.unmap(self.page, &mut self.alloc)
    }
}

impl TinyAllocator {
    fn new<A>(allocator: &mut A) -> TinyAllocator
        where A: FrameAllocator {
        let mut f = || allocator.alloc();
        let frames = [f(), f(), f()];
        TinyAllocator(frames)
    }

    fn from_frames(frames: [Frame; 3]) -> TinyAllocator {
        TinyAllocator([Some(frames[0]), Some(frames[1]), Some(frames[2])])
    }
}

impl FrameAllocator for TinyAllocator {
    type FrameSetImpl = EmptyFrameSet;

    fn alloc(&mut self) -> Option<Frame> {
        self.0.iter_mut().find(|x| x.is_some()).and_then(|x| x.take())
    }

    fn release(&mut self, frame: Frame) {
        self.0.iter_mut().find(|x| x.is_none())
            .map(|x| *x = Some(frame))
            .or_else(|| { panic!("Tiny allocator can only hold 3 frames")});
    }

    fn allocated_frames(&self) -> Self::FrameSetImpl {
        EmptyFrameSet
    }
}